/***
 * This file is part of Poki Launcher.
 *
 * Poki Launcher is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Poki Launcher is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Poki Launcher.  If not, see <https://www.gnu.org/licenses/>.
 */
mod implementation;
pub mod interface {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/src/interface.rs"));
}

use env_logger::Env;
use human_panic::setup_panic;
use implementation::DB_PATH;
use lib_poki_launcher::prelude::AppsDB;
use poki_launcher_notifier as notifier;
use structopt::StructOpt;

extern "C" {
    fn main_cpp(app: *const std::os::raw::c_char);
}

#[derive(Debug, StructOpt)]
#[structopt(name = "poki-launcher", about = "Poki App Launcher")]
struct Opt {
    #[structopt(long)]
    dump_db: bool,
}

fn main() {
    setup_panic!(Metadata {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        authors: "Ben Goldberg <benaagoldberg@gmail.com>".into(),
        homepage: "https://github.com/zethra/poki-launcher".into(),
    });

    let opt = Opt::from_args();
    if opt.dump_db {
        use std::fs::File;

        if DB_PATH.exists() {
            let mut file = File::open(&*DB_PATH).expect("Failed to open db file");
            let data: AppsDB = rmp_serde::from_read(&mut file).expect("Failed to parse db");
            println!("{}", serde_json::to_string_pretty(&data).unwrap());
        } else {
            eprintln!("Database file doesn't exit");
            std::process::exit(1);
        }
    } else if notifier::is_running() {
        if let Err(e) = notifier::notify() {
            eprintln!("{}", e);
            start_ui();
        }
    } else {
        start_ui();
    }
}

fn start_ui() {
    let env = Env::new().filter("POKI_LOGGER");
    env_logger::init_from_env(env);

    use std::ffi::CString;
    let app_name = std::env::args().next().unwrap();
    let app_name = CString::new(app_name).unwrap();
    unsafe {
        main_cpp(app_name.as_ptr());
    }
}
