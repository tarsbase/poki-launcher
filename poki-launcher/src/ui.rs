/***
 * This file is part of Poki Launcher.
 *
 * Poki Launcher is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Poki Launcher is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Poki Launcher.  If not, see <https://www.gnu.org/licenses/>.
 */
use anyhow::Error;
use cstr::*;
use lazy_static::lazy_static;
use lib_poki_launcher::prelude::*;
use log::{debug, error, trace, warn};
use notify::{watcher, RecursiveMode, Watcher};
use poki_launcher_notifier::{self as notifier, Notifier};
use poki_launcher_x11::foreground;
use qmetaobject::*;
use std::cell::{Cell, RefCell};
use std::convert::From;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const MAX_APPS_SHOWN: usize = 5;

pub fn log_errs(errs: &[Error]) {
    for err in errs {
        error!("{}", err);
    }
}

lazy_static! {
    pub static ref DB_PATH: PathBuf = {
        use std::fs::create_dir;
        let data_dir = DIRS.data_dir();
        if !data_dir.exists() {
            create_dir(&data_dir).unwrap_or_else(|_| {
                panic!("Failed to create data dir: {}", data_dir.to_string_lossy())
            });
        }
        let mut db_file = data_dir.to_path_buf();
        db_file.push("apps.db");
        db_file
    };
    pub static ref APPS: Arc<Mutex<AppsDB>> = {
        let config = match Config::load() {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to load config file: {}", e);
                std::process::exit(2);
            }
        };
        let apps = if DB_PATH.exists() {
            debug!("Loading db from: {}", DB_PATH.display());
            match AppsDB::load(&*DB_PATH, config) {
                Ok(apps) => apps,
                Err(e) => {
                    error!("Failed to load database file: {}", e);
                    std::process::exit(3);
                }
            }
        } else {
            let (apps, errors) = AppsDB::from_desktop_entries(config);
            log_errs(&errors);
            // TODO visual error indicator
            if let Err(e) = apps.save(&*DB_PATH) {
                error!("Failed to save apps database to disk: {}", e);
            }
            apps
        };
        Arc::new(Mutex::new(apps))
    };
}

thread_local! {
    pub static SHOW_ON_START: Cell<bool> = Cell::new(true);
}

#[derive(QObject, Default)]
struct PokiLauncher {
    base: qt_base_class!(trait QObject),
    list: Vec<App>,
    model: qt_property!(RefCell<SimpleListModel<QApp>>; NOTIFY model_changed),
    selected: qt_property!(QString; NOTIFY selected_changed WRITE set_selected),
    visible: qt_property!(bool; NOTIFY visible_changed),
    scanning: qt_property!(bool; NOTIFY scanning_changed),
    has_moved: qt_property!(bool),

    init: qt_method!(fn(&mut self)),
    search: qt_method!(fn(&mut self, text: String)),
    scan: qt_method!(fn(&mut self)),
    down: qt_method!(fn(&mut self)),
    up: qt_method!(fn(&mut self)),
    run: qt_method!(fn(&mut self)),
    hide: qt_method!(fn(&mut self)),
    exit: qt_method!(fn(&mut self)),

    selected_changed: qt_signal!(),
    visible_changed: qt_signal!(),
    scanning_changed: qt_signal!(),
    model_changed: qt_signal!(),
}

impl PokiLauncher {
    fn init(&mut self) {
        self.scan();

        // Setup signal notifier and callback
        self.visible = SHOW_ON_START.with(|b| b.get());
        self.visible_changed();
        let rx = match Notifier::start() {
            Ok(rx) => rx,
            Err(e) => {
                error!("Failed to start signal notifier: {}", e);
                std::process::exit(5);
            }
        };
        let qptr = QPointer::from(&*self);
        let show = qmetaobject::queued_callback(move |()| {
            qptr.as_pinned().map(|self_| {
                self_.borrow_mut().visible = true;
                self_.borrow().visible_changed();
            });
        });
        thread::spawn(move || loop {
            use notifier::Msg;
            match rx.recv() {
                Ok(msg) => match msg {
                    Msg::Show => {
                        show(());
                        foreground("Poki Launcher");
                    }
                    Msg::Exit => {
                        drop(rx);
                        std::process::exit(0);
                    }
                },
                Err(e) => {
                    warn!("Signal notifier notifier error: {}", e);
                }
            }
        });

        // Setup desktop file change notifier and callback
        let qptr = QPointer::from(&*self);
        let rescan = qmetaobject::queued_callback(move |()| {
            qptr.as_pinned().map(|self_| {
                self_.borrow_mut().scan();
            });
        });
        thread::spawn(move || {
            let (tx, rx) = mpsc::channel();
            let mut watcher = match watcher(tx, Duration::from_secs(10)) {
                Ok(watcher) => watcher,
                Err(e) => {
                    error!("Error creating file system watcher: {}", e);
                    return;
                }
            };

            for path in &APPS.lock().expect("Apps Mutex Poisoned").config.app_paths {
                let expanded = match shellexpand::full(&path) {
                    Ok(path) => path.into_owned(),
                    Err(e) => {
                        error!("Failed to expand desktop files dir path {}: {:?}", path, e);
                        continue;
                    }
                };
                let path = Path::new(&expanded);
                if path.exists() {
                    if let Err(e) = watcher.watch(path, RecursiveMode::Recursive) {
                        warn!("Failed to set watcher for dir {}: {}", expanded, e);
                    }
                }
            }
            loop {
                match rx.recv() {
                    Ok(event) => {
                        debug!("Desktop file watcher received {:?}", event);
                        rescan(());
                    }
                    Err(e) => {
                        debug!("Desktop file watcher error {}", e);
                        return;
                    }
                }
            }
        });
    }

    fn set_selected<T: Into<QString>>(&mut self, selected: T) {
        self.selected = selected.into().into();
        self.selected_changed();
    }

    fn get_selected(&self) -> String {
        self.selected.clone().into()
    }

    fn search(&mut self, text: String) {
        self.list = APPS
            .lock()
            .expect("Apps Mutex Poisoned")
            .get_ranked_list(&text, Some(MAX_APPS_SHOWN));
        if !self.has_moved || !self.list.iter().any(|app| app.uuid == self.get_selected()) {
            if !self.list.is_empty() {
                self.set_selected(self.list[0].uuid.clone());
            } else {
                self.set_selected(QString::default());
            }
        }
        self.model
            .borrow_mut()
            .reset_data(self.list.clone().into_iter().map(QApp::from).collect());
    }

    fn scan(&mut self) {
        trace!("Scanning...");
        self.scanning = true;
        self.scanning_changed();
        let qptr = QPointer::from(&*self);
        let done = qmetaobject::queued_callback(move |()| {
            qptr.as_pinned().map(|self_| {
                self_.borrow_mut().scanning = false;
                self_.borrow().scanning_changed();
            });
        });
        thread::spawn(move || {
            let mut apps = APPS.lock().expect("Apps Mutex Poisoned");
            let (app_list, errors) = scan_desktop_entries(&apps.config.app_paths);
            apps.merge_new_entries(app_list);
            if let Err(e) = apps.save(&*DB_PATH) {
                error!("Saving database failed: {}", e);
            }
            log_errs(&errors);
            done(());
            trace!("Scanning...done");
        });
    }

    fn down(&mut self) {
        trace!("Down");
        if self.list.is_empty() {
            return;
        }
        self.has_moved = true;
        let (idx, _) = self
            .list
            .iter()
            .enumerate()
            .find(|(_, app)| app.uuid == self.get_selected())
            .unwrap();
        if idx >= self.list.len() - 1 {
            self.set_selected(self.list[self.list.len() - 1].uuid.clone());
        } else {
            self.set_selected(self.list[idx + 1].uuid.clone());
        }
    }

    fn up(&mut self) {
        trace!("Up");
        if self.list.is_empty() {
            return;
        }
        self.has_moved = true;
        let (idx, _) = self
            .list
            .iter()
            .enumerate()
            .find(|(_, app)| app.uuid == self.get_selected())
            .unwrap();
        if idx == 0 {
            self.set_selected(self.list[0].uuid.clone());
        } else {
            self.set_selected(self.list[idx - 1].uuid.clone());
        }
    }

    fn run(&mut self) {
        trace!("Run");
        if self.list.is_empty() {
            return;
        }
        self.has_moved = false;
        let app = self
            .list
            .iter()
            .find(|app| app.uuid == self.get_selected())
            .unwrap();
        let mut apps = APPS.lock().expect("Apps Mutex Poisoned");
        if let Err(err) = app.run(&apps.config) {
            error!("{}", err);
        }
        apps.update(app);
        if let Err(e) = apps.save(&*DB_PATH) {
            error!("Failed to save apps database to disk: {}", e);
        }
        self.list.clear();
        self.model.borrow_mut().reset_data(Default::default());
        self.set_selected(QString::default());
    }

    fn hide(&mut self) {
        trace!("Hide");
        self.has_moved = false;
        self.visible = false;
        self.visible_changed();
    }

    fn exit(&mut self) {
        trace!("Exit");
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        if let Err(e) = kill(Pid::this(), Signal::SIGINT) {
            error!("Failed to signal self to exit: {}", e);
        }
    }
}

// impl Default for PokiLauncher {
//     fn default() -> Self {
//         PokiLauncher {
//             visible: true,
//             base: Default::default(),
//             list: Default::default(),
//             model: Default::default(),
//             selected: Default::default(),
//             init: Default::default(),
//             scanning: Default::default(),
//             scanning_changed: Default::default(),
//             visible_changed: Default::default(),
//             selected_changed: Default::default(),
//             search: Default::default(),
//             down: Default::default(),
//             up: Default::default(),
//             scan: Default::default(),
//             get_icon: Default::default(),
//             hide: Default::default(),
//             run: Default::default(),
//             exit: Default::default(),
//         }
//     }
// }

#[derive(Default, Clone, SimpleListItem)]
struct QApp {
    pub name: String,
    pub uuid: String,
    pub icon: String,
}

impl From<App> for QApp {
    fn from(app: App) -> QApp {
        QApp {
            name: app.name,
            uuid: app.uuid,
            icon: app.icon,
        }
    }
}

impl QMetaType for QApp {}

qrc!(init_qml_resources,
    "ui" {
        "ui/main.qml" as "main.qml",
        "ui/MainForm.ui.qml" as "MainForm.ui.qml",
    }
);

pub fn init_ui() {
    init_qml_resources();
    qml_register_type::<PokiLauncher>(cstr!("PokiLauncher"), 1, 0, cstr!("PokiLauncher"));
}
