use ini::Ini;
use failure::{Error, Fail};
use super::App;

#[allow(dead_code)]
#[derive(Debug, Fail)]
pub (super) enum DesktopEntryParseError {
    #[fail(display = "Desktop file is missing 'Desktop Entry' section")]
    MissingSection,
}

#[allow(dead_code)]
pub (super) fn parse_desktop_file(path: &str) -> Result<App, Error> {
    // TODO Finish implementation
    let file = Ini::load_from_file(path)?;
    let entry = file.section(Some("Desktop Entry".to_owned())).ok_or(DesktopEntryParseError::MissingSection)?;
    Ok(App {
        name: entry.get("Name").unwrap().clone(),
        exec: entry.get("Exec").unwrap().clone(),
        ..Default::default()
    })
}