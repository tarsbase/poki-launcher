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
/// Parse desktop entries
pub mod desktop_entry;
/// Run an app
pub mod runner;
/// Scan for desktop entries
pub mod scan;

use super::ListItem;
use super::Plugin;
use crate::config::Config;
use crate::event::Event;
use crate::frecency_db::*;
use anyhow::{anyhow, Error, Result};
use log::{debug, error, warn};
use notify::{watcher, RecursiveMode, Watcher};
use serde_derive::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::default::Default;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub struct Apps {
    db: Mutex<AppsDB>,
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
        let (db, errors) = AppsDB::from_desktop_entries(&db_path, &app_paths)?;
        crate::log_errs(&errors);

        Ok(Apps {
            db: Mutex::new(db),
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
        let db = self.db.lock().expect("Apps Mutex poisoned");
        Ok(db
            .get_ranked_list(input, Some(num_items))?
            .into_iter()
            .map(ListItem::from)
            .collect())
    }

    fn run(&mut self, _: &Config, id: u64) -> Result<()> {
        let cont = self
            .db
            .lock()
            .expect("Apps Mutex poisoned")
            .get_by_id(id)?
            .unwrap();
        cont.item.run(&self.term_cmd)?;
        Ok(())
    }

    fn reload(&mut self, _: &Config) -> Result<()> {
        let errors = self
            .db
            .lock()
            .expect("Apps Mutex poisoned")
            .rescan_desktop_entries(&self.app_paths)?;
        crate::log_errs(&errors);
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
                    "{:?}",
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
                        "{:?}",
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
                    debug!("Desktop file watcher received: {:?}", event);
                    if let Err(e) = event_tx.send(Event::Reload) {
                        error!("Error sending event to ui: {:?}", e);
                    }
                }
                Err(e) => {
                    error!("Desktop file watcher error: {:?}", e);
                    return;
                }
            }
        });
    }
}

/// An app on your machine.
#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq)]
pub struct App {
    /// Display name of the app.
    pub name: String,
    /// The exec string used to run the app.
    pub(crate) exec: String,
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

pub type AppsDB = FrecencyDB<App>;

impl Hash for App {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.name.hash(hasher);
        self.exec.hash(hasher);
        self.icon.hash(hasher);
    }
}

impl DBItem for App {
    fn get_sort_string(&self) -> &str {
        self.name.as_str()
    }
}

impl From<Container<App>> for ListItem {
    fn from(cont: Container<App>) -> Self {
        Self {
            name: cont.item.name.clone(),
            icon: cont.item.icon.clone(),
            id: cont.id,
        }
    }
}
