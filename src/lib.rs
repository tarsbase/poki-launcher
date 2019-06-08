mod desktop_entry;
mod db;
mod runner;
pub mod scan;

use derive_new::*;

#[derive(new, Debug, Default)]
#[allow(dead_code)]
pub struct App {
    name: String,
    exec: String,
    #[new(default)]
    score: f32,
}
