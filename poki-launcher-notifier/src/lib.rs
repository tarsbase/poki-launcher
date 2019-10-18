use failure::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread;

const LOCK_FILE_PATH: &'static str = "/tmp/poki-launcher.pid";

pub enum Msg {
    Show,
    Exit,
}

pub fn is_running() -> bool {
    Path::new(&LOCK_FILE_PATH).exists()
}

pub struct Notifier(Receiver<Msg>);

impl Notifier {
    pub fn start() -> Result<Notifier, Error> {
        use nix::unistd::getpid;
        use signal_hook::iterator::Signals;
        use signal_hook::*;

        let mut file = File::create(&LOCK_FILE_PATH)?;
        write!(file, "{}", getpid())?;
        drop(file);
        let signals = Signals::new(&[SIGUSR1, SIGINT, SIGTERM, SIGQUIT])?;
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            for signal in signals.forever() {
                match signal {
                    SIGUSR1 => tx.send(Msg::Show).expect("Failed to send show message"),
                    SIGINT | SIGTERM | SIGQUIT => {
                        tx.send(Msg::Exit).expect("Failed to send show message");
                        break;
                    }
                    _ => (),
                }
            }
        });
        Ok(Notifier(rx))
    }

    pub fn recv(&self) -> Result<Msg, std::sync::mpsc::RecvError> {
        self.0.recv()
    }
}

impl Drop for Notifier {
    fn drop(&mut self) {
        if is_running() {
            std::fs::remove_file(&LOCK_FILE_PATH).expect("Failed to delete lock file");
        }
    }
}

pub fn notify() -> Result<(), Error> {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    let mut file = File::open(&LOCK_FILE_PATH)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    kill(Pid::from_raw(buf.parse()?), Signal::SIGUSR1)?;
    Ok(())
}
