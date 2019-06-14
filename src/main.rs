
use launcher::desktop_entry::{parse_desktop_file, DesktopEntryParseError};
use launcher::scan::*;
use launcher::{self, App};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use rmp_serde as rmp;

const DB_PATH: &'static str = "apps.db";

fn main() {
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
        let (apps, errs) = parse_parse_entries(desktop_files);
        let mut buf = Vec::new();
        apps.serialize(&mut rmp::Serializer::new(&mut buf)).unwrap();
        let mut file = File::create("apps.db").unwrap();
        file.write_all(&buf).unwrap();
        apps
    };
    dbg!(&apps);
}
