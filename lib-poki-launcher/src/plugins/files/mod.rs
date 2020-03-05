use super::ListItem;
use super::Plugin;
use crate::config::Config;
use crate::frecency_db::*;
use crate::run::run_bg;
use anyhow::Context as _;
use anyhow::{Error, Result};
use log::{debug, error, trace};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;
use walkdir::{DirEntry, WalkDir};

pub struct Files {
    db: FilesDB,
    db_path: PathBuf,
}

impl Files {
    pub fn init(config: &Config) -> Result<Self> {
        let db_path = config.data_dir.join("files.db");

        let db = if db_path.as_path().exists() {
            debug!("Loading db from: {}", db_path.display());
            FilesDB::load(&db_path)?
        } else {
            trace!("Creating new files.db");
            let files_db = FilesDB::new(vec![]);
            // TODO visual error indicator
            if let Err(e) = files_db.save(&db_path) {
                error!("{:?}", e);
            }
            files_db
        };
        Ok(Files { db, db_path })
    }
}

impl Plugin for Files {
    fn matcher(&self, _config: &Config, input: &str) -> bool {
        match input.get(0..1) {
            Some(":") => true,
            _ => false,
        }
    }
    fn search(
        &self,
        _: &Config,
        input: &str,
        num_items: usize,
    ) -> Result<Vec<crate::ListItem>> {
        trace!("Files search");
        let list: Vec<_> = self
            .db
            .get_ranked_list(input, Some(num_items))
            .into_iter()
            .map(ListItem::from)
            .collect();
        Ok(list)
    }
    fn run(&mut self, _config: &Config, id: &str) -> Result<()> {
        let item = self.db.iter().find(|item| item.uuid == id).unwrap().clone();
        item.item.open()?;
        self.db.update(&item);
        self.db.save(&self.db_path)?;
        Ok(())
    }
    fn reload(&mut self, config: &Config) -> Result<()> {
        let (entries, errors): (Vec<_>, Vec<_>) =
            WalkDir::new(&config.user_home)
                .into_iter()
                .filter_entry(|e| !is_hidden(e))
                .partition(Result::is_ok);
        let errors: Vec<_> = errors
            .into_iter()
            .map(Result::unwrap_err)
            .map(|e| Error::new(e).context("Error indexing files"))
            .collect();
        crate::log_errs(&errors);
        let files: Vec<_> = entries
            .into_iter()
            .map(Result::unwrap)
            .map(|e| File {
                name: (*e.file_name().to_string_lossy()).to_owned(),
                path: e.path().to_owned(),
            })
            .collect();

        debug!("Found {} files", files.len());
        // debug!("{:#?}", files);
        self.db.merge_new_entries(files);
        trace!("Saving");
        self.db.save(&self.db_path)?;
        Ok(())
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
struct File {
    name: String,
    path: PathBuf,
}

impl File {
    pub fn open(&self) -> Result<()> {
        let mut command = Command::new("xdg-open");
        command.arg(self.path.as_path().as_os_str());
        let _ = run_bg(command).with_context(|| {
            format!("Error opening file {}", self.path.display())
        })?;
        Ok(())
    }
}

impl DBItem for File {
    fn get_sort_string(&self) -> &str {
        self.name.as_str()
    }
}

impl PartialEq for File {
    fn eq(&self, other: &File) -> bool {
        self.path == other.path
    }
}

impl Hash for File {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

type FilesDB = FrecencyDB<File>;

impl From<&Item<File>> for ListItem {
    fn from(item: &Item<File>) -> Self {
        Self {
            name: item.item.name.clone(),
            icon: "".to_owned(),
            id: item.uuid.clone(),
        }
    }
}
