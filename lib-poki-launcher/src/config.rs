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
use directories::ProjectDirs;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::default::Default;
use std::fs::create_dir;
use std::path::PathBuf;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub file_options: FileOptions,
    pub data_dir: PathBuf,
}

/// User settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FileOptions {
    pub window_height: i32,
    pub window_width: i32,
    pub background_color: String,
    pub border_color: String,
    pub input_box_color: String,
    pub input_text_color: String,
    pub selected_app_color: String,
    pub app_text_color: String,
    pub app_separator_color: String,

    pub input_font_size: i32,
    pub app_font_size: i32,
    pub input_box_ratio: f32,

    pub plugin_load_order: Vec<String>,
    pub plugins: Value,
}

impl Default for FileOptions {
    fn default() -> Self {
        FileOptions {
            // app_paths: vec![
            //     "/usr/share/applications".into(),
            //     "~/.local/share/applications/".into(),
            //     "/var/lib/snapd/desktop/applications".into(),
            //     "/var/lib/flatpak/exports/share/applications".into(),
            // ],
            // term_cmd: None,
            window_height: 500,
            window_width: 500,

            background_color: "#282a36".into(),
            border_color: "#2e303b".into(),
            input_box_color: "#44475a".into(),
            input_text_color: "#f8f8f2".into(),
            selected_app_color: "#44475a".into(),
            app_text_color: "#f8f8f2".into(),
            app_separator_color: "#bd93f9".into(),

            input_font_size: 13,
            app_font_size: 20,
            input_box_ratio: 0.1,

            plugin_load_order: vec!["apps".into()],
            plugins: json!({
                "apps": {
                    "app_paths": [
                        "/usr/share/applications",
                        "~/.local/share/applications/",
                        "/var/lib/snapd/desktop/applications",
                        "/var/lib/flatpak/exports/share/applications"
                    ]
                }
            }),
        }
    }
}

impl Config {
    /// Load the app config.
    pub fn load() -> Result<Config, Error> {
        let mut cfg = config::Config::default();
        let config_dir = DIRS.config_dir();
        let file_path = config_dir.join("poki-launcher.hjson");
        if !config_dir.exists() {
            create_dir(&config_dir)?;
        }

        let file_options = if file_path.as_path().exists() {
            cfg.merge(config::File::with_name(file_path.to_str().unwrap()))?;
            cfg.try_into()?
        } else {
            FileOptions::default()
        };

        let dirs =
            ProjectDirs::from("info", "Ben Goldberg", "Poki-Launcher").unwrap();

        Ok(Config {
            file_options,
            data_dir: dirs.data_dir().to_owned(),
        })
    }
}
