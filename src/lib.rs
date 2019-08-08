mod db;
pub mod desktop_entry;
pub mod runner;
pub mod scan;

use derive_new::*;
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(new, Debug, Default, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct App {
    pub name: String,
    exec: String,
    #[new(default)]
    score: f32,
}

impl App {
    pub fn strip_args(&mut self) {
        self.exec = self
            .exec
            .split(" ")
            .filter(|item| !item.starts_with("%"))
            .join(" ");
    }
}

impl fmt::Display for App {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
