use std::error::Error;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

pub struct Notifier;

const DOMAIN: &str = "info.bengoldberg.poki_launcher";
const OBJ_PATH: &str = "/launcher";
const METHOD: &str = "show";

impl Notifier {
    pub fn start() -> Receiver<()> {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            Notifier::start_bg(tx).expect("Failed to start dbus server");
        });
        rx
    }

    fn start_bg(tx: Sender<()>) -> Result<(), Box<dyn Error>> {
        use dbus::{blocking::Connection, tree::Factory};
        let mut c = Connection::new_session()?;
        c.request_name(DOMAIN, false, true, false)?;
        let f = Factory::new_fn::<()>();
        let tree = f.tree(()).add(
            f.object_path(OBJ_PATH, ())
                .introspectable()
                .add(
                    f.interface(DOMAIN, ())
                        .add_m(f.method(METHOD, (), move |m| {
                            tx.send(()).expect("Failed to send show message");
                            Ok(vec![m.msg.method_return()])
                        })),
                ),
        );
        tree.start_receive(&c);
        loop {
            c.process(Duration::from_millis(1000))?;
        }
    }
}

pub fn notify() -> Result<(), Box<dyn Error>> {
    use dbus::ffidisp::Connection;
    let conn = Connection::new_session()?;
    let obj = conn.with_path(DOMAIN, OBJ_PATH, 5000);
    obj.method_call(DOMAIN, METHOD, ())?;
    Ok(())
}
