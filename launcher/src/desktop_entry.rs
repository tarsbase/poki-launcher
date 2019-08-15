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
}

#[allow(dead_code)]
pub fn parse_desktop_file(path: impl AsRef<Path>) -> Result<App, Error> {
    // TODO Finish implementation
    let file = Ini::load_from_file(path)?;
    let entry = file
        .section(Some("Desktop Entry".to_owned()))
        .ok_or(DesktopEntryParseError::MissingSection)?;
    let mut app = App {
        name: entry
            .get("Name")
            .ok_or(DesktopEntryParseError::MissingName)?
            .clone(),
        exec: entry
            .get("Exec")
            .ok_or(DesktopEntryParseError::MissingExec)?
            .clone(),
        ..Default::default()
    };
    app.strip_args();
    Ok(app)
}
