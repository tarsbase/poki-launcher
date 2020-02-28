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
use super::App;
use anyhow::{Context as _, Result};
use freedesktop_entry_parser::*;
use itertools::Itertools as _;
use std::fs::File;
use std::io::Read as _;
use std::path::Path;
use std::str::from_utf8;
use thiserror::Error;

/// Error from paring a desktop entry
#[derive(Debug, Error)]
pub enum EntryParseError {
    /// Desktop file is missing the 'Desktop Entry' section.
    #[error("Desktop file {file} is missing 'Desktop Entry' section")]
    MissingSection { file: String },
    /// Desktop file is missing the 'Name' parameter.
    #[error("Desktop file {file} is missing the 'Name' parameter")]
    MissingName { file: String },
    /// Desktop file is missing the 'Exec' parameter.
    #[error("Desktop file {file} is missing the 'Exec' parameter")]
    MissingExec { file: String },
    /// Desktop file is missing the 'Icon' parameter.
    #[error("Desktop file {file} is missing the 'Icon' parameter")]
    MissingIcon { file: String },
    #[error("In entry {file} property {name} has an invalid value {value}")]
    /// A property had an invalid value.
    /// This is returned if NoDisplay or Hidden are set ti a value that isn't
    /// `true` or `false`.
    InvalidPropVal {
        file: String,
        name: String,
        value: String,
    },
}

fn prop_is_true(item: Option<&str>) -> Result<bool> {
    match item {
        Some(text) => Ok(text.parse()?),
        None => Ok(false),
    }
}

fn strip_entry_args(exec: &str) -> String {
    let iter = exec.split(' ');
    iter.filter(|item| !item.starts_with('%')).join(" ")
}

impl App {
    /// Parse a desktop entry
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the desktop entry
    ///
    /// # Return
    ///
    /// Returns `Ok(None)` if the app should not be listed.
    ///
    /// # Example
    ///
    /// Parse a list of desktop entries, separating successes from failures,
    /// then removing apps that shouldn't be displayed (None) from the successes.
    /// ```no_run
    /// use lib_poki_launcher::App;
    /// use std::path::Path;
    ///
    /// let entries = vec![Path::new("./firefox.desktop"), Path::new("./chrome.desktop")];
    /// let (apps, errors): (Vec<_>, Vec<_>) = entries
    ///     .into_iter()
    ///     .map(|path| App::parse_desktop_file(&path))
    ///     .partition(Result::is_ok);
    /// let mut apps: Vec<_> = apps
    ///     .into_iter()
    ///     .map(Result::unwrap)
    ///     .filter_map(|x| x)
    ///     .collect();
    /// ```
    pub fn parse_desktop_file(path: impl AsRef<Path>) -> Result<Option<Self>> {
        let path_str = path.as_ref().display().to_string();
        // TODO Finish implementation
        let mut file = File::open(path).with_context(|| {
            format!("Error opening desktop file {}", path_str)
        })?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).with_context(|| {
            format!("Error reading desktop file {}", path_str)
        })?;
        let section = parse_entry(&buf)
            .with_context(|| {
                format!("Error parsing desktop file {}", path_str)
            })?
            .find(|section| {
                if let Ok(section) = section {
                    section.title == b"Desktop Entry"
                } else {
                    false
                }
            })
            .ok_or(EntryParseError::MissingSection {
                file: path_str.clone(),
            })?
            .with_context(|| {
                format!("Error parsing desktop file {}", path_str)
            })?;

        let mut name = None;
        let mut exec = None;
        let mut no_display = None;
        let mut hidden = None;
        let mut icon = None;
        let mut terminal = None;

        for attr in section.attrs {
            match attr.name {
                b"Name" => name = Some(from_utf8(attr.value)?),
                b"Exec" => exec = Some(from_utf8(attr.value)?),
                b"NoDisplay" => no_display = Some(from_utf8(attr.value)?),
                b"Hidden" => hidden = Some(from_utf8(attr.value)?),
                b"Icon" => icon = Some(from_utf8(attr.value)?),
                b"Terminal" => terminal = Some(from_utf8(attr.value)?),
                _ => {}
            }
        }

        if prop_is_true(no_display).map_err(|_| {
            EntryParseError::InvalidPropVal {
                file: path_str.to_owned(),
                name: "NoDisplay".into(),
                value: no_display.unwrap().to_owned(),
            }
        })? || prop_is_true(hidden).map_err(|_| {
            EntryParseError::InvalidPropVal {
                file: path_str.to_owned(),
                name: "Hidden".into(),
                value: hidden.unwrap().to_owned(),
            }
        })? {
            return Ok(None);
        }

        let name = name.ok_or(EntryParseError::MissingName {
            file: path_str.to_owned(),
        })?;
        let exec = exec.ok_or(EntryParseError::MissingExec {
            file: path_str.to_owned(),
        })?;
        let exec = strip_entry_args(exec);
        let icon = match icon {
            Some(icon) => icon.to_owned(),
            None => String::new(),
        };
        let terminal = {
            if let Some(value) = terminal {
                value.parse().map_err(|_| EntryParseError::InvalidPropVal {
                    file: path_str.to_owned(),
                    name: "Terminal".into(),
                    value: value.to_owned(),
                })?
            } else {
                false
            }
        };

        Ok(Some(App::new(name.to_owned(), icon, exec, terminal)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod strip_entry_args {
        use super::*;

        #[test]
        fn no_args() {
            let exec = "/usr/bin/cat --flag".to_owned();
            assert_eq!(strip_entry_args(&exec), exec);
        }

        #[test]
        fn has_args() {
            let exec = "/usr/bin/cat %f --flag";
            let exec_no_args = "/usr/bin/cat --flag".to_owned();
            assert_eq!(strip_entry_args(&exec), exec_no_args);
        }
    }

    mod parse_desktop_file {
        #[test]
        fn vaild_file_exist() {
            use crate::App;
            use std::fs::{remove_file, File};
            use std::io::prelude::*;
            use std::path::Path;

            let path = Path::new("./test.desktop");
            let mut file = File::create(&path).unwrap();
            file.write_all(
                b"[Desktop Entry]
 Name=Test
 Icon=testicon
 Exec=/usr/bin/test --with-flag %f",
            )
            .unwrap();
            let app = App::parse_desktop_file(&path).unwrap().unwrap();
            let other_app = App::new(
                "Test".to_owned(),
                "testicon".to_owned(),
                "/usr/bin/test --with-flag".to_owned(),
                false,
            );
            // Note, apps will have different uuids but Eq doesn't consider them
            assert_eq!(app, other_app);
            remove_file(&path).unwrap();
        }
    }
}
