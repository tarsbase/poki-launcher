use failure::Error;

use std::fs::read_dir;
use std::path::PathBuf;
use crate::App;
use std::fs::File;
use std::io::Read;
use crate::desktop_entry::parse_desktop_file;

fn search_locations() -> Vec<&'static str> {
    vec!["/usr/share/applications"]
}

pub fn desktop_files() -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    for loc in search_locations() {
        for entry in read_dir(&loc)? {
            let entry = entry?;
            if entry.file_name().to_str().unwrap().contains(".desktop") {
                files.push(entry.path())
            }
        }
    }
    Ok(files)
}

pub fn parse_parse_entries(files: Vec<PathBuf>) -> (Vec<App>, Vec<Error>) {
    let (apps, errs): (Vec<_>, Vec<_>) = files
        .into_iter()
        .map(|path| parse_desktop_file(&path))
        .partition(Result::is_ok);
    let apps: Vec<_> = apps.into_iter().map(Result::unwrap).collect();
    let errs: Vec<_> = errs.into_iter().map(Result::unwrap_err).collect();
    (apps, errs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan() {
        let entries = desktop_files();
        #[allow(unused_must_use)]
        dbg!(entries);
    }
}