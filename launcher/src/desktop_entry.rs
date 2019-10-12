use super::App;
use failure::{Error, Fail};
use ini::Ini;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Fail)]
pub enum DesktopEntryParseError {
    #[fail(display = "Desktop file is missing 'Desktop Entry' section")]
    MissingSection,
    #[fail(display = "Desktop file is missing the 'Name' parameter")]
    MissingName,
    #[fail(display = "Desktop file is missing the 'Exec' parameter")]
    MissingExec,
    #[fail(display = "Desktop file is missing the 'Icon' parameter")]
    MissingIcon,
}

pub fn parse_desktop_file(path: impl AsRef<Path>) -> Result<Option<App>, Error> {
    // TODO Finish implementation
    let file = Ini::load_from_file(path)?;
    let entry = file
        .section(Some("Desktop Entry".to_owned()))
        .ok_or(DesktopEntryParseError::MissingSection)?;
    let name = entry
        .get("Name")
        .ok_or(DesktopEntryParseError::MissingName)?
        .clone();
    let exec = entry
        .get("Exec")
        .ok_or(DesktopEntryParseError::MissingExec)?
        .clone();
    let icon = entry
        .get("Icon")
        .ok_or(DesktopEntryParseError::MissingIcon)?
        .clone();
    let app = App::new(name, icon, exec);
    match entry.get("NoDisplay") {
        Some(text) => match text.parse() {
            Ok(no_display) => {
                if no_display {
                    Ok(None)
                } else {
                    Ok(Some(app))
                }
            }
            Err(e) => {
                eprintln!("NoDisplay has a invalid value: {}", text);
                Ok(Some(app))
            }
        },
        None => Ok(Some(app)),
    }
}
