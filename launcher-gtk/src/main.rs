use fuzzy_matcher::skim::fuzzy_match;
use lib_poki_launcher::scan::*;
use lib_poki_launcher::{self, db::AppsDB, App};

use rmp_serde as rmp;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::mem;
use std::path::Path;
use std::sync::{Arc, Mutex};

use gdk::enums::key;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use gio::prelude::*;
use glib::{self, signal::Inhibit};
use gtk::prelude::*;
use gtk::*;
use lazy_static::*;

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
    AppList(Vec<(App, usize)>),
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
                    let mut app_list = apps
                        .apps
                        .iter()
                        .enumerate()
                        .filter_map(|(i, app)| match fuzzy_match(&app.name, &text) {
                            Some(score) if score > 0 => Some((app.clone(), score, i)),
                            _ => None,
                        })
                        .collect::<Vec<(App, i64, usize)>>();
                    app_list.sort_by(|left, right| right.1.cmp(&left.1));
                    let ret_list: Vec<(App, usize)> =
                        app_list.into_iter().map(|(app, _, i)| (app, i)).collect();
                    if let Some(app) = ret_list.get(0) {
                        to_launch = Some(app.clone());
                    } else {
                        to_launch = None;
                    }
                    output_tx.send(OutMsg::AppList(ret_list)).unwrap();
                }
                InMsg::Run => {
                    if let Some((app, i)) = &to_launch {
                        // TODO Handle app run failures
                        app.run().unwrap();
                        apps.update_score(*i, 1.0);
                        let mut buf = Vec::new();
                        apps.serialize(&mut rmp::Serializer::new(&mut buf)).unwrap();
                        let mut file = File::create("apps.db").unwrap();
                        file.write_all(&buf).unwrap();
                    }
                    output_tx.send(OutMsg::Hide).unwrap();
                    break;
                }
                InMsg::Exit => {
                    break;
                }
            }
        }
    });
    *BG.lock().unwrap() = Some(bg_handle);

    let window = ApplicationWindow::new(application);
    let top_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let entry = Entry::new();
    let tree = TreeView::new();
    let column_types = [String::static_type()];
    let store = TreeStore::new(&column_types);
    let col = TreeViewColumn::new();
    let renderer = CellRendererText::new();
    col.pack_start(&renderer, true);
    col.add_attribute(&renderer, "text", 0);
    tree.append_column(&col);
    tree.set_model(Some(&store));
    tree.set_headers_visible(false);
    // Initalize with 5 empty entryies for spacing
    for _ in 0..5 {
        store.insert_with_values(None, None, &[0], &[&" ".to_owned()]);
    }

    window.set_title("Poki Launcher");
    window.set_default_size(350, 70);
    window.set_position(gtk::WindowPosition::Center);

    top_box.pack_start(&entry, true, true, 0);
    top_box.pack_end(&tree, true, true, 0);
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
        }
        Inhibit(false)
    });

    let exit_window = window.clone();
    output_rx.attach(None, move |msg| {
        match msg {
            OutMsg::AppList(apps) => {
                store.clear();
                let end = if apps.len() > MAX_APPS_SHOWN {
                    MAX_APPS_SHOWN
                } else {
                    apps.len()
                };
                // println!("--------------------------");
                for (app, _) in &apps[0..end] {
                    // println!("{}", app);
                    store.insert_with_values(None, None, &[0], &[&app.name]);
                }
                for _ in end..5 {
                    store.insert_with_values(None, None, &[0], &[&" ".to_owned()]);
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
    tree.set_sensitive(false);
}

// if let Some(app) = app_list.get(0) {
//     *to_launch.borrow_mut() = Some(app.0.clone());
// }
fn main() {
    let application = Application::new("info.bengoldberg.poki_launcher", Default::default())
        .expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        let db_path = Path::new(&DB_PATH);
        let apps: AppsDB = if db_path.exists() {
            let mut apps_file = File::open(&db_path).unwrap();
            let mut buf = Vec::new();
            apps_file.read_to_end(&mut buf).unwrap();
            let mut de = rmp::Deserializer::new(&buf[..]);
            Deserialize::deserialize(&mut de).unwrap()
        } else {
            let desktop_files = desktop_files();
            let desktop_files = desktop_files.unwrap();
            let (apps, _errs) = parse_parse_entries(desktop_files);
            let apps = AppsDB::new(apps);
            let mut buf = Vec::new();
            apps.serialize(&mut rmp::Serializer::new(&mut buf)).unwrap();
            let mut file = File::create("apps.db").unwrap();
            file.write_all(&buf).unwrap();
            apps
        };

        build_ui(app, apps);
    });

    application.run(&[]);
}
