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
use super::interface::*;
use failure::Error;
use gtk::{Application, IconLookupFlags, IconTheme, IconThemeExt};
use lazy_static::lazy_static;
use lib_poki_launcher::prelude::*;
use log::{error, trace, warn};
use poki_launcher_notifier::{self as notifier, Notifier};
use poki_launcher_x11::foreground;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

const MAX_APPS_SHOWN: usize = 5;

fn log_errs(errs: &[Error]) {
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
    pub static ref SHOW_ON_START: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
}

pub struct AppsModel {
    emit: AppsModelEmitter,
    model: AppsModelList,
    list: Vec<App>,
    apps: Arc<Mutex<AppsDB>>,
    selected_item: String,
    window_visible: Arc<AtomicBool>,
    config: Config,
    scanning: Arc<AtomicBool>,
}

fn setup_notifier(
    mut emit: AppsModelEmitter,
    window_visible: Arc<AtomicBool>,
) -> Result<(), Error> {
    let rx = Notifier::start()?;
    thread::spawn(move || loop {
        use notifier::Msg;
        match rx.recv().unwrap() {
            Msg::Show => {
                window_visible.store(true, Ordering::Relaxed);
                emit.visible_changed();
                foreground("Poki Launcher");
            }
            Msg::Exit => {
                drop(rx);
                std::process::exit(0);
            }
        }
    });
    Ok(())
}

impl AppsModelTrait for AppsModel {
    fn new(mut emit: AppsModelEmitter, model: AppsModelList) -> AppsModel {
        let _application =
            Application::new(Some("info.bengoldberg.poki_launcher"), Default::default())
                .expect("failed to initialize GTK application");
        let config = Config::load().unwrap();
        let apps = if DB_PATH.exists() {
            AppsDB::load(&*DB_PATH).unwrap()
        } else {
            let (apps, errors) = AppsDB::from_desktop_entries(&config.app_paths);
            log_errs(&errors);
            apps.save(&*DB_PATH).unwrap();
            apps
        };

        let paths = config.app_paths.clone();
        thread::spawn(move || {
            trace!("Scanning...");
            let (app_list, errors) = scan_desktop_entries(&paths);
            let apps = AppsDB::new(app_list);
            if let Err(e) = apps.save(&*DB_PATH) {
                error!("Saving database failed: {}", e);
            }
            log_errs(&errors);
            trace!("Scanning...done");
        });

        setup_notifier(emit.clone(), SHOW_ON_START.clone()).expect("Failed to setup notifier");
        let scanning = Arc::new(AtomicBool::new(false));

        AppsModel {
            emit,
            model,
            list: Vec::new(),
            apps: Arc::new(Mutex::new(apps)),
            selected_item: String::new(),
            window_visible: SHOW_ON_START.clone(),
            config,
            scanning,
        }
    }

    fn emit(&mut self) -> &mut AppsModelEmitter {
        &mut self.emit
    }

    fn row_count(&self) -> usize {
        self.list.len()
    }

    fn selected(&self) -> &str {
        &self.selected_item
    }

    fn set_selected(&mut self, value: String) {
        self.selected_item = value;
    }

    fn visible(&self) -> bool {
        self.window_visible.load(Ordering::Relaxed)
    }

    fn set_visible(&mut self, value: bool) {
        self.window_visible.store(value, Ordering::Relaxed);
    }

    fn is_scanning(&self) -> bool {
        self.scanning.load(Ordering::Relaxed)
    }

    fn set_is_scanning(&mut self, value: bool) {
        self.scanning.store(value, Ordering::Relaxed);
    }

    fn name(&self, index: usize) -> &str {
        if index < self.list.len() {
            &self.list[index].name
        } else {
            ""
        }
    }

    fn uuid(&self, index: usize) -> &str {
        if index < self.list.len() {
            &self.list[index].uuid
        } else {
            ""
        }
    }

    fn icon(&self, index: usize) -> &str {
        if index < self.list.len() {
            &self.list[index].icon
        } else {
            ""
        }
    }

    fn scan(&mut self) {
        trace!("Scanning...");
        self.scanning.store(true, Ordering::Relaxed);
        self.emit.is_scanning_changed();
        let mut emit = self.emit.clone();
        let scanning = self.scanning.clone();
        let apps = self.apps.clone();
        let config = self.config.clone();
        thread::spawn(move || {
            let (app_list, errors) = scan_desktop_entries(&config.app_paths);
            let apps = {
                let mut apps = apps.lock().expect("Apps Mutex Poisoned");
                apps.merge_new_entries(app_list);
                apps.clone()
            };
            if let Err(e) = apps.save(&*DB_PATH) {
                error!("Saving database failed: {}", e);
            }
            log_errs(&errors);
            scanning.store(false, Ordering::Relaxed);
            emit.is_scanning_changed();
            trace!("Scanning...done");
        });
    }

    fn search(&mut self, text: String) {
        self.model.begin_reset_model();
        self.list = self
            .apps
            .lock()
            .expect("Apps Mutex Poisoned")
            .get_ranked_list(&text, Some(MAX_APPS_SHOWN));
        if !self.list.is_empty() {
            self.selected_item = self.list[0].uuid.clone();
        } else {
            self.selected_item = String::new();
        }
        self.model.end_reset_model();
    }

    fn down(&mut self) {
        if self.list.is_empty() {
            return;
        }
        self.model.begin_reset_model();
        let (idx, _) = self
            .list
            .iter()
            .enumerate()
            .find(|(_, app)| app.uuid == self.selected_item)
            .unwrap();
        if idx >= self.list.len() - 1 {
            self.selected_item = self.list[self.list.len() - 1].uuid.clone();
        } else {
            self.selected_item = self.list[idx + 1].uuid.clone();
        }
        self.model.end_reset_model();
    }

    fn up(&mut self) {
        if self.list.is_empty() {
            return;
        }
        self.model.begin_reset_model();
        let (idx, _) = self
            .list
            .iter()
            .enumerate()
            .find(|(_, app)| app.uuid == self.selected_item)
            .unwrap();
        if idx == 0 {
            self.selected_item = self.list[0].uuid.clone();
        } else {
            self.selected_item = self.list[idx - 1].uuid.clone();
        }
        self.model.end_reset_model();
    }

    fn run(&mut self) {
        if self.list.is_empty() {
            return;
        }
        self.model.begin_reset_model();
        let app = self
            .list
            .iter()
            .find(|app| app.uuid == self.selected_item)
            .unwrap();
        if let Err(err) = app.run() {
            error!("{}", err);
        }
        let mut apps = self.apps.lock().expect("Apps Mutex Poisoned");
        apps.update(app);
        apps.save(&*DB_PATH).unwrap();
        self.list.clear();
        self.model.end_reset_model();
    }

    fn get_icon(&self, name: String) -> String {
        if Path::new(&name).is_absolute() {
            name
        } else {
            let theme = if self.config.icon_theme.is_some() {
                use std::ops::Deref as _;
                let theme = IconTheme::new();
                let name = self.config.icon_theme.as_ref().map(|v| v.deref());
                theme.set_custom_theme(name);
                theme
            } else {
                IconTheme::get_default().expect("Couldn't get default icon theme.")
            };
            // let theme = IconTheme::get_default().unwrap();
            let icon = match theme.lookup_icon(&name, 128, IconLookupFlags::empty()) {
                Some(icon) => icon,
                None => {
                    warn!("No icon found for {}", name);
                    return String::new();
                }
            };
            icon.get_filename()
                .unwrap()
                .clone()
                .to_string_lossy()
                .into_owned()
        }
    }

    fn hide(&mut self) {
        self.set_visible(false);
        self.emit.visible_changed();
    }

    fn exit(&mut self) {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        if let Err(e) = kill(Pid::this(), Signal::SIGINT) {
            error!("Failed to signal self to exit: {}", e);
        }
    }
}
