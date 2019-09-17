use std::error::Error;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

pub struct Notifier;

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
        c.request_name("info.bengoldberg.pokilauncher", false, true, false)?;
        let f = Factory::new_fn::<()>();
        let tree = f.tree(()).add(
            f.object_path("/launcher", ()).introspectable().add(
                f.interface("info.bengoldberg.pokilauncher", ())
                    .add_m(f.method("show", (), move |m| {
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
    let obj = conn.with_path("info.bengoldberg.pokilauncher", "/launcher", 5000);
    obj.method_call("info.bengoldberg.pokilauncher", "show", ())?;
    Ok(())
}
