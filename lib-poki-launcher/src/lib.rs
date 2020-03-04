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
/// Application configuration
pub mod config;

pub mod event;
mod frecency_db;
mod plugins;

use self::config::Config;
use self::event::Event;
use self::plugins::Plugin;
use anyhow::{anyhow, Error, Result};
use directories::{BaseDirs, ProjectDirs};
use lazy_static::lazy_static;
use log::{debug, error};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

/// Things that you'll probably need in include when using this lib
pub mod prelude {
    pub use crate::config::Config;
    // pub use crate::db::AppsDB;
    // pub use crate::scan::*;
    // pub use crate::App;
    // pub use crate::DIRS;
}

lazy_static! {
    pub static ref DIRS: ProjectDirs =
        ProjectDirs::from("info", "Ben Goldberg", "Poki-Launcher").unwrap();
    pub static ref HOME_PATH: PathBuf =
        BaseDirs::new().unwrap().home_dir().to_owned();
}

pub struct PokiLauncher {
    pub config: Config,
    plugins: Vec<Box<dyn Plugin>>,
    selected_plugin: Option<usize>,
}

impl PokiLauncher {
    pub fn init() -> Result<PokiLauncher> {
        let config = Config::load()?;
        let plugins = self::plugins::init_plugins(&config);
        Ok(PokiLauncher {
            config,
            plugins,
            selected_plugin: None,
        })
    }

    pub fn search(
        &mut self,
        input: &str,
        num_items: usize,
    ) -> Result<Vec<ListItem>> {
        for (i, plugin) in self.plugins.iter().enumerate() {
            if plugin.matcher(&self.config, &input) {
                self.selected_plugin = Some(i);
                debug!("Selecting plugin {}", i);
                return plugin.search(&self.config, &input, num_items);
            }
        }
        Ok(vec![])
    }

    pub fn run(&mut self, id: &str) -> Result<()> {
        let selected = self.selected_plugin.take();
        match selected {
            Some(selected) => self.plugins[selected].run(&self.config, &id),
            None => Err(anyhow!("No app selected")),
        }
    }

    pub fn reload(&mut self) -> Result<()> {
        for plugin in &mut self.plugins {
            if let Err(e) = plugin.reload(&self.config) {
                error!("{:?}", e);
            }
        }
        Ok(())
    }

    pub fn register_event_handlers(&mut self) -> Receiver<Event> {
        let (event_tx, event_rx) = mpsc::channel();
        for plugin in &mut self.plugins {
            let tx = event_tx.clone();
            plugin.register_event_handlers(&self.config, tx);
        }
        std::mem::forget(event_tx);
        event_rx
    }
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub name: String,
    pub icon: String,
    pub id: String,
}

pub fn log_errs(errs: &[Error]) {
    for err in errs {
        error!("{:?}", err);
    }
}
