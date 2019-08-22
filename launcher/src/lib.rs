pub mod db;
pub mod desktop_entry;
pub mod runner;
pub mod scan;

use db::AppsDB;
use failure::Error;
use fuzzy_matcher::skim::fuzzy_match;
use rmp_serde as rmp;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use uuid::prelude::*;

pub mod prelude {
    pub use crate::db::AppsDB;
    pub use crate::App;
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct App {
    pub name: String,
    exec: String,
    score: f32,
    pub uuid: String,
}

impl App {
    pub fn new(name: String, exec: String) -> App {
        App {
            name,
            exec,
            uuid: Uuid::new_v4().to_string(),
            score: 0.0,
        }
    }
}

impl AppsDB {
    pub fn load(path: impl AsRef<Path>) -> Result<AppsDB, Error> {
        let mut apps_file = File::open(&path)?;
        let mut buf = Vec::new();
        apps_file.read_to_end(&mut buf)?;
        let mut de = rmp::Deserializer::new(&buf[..]);
        Ok(Deserialize::deserialize(&mut de)?)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let mut buf = Vec::new();
        self.serialize(&mut rmp::Serializer::new(&mut buf))?;
        let mut file = File::create(&path)?;
        file.write_all(&buf)?;
        Ok(())
    }

    pub fn get_ranked_list(&self, search: &str, num_items: Option<usize>) -> Vec<App> {
        let mut app_list = self
            .apps
            .iter()
            .filter_map(|app| match fuzzy_match(&app.name, &search) {
                Some(score) if score > 0 => {
                    let mut app = app.clone();
                    app.score += score as f32;
                    Some(app)
                }
                _ => None,
            })
            .collect::<Vec<App>>();
        app_list.sort_by(|left, right| right.score.partial_cmp(&left.score).unwrap());
        if let Some(n) = num_items {
            app_list = app_list.into_iter().take(n).collect();
        }
        app_list
    }

    pub fn update(&mut self, to_update: &App) {
        self.update_score(&to_update.uuid, 1.0);
    }
}

impl fmt::Display for App {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
