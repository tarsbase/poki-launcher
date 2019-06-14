
mod db;
pub mod desktop_entry;
pub mod scan;

use derive_new::*;
use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(new, Debug, Default, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct App {
    pub name: String,
    exec: String,
    #[new(default)]
    score: f32,
}

impl fmt::Display for App {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
