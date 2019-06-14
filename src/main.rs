use fuzzy_matcher::skim::fuzzy_match;
use launcher::scan::*;
use launcher::{self, App};
use rmp_serde as rmp;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use gdk::enums::key;
use glib::signal::Inhibit;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Entry, EntryExt, WidgetExt};


const DB_PATH: &'static str = "apps.db";

fn main() {
    let application = Application::new("com.github.gtk-rs.examples.basic", Default::default())
        .expect("failed to initialize GTK application");

    application.connect_activate(|app| {

        let db_path = Path::new(&DB_PATH);
        let apps: Vec<App> = if db_path.exists() {
            let mut apps_file = File::open(&db_path).unwrap();
            let mut buf = Vec::new();
            apps_file.read_to_end(&mut buf).unwrap();
            let mut de = rmp::Deserializer::new(&buf[..]);
            Deserialize::deserialize(&mut de).unwrap()
        } else {
            let desktop_files = desktop_files();
            let desktop_files = desktop_files.unwrap();
            let (apps, _errs) = parse_parse_entries(desktop_files);
            let mut buf = Vec::new();
            apps.serialize(&mut rmp::Serializer::new(&mut buf)).unwrap();
            let mut file = File::create("apps.db").unwrap();
            file.write_all(&buf).unwrap();
            apps
        };

        let window = ApplicationWindow::new(app);
        window.set_title("First GTK+ Program");
        window.set_default_size(350, 70);

        let entry = Entry::new();
        entry.connect_changed(move |entry| {
            dbg!(&entry);
            if let Some(text) = entry.get_text() {
                let apps = apps.clone();
                let mut app_list = apps
                    .iter()
                    .filter_map(|app| match fuzzy_match(&app.name, &text) {
                        Some(score) if score > 0 => Some((app, score)),
                        _ => None,
                    })
                    .collect::<Vec<(&App, i64)>>();
                app_list.sort_by(|left, right| right.1.cmp(&left.1));
                println!("--------------------------");
                for (app, score) in app_list {
                    println!("{}\t{}", app, score);
                }
            }
        });
        entry.connect_key_press_event(|entry, event| {
            if event.get_keyval() == key::Return {
                println!("Enter pressed!");
            }
            Inhibit(false)
        });
        window.add(&entry);

        window.show_all();
    });

    application.run(&[]);
}
