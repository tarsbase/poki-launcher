mod implementation;
pub mod interface {
    include!(concat!(env!("OUT_DIR"), "/src/interface.rs"));
}

use env_logger::Env;
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
    let env = Env::new().filter("POKI_LOGGER");
    env_logger::init_from_env(env);

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
    } else {
        if notifier::is_running() {
            if let Err(e) = notifier::notify() {
                eprintln!("{}", e);
                start_ui();
            }
        } else {
            start_ui();
        }
    }
}

fn start_ui() {
    use std::ffi::CString;
    let app_name = std::env::args().next().unwrap();
    let app_name = CString::new(app_name).unwrap();
    unsafe {
        main_cpp(app_name.as_ptr());
    }
}
