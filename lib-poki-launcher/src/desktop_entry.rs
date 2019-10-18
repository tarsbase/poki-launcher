use super::App;
use failure::{Error, Fail};
use ini::Ini;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Fail)]
pub enum DesktopEntryParseError {
    #[fail(display = "Desktop file {} is missing 'Desktop Entry' section", file)]
    MissingSection { file: String },
    #[fail(display = "Desktop file {} is missing the 'Name' parameter", file)]
    MissingName { file: String },
    #[fail(display = "Desktop file {} is missing the 'Exec' parameter", file)]
    MissingExec { file: String },
    #[fail(display = "Desktop file {} is missing the 'Icon' parameter", file)]
    MissingIcon { file: String },
    #[fail(display = "Failed to parse deskop file {}: {}", file, err)]
    InvalidIni { file: String, err: Error },
}

fn remove_if_true(item: Option<&String>, app: App) -> Option<App> {
    match item {
        Some(text) => match text.parse() {
            Ok(item) => {
                if item {
                    None
                } else {
                    Some(app)
                }
            }
            Err(_) => {
                eprintln!("NoDisplay has a invalid value: {}", text);
                Some(app)
            }
        },
        None => Some(app),
    }
}

fn strip_entry_args(exec: &str) -> String {
    let iter = exec.split(' ');
    iter.filter(|item| !item.starts_with('%')).collect()
}

pub fn parse_desktop_file(path: impl AsRef<Path>) -> Result<Option<App>, Error> {
    let path_str = path.as_ref().to_string_lossy().into_owned();
    // TODO Finish implementation
    let file = Ini::load_from_file(path).map_err(|e| DesktopEntryParseError::InvalidIni {
        file: path_str.clone(),
        err: e.into(),
    })?;
    let entry = file.section(Some("Desktop Entry".to_owned())).ok_or(
        DesktopEntryParseError::MissingSection {
            file: path_str.clone(),
        },
    )?;
    let name = entry
        .get("Name")
        .ok_or(DesktopEntryParseError::MissingName {
            file: path_str.clone(),
        })?
        .clone();
    let exec = entry
        .get("Exec")
        .ok_or(DesktopEntryParseError::MissingExec {
            file: path_str.clone(),
        })?
        .clone();
    let exec = strip_entry_args(&exec);
    let icon = entry
        .get("Icon")
        .ok_or(DesktopEntryParseError::MissingIcon {
            file: path_str.clone(),
        })?
        .clone();
    let app = App::new(name, icon, exec);
    Ok(remove_if_true(entry.get("NoDisplay"), app.clone())
        .or_else(|| remove_if_true(entry.get("Hidden"), app)))
}
