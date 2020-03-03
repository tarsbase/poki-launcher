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
use crate::event::Event;
use anyhow::{anyhow, Error, Result};
use log::{debug, error, trace, warn};
use notify::{watcher, RecursiveMode, Watcher};
use serde_derive::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::default::Default;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub struct Apps {
    db: AppsDB,
    db_path: PathBuf,
    app_paths: Vec<String>,
    term_cmd: Option<String>,
}

impl Apps {
    pub fn init(config: &Config) -> Result<Self> {
        let db_path = config.data_dir.join("apps.db");
        let app_paths = match get_app_paths(&config.file_options.plugins) {
            Ok(app_paths) => app_paths,
            Err(_) => vec![
                "/usr/share/applications".into(),
                "~/.local/share/applications/".into(),
                "/var/lib/snapd/desktop/applications".into(),
                "/var/lib/flatpak/exports/share/applications".into(),
            ],
        };
        if app_paths.is_empty() {
            warn!("The list of search paths for apps is empty so none will be found");
        }
        let term_cmd = get_term_cmd(&config.file_options.plugins).ok();
        let db = if db_path.as_path().exists() {
            debug!("Loading db from: {}", db_path.display());
            AppsDB::load(&db_path)?
        } else {
            trace!("Creating new apps.db");
            trace!("{:#?}", app_paths);
            let (apps_db, errors) = AppsDB::from_desktop_entries(&app_paths);
            trace!("{:#?}", apps_db);
            crate::log_errs(&errors);
            // TODO visual error indicator
            if let Err(e) = apps_db.save(&db_path) {
                error!("{}", e);
            }
            apps_db
        };

        Ok(Apps {
            db,
            db_path,
            app_paths,
            term_cmd,
        })
    }
}

fn get_apps_config(plugins: &Value) -> Result<&Map<String, Value>> {
    Ok(plugins
        .as_object()
        .ok_or(anyhow!(""))?
        .get("apps")
        .ok_or(anyhow!(""))?
        .as_object()
        .ok_or(anyhow!(""))?)
}

fn get_app_paths(plugins: &Value) -> Result<Vec<String>> {
    Ok(get_apps_config(plugins)?
        .get("app_paths")
        .ok_or(anyhow!(""))?
        .as_array()
        .ok_or(anyhow!(""))?
        .into_iter()
        .filter_map(|item| item.as_str().map(|s| s.to_owned()))
        .collect())
}

fn get_term_cmd(plugins: &Value) -> Result<String> {
    Ok(get_apps_config(plugins)?
        .get("term_cmd")
        .ok_or(anyhow!(""))?
        .as_str()
        .ok_or(anyhow!(""))?
        .to_owned())
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

    fn run(&mut self, _: &Config, uuid: &str) -> Result<()> {
        let app = self
            .db
            .apps
            .iter()
            .find(|app| app.uuid == uuid)
            .unwrap()
            .clone();
        app.run(&self.term_cmd)?;
        self.db.update(&app);
        self.db.save(&self.db_path)?;
        Ok(())
    }

    fn reload(&mut self, _: &Config) -> Result<()> {
        let errors = self.db.rescan_desktop_entries(&self.app_paths);
        crate::log_errs(&errors);
        self.db.save(&self.db_path)?;
        Ok(())
    }

    fn register_event_handlers(
        &mut self,
        _config: &Config,
        event_tx: Sender<Event>,
    ) {
        let (tx, rx) = mpsc::channel();
        let mut watcher = match watcher(tx, Duration::from_secs(10)) {
            Ok(watcher) => watcher,
            Err(e) => {
                error!(
                    "{}",
                    Error::new(e).context("Error creating file system watcher")
                );
                return;
            }
        };
        for path in &self.app_paths {
            let expanded = match shellexpand::full(&path) {
                Ok(path) => path.into_owned(),
                Err(e) => {
                    error!(
                        "{}",
                        Error::new(e).context(format!(
                            "Error expanding desktop files dir path {}",
                            path
                        ))
                    );
                    continue;
                }
            };
            let path = Path::new(&expanded);
            if path.exists() {
                if let Err(e) = watcher.watch(path, RecursiveMode::Recursive) {
                    warn!(
                        "{}",
                        Error::new(e).context(format!(
                            "Error setting watcher for dir {}",
                            expanded
                        ))
                    );
                }
            }
        }
        std::mem::forget(watcher);
        thread::spawn(move || loop {
            match rx.recv() {
                Ok(event) => {
                    debug!("Desktop file watcher received {:?}", event);
                    if let Err(e) = event_tx.send(Event::Reload) {
                        error!("Error sending event to ui: {}", e);
                    }
                }
                Err(e) => {
                    error!("Desktop file watcher error {}", e);
                    return;
                }
            }
        });
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
