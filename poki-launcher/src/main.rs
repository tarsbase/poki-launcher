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

mod ui;

use crate::ui::{DB_PATH, SHOW_ON_START};
use cpp::*;
use env_logger::Env;
use human_panic::setup_panic;
use lib_poki_launcher::prelude::*;
use poki_launcher_notifier as notifier;
use qmetaobject::*;
use std::os::raw::c_void;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "poki-launcher", about = "Poki App Launcher")]
struct Opt {
    /// Dump the apps database to stdout as json and exit
    #[structopt(long)]
    dump_db: bool,
    /// Start the daemon without showing the launcher window or exit if daemon is already running
    #[structopt(long)]
    no_show: bool,
}

fn main() {
    setup_panic!(Metadata {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        authors: "Ben Goldberg <benaagoldberg@gmail.com>".into(),
        homepage: "https://github.com/zethra/poki-launcher".into(),
    });

    let opt = Opt::from_args();
    SHOW_ON_START.with(|b| b.set(!opt.no_show));
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
    } else if !opt.no_show {
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

// Include stuff for later cpp
cpp! {{
#include "src/icon.cpp"
#include <QtCore/QLatin1String>
}}

fn start_ui() {
    let env = Env::new().filter("POKI_LOGGER");
    env_logger::init_from_env(env);
    lazy_static::initialize(&ui::APPS);
    // Install my logger into QT
    install_message_handler(logger);
    // Init ui res
    ui::init_ui();
    let mut engine = QmlEngine::new();
    engine.load_file("qrc:/ui/main.qml".into());
    // Icon provider hack
    let provider = cpp!(unsafe [] -> *mut c_void as "IconProvider*" { return new IconProvider(); });
    engine.add_image_provider("icon".into(), provider);
    engine.exec();
}

/// Make QT use my logger
extern "C" fn logger(msg_type: QtMsgType, context: &QMessageLogContext, msg: &QString) {
    use log::{log, Level};
    let level = match msg_type {
        QtMsgType::QtCriticalMsg | QtMsgType::QtFatalMsg => Level::Error,
        QtMsgType::QtWarningMsg => Level::Warn,
        QtMsgType::QtInfoMsg => Level::Info,
        QtMsgType::QtDebugMsg => Level::Debug,
    };
    log!(
        level,
        "{}:{} [{:?} {} {}] {}",
        context.file(),
        context.line(),
        msg_type,
        context.category(),
        context.function(),
        msg
    );
}
