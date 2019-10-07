use failure::Error;

use crate::db::AppsDB;
use crate::desktop_entry::parse_desktop_file;
use crate::App;
use std::fs::read_dir;
use std::path::PathBuf;

pub fn desktop_entires(paths: &Vec<String>) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    for loc in paths {
        // TODO Print a nice error
        for entry in read_dir(&loc)? {
            let entry = entry?;
            if entry.file_name().to_str().unwrap().contains(".desktop") {
                files.push(entry.path())
            }
        }
    }
    Ok(files)
}

fn scan_desktop_entries(paths: &Vec<String>) -> Result<Vec<App>, Error> {
    let entries = desktop_entires(&paths)?;
    let (apps, errs): (Vec<_>, Vec<_>) = entries
        .into_iter()
        .map(|path| parse_desktop_file(&path))
        .partition(Result::is_ok);
    let apps: Vec<_> = apps.into_iter().map(Result::unwrap).collect();
    // TODO Don't ignore errors
    let _errs: Vec<_> = errs.into_iter().map(Result::unwrap_err).collect();
    Ok(apps)
}

impl AppsDB {
    pub fn from_desktop_entries(paths: &Vec<String>) -> Result<AppsDB, Error> {
        Ok(AppsDB::new(scan_desktop_entries(paths)?))
    }

    pub fn rescan_desktop_entries(&mut self, paths: &Vec<String>) -> Result<(), Error> {
        self.merge(&scan_desktop_entries(paths)?);
        Ok(())
    }
}
