mod desktop_entry;
mod db;

use derive_new::*;

#[derive(new, Debug, Default)]
#[allow(dead_code)]
struct App {
    name: String,
    exec: String,
    #[new(default)]
    score: f32,
}
