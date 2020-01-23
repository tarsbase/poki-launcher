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
#[serde(default)]
pub struct Config {
    /// The list of directories to search for desktop entries in.
    pub app_paths: Vec<String>,
    /// Command to use to run terminal apps
    pub term_cmd: Option<String>,

    pub window_height: i32,
    pub window_width: i32,
    pub background_color: String,
    pub border_color: String,
    pub input_box_color: String,
    pub input_text_color: String,
    pub selected_app_color: String,
    pub app_text_color: String,
    pub app_separator_color: String,

    pub attempt_force_focus: bool,

    pub input_font_size: i32,
    pub app_font_size: i32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_paths: vec![
                "/usr/share/applications".into(),
                "~/.local/share/applications/".into(),
                "/var/lib/snapd/desktop/applications".into(),
                "/var/lib/flatpak/exports/share/applications".into(),
            ],
            term_cmd: None,

            window_height: 500,
            window_width: 500,

            background_color: "#282a36".into(),
            border_color: "#2e303b".into(),
            input_box_color: "#44475a".into(),
            input_text_color: "#f8f8f2".into(),
            selected_app_color: "#44475a".into(),
            app_text_color: "#f8f8f2".into(),
            app_separator_color: "#bd93f9".into(),

            attempt_force_focus: true,

            input_font_size: 13,
            app_font_size: 20,
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

        if file_path.as_path().exists() {
            cfg.merge(config::File::with_name(file_path.to_str().unwrap()))?;
            Ok(cfg.try_into()?)
        } else {
            Ok(Self::default())
        }
    }
}
