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
pub mod config;
pub mod db;
pub mod desktop_entry;
pub mod runner;
pub mod scan;

use db::AppsDB;
use directories::{BaseDirs, ProjectDirs};
use failure::{Error, Fail};
use fuzzy_matcher::skim::fuzzy_match;
use lazy_static::lazy_static;
use rmp_serde as rmp;
use serde_derive::{Deserialize, Serialize};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::fmt;
use std::fs::File;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use uuid::prelude::*;

pub mod prelude {
    pub use crate::config::Config;
    pub use crate::db::AppsDB;
    pub use crate::scan::*;
    pub use crate::App;
    pub use crate::DIRS;
}

lazy_static! {
    pub static ref DIRS: ProjectDirs =
        ProjectDirs::from("info", "Ben Goldberg", "Poki-Launcher").unwrap();
    pub static ref HOME_PATH: PathBuf = BaseDirs::new().unwrap().home_dir().to_owned();
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct App {
    pub name: String,
    exec: String,
    score: f32,
    pub uuid: String,
    pub icon: String,
}

impl App {
    pub fn new(name: String, icon: String, exec: String) -> App {
        App {
            name,
            icon,
            exec,
            uuid: Uuid::new_v4().to_string(),
            score: 0.0,
        }
    }

    pub fn merge(&mut self, other: &App) {
        self.name = other.name.clone();
        self.icon = other.icon.clone();
        self.exec = other.exec.clone();
    }
}

impl PartialEq for App {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.exec == other.exec && self.icon == other.icon
    }
}

impl Eq for App {}

impl Ord for App {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name
            .cmp(&other.name)
            .then(self.exec.cmp(&other.exec))
            .then(self.icon.cmp(&other.icon))
    }
}

impl PartialOrd for App {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl AppsDB {
    pub fn load(path: impl AsRef<Path>) -> Result<AppsDB, Error> {
        let path_str = path.as_ref().to_string_lossy().into_owned();
        Ok(
            rmp::from_read(File::open(&path).map_err(|e| AppDBError::FileOpen {
                file: path_str.clone(),
                err: e.into(),
            })?)
            .map_err(|e| AppDBError::ParseDB {
                file: path_str.clone(),
                err: e.into(),
            })?,
        )
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let path_str = path.as_ref().to_string_lossy().into_owned();
        let buf = rmp::to_vec(&self).expect("Failed to encode apps db");
        let mut file = File::create(&path).map_err(|e| AppDBError::FileCreate {
            file: path_str.clone(),
            err: e.into(),
        })?;
        file.write_all(&buf).map_err(|e| AppDBError::FileWrite {
            file: path_str.clone(),
            err: e.into(),
        })?;
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

#[derive(Debug, Fail)]
pub enum AppDBError {
    #[fail(display = "Failed to open apps database file {}: {}", file, err)]
    FileOpen { file: String, err: Error },
    #[fail(display = "Failed to create apps database file {}: {}", file, err)]
    FileCreate { file: String, err: Error },
    #[fail(display = "Failed to write to apps database file {}: {}", file, err)]
    FileWrite { file: String, err: Error },
    #[fail(display = "Couldn't parse apps database file {}: {}", file, err)]
    ParseDB { file: String, err: Error },
}
