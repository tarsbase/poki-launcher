mod implementation;
pub mod interface {
    include!(concat!(env!("OUT_DIR"), "/src/interface.rs"));
}

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(long)]
    show: bool,
}

extern "C" {
    fn main_cpp(app: *const ::std::os::raw::c_char);
}

fn main() {
    let opt = Opt::from_args();
    if opt.show {
        poki_launcher_notifier::notify().expect("Failed to send dbus message");
    } else {
        use std::ffi::CString;
        let app_name = ::std::env::args().next().unwrap();
        let app_name = CString::new(app_name).unwrap();
        unsafe {
            main_cpp(app_name.as_ptr());
        }
    }
}
