/// Interact with the app database
pub mod db;
/// Parse desktop entries
pub mod desktop_entry;
/// Run an app
pub mod runner;
/// Scan for desktop entries
pub mod scan;

use self::db::AppsDB;
use super::ListItem;
use super::Plugin;
use crate::config::Config;
use anyhow::Result;
use log::{debug, error};
use serde_derive::{Deserialize, Serialize};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::default::Default;
use std::fmt;
use std::path::PathBuf;
use uuid::Uuid;

pub struct Apps {
    db: AppsDB,
    db_path: PathBuf,
}

impl Apps {
    pub fn init(config: &Config) -> Result<Self> {
        let db_path = config.data_dir.join("apps.db");
        let db = if db_path.as_path().exists() {
            debug!("Loading db from: {}", db_path.display());
            AppsDB::load(&db_path)?
        } else {
            let (apps_db, errors) = AppsDB::from_desktop_entries(&config);
            crate::log_errs(&errors);
            // TODO visual error indicator
            if let Err(e) = apps_db.save(&db_path) {
                error!("{}", e);
            }
            apps_db
        };
        Ok(Apps { db, db_path })
    }
}

impl Plugin for Apps {
    fn matcher(&self, _: &Config, _: &str) -> bool {
        true
    }

    fn search(
        &self,
        _config: &Config,
        input: &str,
        num_items: usize,
    ) -> Result<Vec<ListItem>> {
        Ok(self
            .db
            .get_ranked_list(input, Some(num_items))
            .into_iter()
            .map(ListItem::from)
            .collect())
    }

    fn run(&mut self, config: &Config, uuid: &str) -> Result<()> {
        let app = self
            .db
            .apps
            .iter()
            .find(|app| app.uuid == uuid)
            .unwrap()
            .clone();
        app.run(&config)?;
        self.db.update(&app);
        self.db.save(&self.db_path)?;
        Ok(())
    }

    fn reload(&mut self, config: &Config) -> Result<()> {
        let (app_list, errors) =
            self::scan::scan_desktop_entries(&config.file_options.app_paths);
        crate::log_errs(&errors);
        self.db.merge_new_entries(app_list);
        self.db.save(&self.db_path)?;
        Ok(())
    }
}

impl From<App> for ListItem {
    fn from(app: App) -> Self {
        Self {
            name: app.name,
            icon: app.icon,
            id: app.uuid,
        }
    }
}

/// An app on your machine.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct App {
    /// Display name of the app.
    pub name: String,
    /// The exec string used to run the app.
    pub(crate) exec: String,
    /// Score of the app of the ranking algo.
    score: f32,
    /// Uuid used to uniquely identify this app.
    /// This is saved to find the app later when the list changes.
    pub uuid: String,
    /// Icon name for this app.
    /// The icon name has to be looked up in the system's icon
    /// theme to get a file path.
    pub icon: String,
    /// If true, launch in terminal
    pub(crate) terminal: bool,
}

impl App {
    /// Create a new app.
    pub fn new(
        name: String,
        icon: String,
        exec: String,
        terminal: bool,
    ) -> App {
        App {
            name,
            icon,
            exec,
            uuid: Uuid::new_v4().to_string(),
            score: 0.0,
            terminal,
        }
    }

    /// Set this app's name, icon, and exec to the values of the other app.
    pub fn merge(&mut self, other: &App) {
        self.name = other.name.clone();
        self.icon = other.icon.clone();
        self.exec = other.exec.clone();
    }
}

impl PartialEq for App {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.exec == other.exec
            && self.icon == other.icon
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

impl fmt::Display for App {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
