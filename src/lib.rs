
mod db;
pub mod desktop_entry;
pub mod scan;

use derive_new::*;
use serde_derive::{Deserialize, Serialize};

#[derive(new, Debug, Default, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct App {
    name: String,
    exec: String,
    #[new(default)]
    score: f32,
}
