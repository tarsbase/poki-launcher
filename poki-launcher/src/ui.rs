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
use lib_poki_launcher::{event::Event, ListItem, PokiLauncher as Launcher};
use log::{debug, error, trace, warn};
use poki_launcher_notifier::{self as notifier, Notifier};
use qmetaobject::*;
use std::cell::{Cell, RefCell};
use std::convert::From;
use std::sync::{Arc, Mutex};
use std::thread;

const MAX_APPS_SHOWN: usize = 5;

lazy_static! {
    pub static ref LAUNCHER: Arc<Mutex<Launcher>> = {
        match Launcher::init() {
            Ok(launcher) => Arc::new(Mutex::new(launcher)),
            Err(e) => {
                error!("{:?}", e);
                std::process::exit(3);
            }
        }
    };
}

thread_local! {
    pub static SHOW_ON_START: Cell<bool> = Cell::new(true);
}

#[derive(QObject, Default)]
struct PokiLauncher {
    base: qt_base_class!(trait QObject),
    list: Vec<ListItem>,
    model: qt_property!(RefCell<SimpleListModel<QListItem>>; NOTIFY model_changed),
    selected: qt_property!(QString; NOTIFY selected_changed WRITE set_selected),
    visible: qt_property!(bool; NOTIFY visible_changed),
    loading: qt_property!(bool; NOTIFY loading_changed),
    has_moved: qt_property!(bool),

    window_height: qt_property!(i32; NOTIFY settings_changed),
    window_width: qt_property!(i32; NOTIFY settings_changed),

    background_color: qt_property!(QString; NOTIFY settings_changed),
    border_color: qt_property!(QString; NOTIFY settings_changed),
    input_box_color: qt_property!(QString; NOTIFY settings_changed),
    input_text_color: qt_property!(QString; NOTIFY settings_changed),
    selected_app_color: qt_property!(QString; NOTIFY settings_changed),
    app_text_color: qt_property!(QString; NOTIFY settings_changed),
    app_separator_color: qt_property!(QString; NOTIFY settings_changed),

    input_font_size: qt_property!(i32; NOTIFY settings_changed),
    app_font_size: qt_property!(i32; NOTIFY settings_changed),
    input_box_ratio: qt_property!(f32; NOTIFY settings_changed),

    init: qt_method!(fn(&mut self)),
    search: qt_method!(fn(&mut self, text: String)),
    load: qt_method!(fn(&mut self)),
    down: qt_method!(fn(&mut self)),
    up: qt_method!(fn(&mut self)),
    run: qt_method!(fn(&mut self)),
    hide: qt_method!(fn(&mut self)),
    exit: qt_method!(fn(&mut self)),

    selected_changed: qt_signal!(),
    visible_changed: qt_signal!(),
    loading_changed: qt_signal!(),
    model_changed: qt_signal!(),
    settings_changed: qt_signal!(),
}

impl PokiLauncher {
    fn init(&mut self) {
        self.load();

        let mut launcher = LAUNCHER.lock().expect("Mutex poisoned");
        self.window_height = launcher.config.file_options.window_height;
        self.window_width = launcher.config.file_options.window_width;

        self.background_color =
            prepend_hash(launcher.config.file_options.background_color.clone())
                .into();
        self.border_color =
            prepend_hash(launcher.config.file_options.border_color.clone())
                .into();
        self.input_box_color =
            prepend_hash(launcher.config.file_options.input_box_color.clone())
                .into();
        self.input_text_color =
            prepend_hash(launcher.config.file_options.input_text_color.clone())
                .into();
        self.selected_app_color = prepend_hash(
            launcher.config.file_options.selected_app_color.clone(),
        )
        .into();
        self.app_text_color =
            prepend_hash(launcher.config.file_options.app_text_color.clone())
                .into();
        self.app_separator_color = prepend_hash(
            launcher.config.file_options.app_separator_color.clone(),
        )
        .into();

        self.input_font_size = launcher.config.file_options.input_font_size;
        self.app_font_size = launcher.config.file_options.app_font_size;
        self.input_box_ratio = launcher.config.file_options.input_box_ratio;

        self.settings_changed();

        // Setup signal notifier and callback
        self.visible = SHOW_ON_START.with(|b| b.get());
        self.visible_changed();
        let rx = match Notifier::start() {
            Ok(rx) => rx,
            Err(e) => {
                error!("{:?}", e.context("Error starting signal notifier"));
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
                    }
                    Msg::Exit => {
                        drop(rx);
                        std::process::exit(0);
                    }
                },
                Err(e) => {
                    warn!(
                        "{}",
                        Error::new(e).context("Error with signal notifier")
                    );
                }
            }
        });

        // Setup desktop file change notifier and callback
        let qptr = QPointer::from(&*self);
        let reload = qmetaobject::queued_callback(move |()| {
            qptr.as_pinned().map(|self_| {
                self_.borrow_mut().load();
            });
        });
        let event_rx = launcher.register_event_handlers();
        thread::spawn(move || loop {
            match event_rx.recv() {
                Ok(event) => {
                    debug!("Received event {:?}", event);
                    match event {
                        Event::Reload => reload(()),
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
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
        self.list = match LAUNCHER
            .lock()
            .expect("Launcher Mutex Poisoned")
            .search(&text, MAX_APPS_SHOWN)
        {
            Ok(list) => list,
            Err(e) => {
                error!("{:?}", e);
                return;
            }
        };
        if !self.has_moved
            || !self.list.iter().any(|item| item.id == self.get_selected())
        {
            if !self.list.is_empty() {
                self.set_selected(self.list[0].id.clone());
            } else {
                self.set_selected(QString::default());
            }
        }
        self.model.borrow_mut().reset_data(
            self.list.clone().into_iter().map(QListItem::from).collect(),
        );
    }

    fn load(&mut self) {
        trace!("Loading...");
        self.loading = true;
        self.loading_changed();
        let qptr = QPointer::from(&*self);
        let done = qmetaobject::queued_callback(move |()| {
            qptr.as_pinned().map(|self_| {
                self_.borrow_mut().loading = false;
                self_.borrow().loading_changed();
            });
        });
        thread::spawn(move || {
            let mut launcher =
                LAUNCHER.lock().expect("Launcher Mutex Poisoned");
            if let Err(e) = launcher.reload() {
                error!("{:?}", e);
            }
            done(());
            trace!("Loading...done");
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
            .find(|(_, item)| item.id == self.get_selected())
            .unwrap();
        if idx >= self.list.len() - 1 {
            self.set_selected(self.list[self.list.len() - 1].id.clone());
        } else {
            self.set_selected(self.list[idx + 1].id.clone());
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
            .find(|(_, item)| item.id == self.get_selected())
            .unwrap();
        if idx == 0 {
            self.set_selected(self.list[0].id.clone());
        } else {
            self.set_selected(self.list[idx - 1].id.clone());
        }
    }

    fn run(&mut self) {
        trace!("Run");
        if self.list.is_empty() {
            return;
        }
        self.has_moved = false;
        let item = self
            .list
            .iter()
            .find(|item| item.id == self.get_selected())
            .unwrap();
        let mut launcher = LAUNCHER.lock().expect("Launcher Mutex Poisoned");

        if let Err(err) = launcher.run(&item.id) {
            error!("{:?}", err);
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
            error!(
                "{:?}",
                Error::new(e).context("Error signaling self to exit")
            );
        }
    }
}

#[derive(Default, Clone, SimpleListItem)]
struct QListItem {
    pub name: String,
    pub id: String,
    pub icon: String,
}

impl From<ListItem> for QListItem {
    fn from(item: ListItem) -> QListItem {
        QListItem {
            name: item.name,
            id: item.id,
            icon: item.icon,
        }
    }
}

fn prepend_hash(mut s: String) -> String {
    match s.chars().nth(0) {
        Some(c) if c != '#' => {
            s.insert(0, '#');
            s
        }
        _ => s,
    }
}

impl QMetaType for QListItem {}

qrc!(init_qml_resources,
    "ui" {
        "ui/main.qml" as "main.qml",
        "ui/MainForm.ui.qml" as "MainForm.ui.qml",
    }
);

pub fn init_ui() {
    init_qml_resources();
    qml_register_type::<PokiLauncher>(
        cstr!("PokiLauncher"),
        1,
        0,
        cstr!("PokiLauncher"),
    );
}
