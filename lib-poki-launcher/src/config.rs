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
use crate::DIRS;
use anyhow::Error;
use serde_derive::{Deserialize, Serialize};
use std::default::Default;
use std::fs::create_dir;

/// User settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The list of directories to search for desktop entries in.
    pub app_paths: Vec<String>,
    /// Command to use to run terminal apps
    pub term_cmd: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_paths: vec!["/usr/share/applications".to_owned()],
            term_cmd: None,
        }
    }
}

impl Config {
    /// Load the app config.
    pub fn load() -> Result<Config, Error> {
        let mut cfg = config::Config::default();
        let config_dir = DIRS.config_dir();
        let mut file_path = None;
        if !config_dir.exists() {
            create_dir(&config_dir)?;
        }
        for entry in config_dir.read_dir()? {
            if let Ok(entry) = entry {
                if entry
                    .file_name()
                    .into_string()
                    .unwrap()
                    .starts_with("poki-launcher")
                {
                    file_path = Some(entry.path());
                }
            }
        }
        let file_path = match file_path {
            Some(p) => p,
            None => {
                return Ok(Self::default());
            }
        };
        cfg.merge(config::File::with_name(file_path.to_str().unwrap()))?;
        Ok(cfg.try_into()?)
    }
}
