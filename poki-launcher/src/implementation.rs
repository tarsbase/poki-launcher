use super::interface::*;
use lib_poki_launcher::prelude::*;
use std::path::Path;

const DB_PATH: &'static str = "apps.db";
const MAX_APPS_SHOWN: usize = 5;

pub struct AppsModel {
    emit: AppsModelEmitter,
    model: AppsModelList,
    list: Vec<App>,
    apps: AppsDB,
}

impl AppsModelTrait for AppsModel {
    fn new(emit: AppsModelEmitter, model: AppsModelList) -> AppsModel {
        let db_path = Path::new(&DB_PATH);
        let apps = if db_path.exists() {
            AppsDB::load(&DB_PATH).unwrap()
        } else {
            let apps = AppsDB::from_desktop_entries().unwrap();
            apps.save(&DB_PATH).expect("Faile to write db to disk");
            apps
        };

        AppsModel {
            emit,
            model,
            list: Vec::new(),
            apps,
        }
    }

    fn emit(&mut self) -> &mut AppsModelEmitter {
        &mut self.emit
    }

    fn row_count(&self) -> usize {
        self.list.len()
    }

    fn name(&self, index: usize) -> &str {
        if index < self.list.len() {
            &self.list[index].name
        } else {
            ""
        }
    }

    fn set_name(&mut self, index: usize, name: String) -> bool {
        if index >= self.list.len() {
            return false;
        }
        self.list[index].name = name;
        true
    }

    fn search(&mut self, text: String) {
        self.model.begin_reset_model();
        self.list = self.apps.get_ranked_list(&text);
        self.model.end_reset_model();
    }
}
