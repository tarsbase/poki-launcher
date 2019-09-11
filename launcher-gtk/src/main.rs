use lib_poki_launcher::prelude::*;

use std::mem;
use std::path::Path;
use std::sync::{Arc, Mutex};

use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use gdk::enums::key;
use gio::prelude::*;
use gio::ListStore;
use glib::{self, signal::Inhibit};
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, CssProvider, Entry, ListBox, StyleContext,
    STYLE_PROVIDER_PRIORITY_USER,
};
use lazy_static::lazy_static;
use row_data::RowData;

const DB_PATH: &'static str = "apps.db";
const MAX_APPS_SHOWN: usize = 5;
const CSS: &str = include_str!("app.css");

#[derive(Debug, Clone)]
enum InMsg {
    SearchText(String),
    Run,
    Exit,
}

#[derive(Debug, Clone)]
enum OutMsg {
    AppList(Vec<App>),
    Hide,
}

lazy_static! {
    static ref BG: Arc<Mutex<Option<JoinHandle<()>>>> = Arc::new(Mutex::new(None));
    // static ref SELECTED: Arc<Mutex<>> = Arc::new(Mutex::new(None));
}

fn build_ui(application: &gtk::Application, mut apps: AppsDB) {
    let (input_tx, input_rx): (mpsc::Sender<InMsg>, mpsc::Receiver<InMsg>) = mpsc::channel();
    let (output_tx, output_rx): (glib::Sender<OutMsg>, glib::Receiver<OutMsg>) =
        glib::MainContext::channel(glib::PRIORITY_HIGH);

    let bg_handle = thread::spawn(move || {
        let mut to_launch = None;
        loop {
            match input_rx.recv().unwrap() {
                InMsg::SearchText(text) => {
                    let app_list = apps.get_ranked_list(&text, None);
                    to_launch = app_list.get(0).map(|app| app.clone());
                    output_tx.send(OutMsg::AppList(app_list)).unwrap();
                }
                InMsg::Run => {
                    if let Some(app) = &to_launch {
                        // TODO Handle app run failures
                        app.run().unwrap();
                        apps.update(app);
                        apps.save(&DB_PATH).unwrap();
                    }
                    output_tx.send(OutMsg::Hide).unwrap();
                    break;
                }
                InMsg::Exit => {
                    output_tx.send(OutMsg::Hide).unwrap();
                    break;
                }
            }
        }
    });
    *BG.lock().unwrap() = Some(bg_handle);

    let window = ApplicationWindow::new(application);
    let top_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let entry = Entry::new();
    let listbox = ListBox::new();
    // let tree = TreeView::new();
    let types = [String::static_type()];
    let store = ListStore::new(RowData::static_type());
    // let model: ListModel = store.upcast();
    listbox.bind_model(Some(&store), |item| {
        let item = item
            .downcast_ref::<RowData>()
            .expect("Row data is of wrong type");
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        let label = gtk::Label::new(None);
        item.bind_property("name", &label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
        row.pack_end(&label, true, true, 0);
        row.upcast()
    });

    window.set_title("Poki Launcher");
    window.set_default_size(350, 70);
    window.set_position(gtk::WindowPosition::Center);

    top_box.pack_start(&entry, true, true, 0);
    top_box.pack_end(&listbox, true, true, 0);
    window.add(&top_box);

    let search_tx = input_tx.clone();
    entry.connect_changed(move |entry| {
        if let Some(text) = entry.get_text() {
            let text_str = text.as_str().to_owned();
            search_tx
                .send(InMsg::SearchText(text_str))
                .expect("Failed to send search text to other thread");
        }
    });
    let run_tx = input_tx.clone();
    let exit_tx = input_tx.clone();
    entry.connect_key_press_event(move |_entry, event| {
        if event.get_keyval() == key::Return {
            run_tx.send(InMsg::Run).unwrap();
        } else if event.get_keyval() == key::Escape {
            run_tx.send(InMsg::Exit).unwrap();
        }
        Inhibit(false)
    });

    let exit_window = window.clone();
    output_rx.attach(None, move |msg| {
        match msg {
            OutMsg::AppList(apps) => {
                store.remove_all();
                let end = if apps.len() > MAX_APPS_SHOWN {
                    MAX_APPS_SHOWN
                } else {
                    apps.len()
                };
                // println!("--------------------------");
                for app in &apps[0..end] {
                    // println!("{}", app);
                    store.append(&RowData::new(&app.name));
                }
                for _ in end..5 {
                    store.append(&RowData::new(" "))
                }
                if apps.len() > 0 {}
            }
            OutMsg::Hide => {
                let bg_handle = mem::replace(&mut *BG.lock().unwrap(), None).unwrap();
                bg_handle.join().unwrap();
                exit_window.destroy();
            }
        }
        glib::Continue(true)
    });

    window.connect_delete_event(move |_, _| {
        exit_tx.send(InMsg::Exit).unwrap();
        let bg_handle = mem::replace(&mut *BG.lock().unwrap(), None).unwrap();
        bg_handle.join().unwrap();
        // main_quit();
        Inhibit(false)
    });

    // entry.show();
    let screen = window.get_screen().unwrap();
    let style = CssProvider::new();
    let _ = CssProvider::load_from_data(&style, CSS.as_bytes());
    StyleContext::add_provider_for_screen(&screen, &style, STYLE_PROVIDER_PRIORITY_USER);

    window.show_all();
    window.present();
    window.set_keep_above(true);
    // entry.grab_default();
    // tree.set_sensitive(false);
}

// if let Some(app) = app_list.get(0) {
//     *to_launch.borrow_mut() = Some(app.0.clone());
// }
fn main() {
    let application = Application::new(Some("info.bengoldberg.poki_launcher"), Default::default())
        .expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        let db_path = Path::new(&DB_PATH);
        let apps = if db_path.exists() {
            AppsDB::load(&DB_PATH).unwrap()
        } else {
            let apps = AppsDB::from_desktop_entries().unwrap();
            apps.save(&DB_PATH).expect("Faile to write db to disk");
            apps
        };

        build_ui(app, apps);
    });

    application.run(&[]);
}

mod row_data {
    use super::*;

    use glib::subclass;
    use glib::subclass::prelude::*;
    use glib::translate::*;
    use glib::{glib_object_impl, glib_object_subclass, glib_object_wrapper, glib_wrapper};

    mod imp {
        use super::*;
        use std::cell::RefCell;

        pub struct RowData {
            name: RefCell<Option<String>>,
        }

        static PROPERTIES: [subclass::Property; 1] = [subclass::Property("name", |name| {
            glib::ParamSpec::string(
                name,
                "Name",
                "Name",
                None, // Default value
                glib::ParamFlags::READWRITE,
            )
        })];

        impl ObjectSubclass for RowData {
            const NAME: &'static str = "RowData";
            type ParentType = glib::Object;
            type Instance = subclass::simple::InstanceStruct<Self>;
            type Class = subclass::simple::ClassStruct<Self>;

            glib_object_subclass!();

            // Called exactly once before the first instantiation of an instance. This
            // sets up any type-specific things, in this specific case it installs the
            // properties so that GObject knows about their existence and they can be
            // used on instances of our type
            fn class_init(klass: &mut Self::Class) {
                klass.install_properties(&PROPERTIES);
            }

            // Called once at the very beginning of instantiation of each instance and
            // creates the data structure that contains all our state
            fn new() -> Self {
                Self {
                    name: RefCell::new(None),
                }
            }
        }

        impl ObjectImpl for RowData {
            glib_object_impl!();

            fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
                let prop = &PROPERTIES[id];

                match *prop {
                    subclass::Property("name", ..) => {
                        let name = value.get();
                        self.name.replace(name);
                    }
                    _ => unimplemented!(),
                }
            }

            fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
                let prop = &PROPERTIES[id];

                match *prop {
                    subclass::Property("name", ..) => Ok(self.name.borrow().to_value()),
                    _ => unimplemented!(),
                }
            }
        }
    }

    glib_wrapper! {
        pub struct RowData(Object<subclass::simple::InstanceStruct<imp::RowData>,
            subclass::simple::ClassStruct<imp::RowData>, RowDataClass>);

        match fn {
            get_type => || imp::RowData::get_type().to_glib(),
        }
    }

    impl RowData {
        pub fn new(name: &str) -> RowData {
            glib::Object::new(Self::static_type(), &[("name", &name)])
                .expect("Failed to create row data")
                .downcast()
                .expect("Created row data is of wrong type")
        }
    }
}
