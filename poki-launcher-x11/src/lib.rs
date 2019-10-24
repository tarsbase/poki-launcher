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
use std::collections::VecDeque;
use std::ffi::CString;
use std::os::raw::c_ulong;
use std::process::Command;
use x11::xlib::{
    ClientMessage, ClientMessageData, SubstructureNotifyMask, SubstructureRedirectMask, Window,
    XClientMessageEvent, XDefaultRootWindow, XInternAtom, XMapRaised, XOpenDisplay, XSendEvent,
};

pub fn foreground(name: &str) -> bool {
    Command::new("wmctrl")
        .arg("-a")
        .arg(&name)
        .status()
        .unwrap()
        .success()
}

pub fn _foreground(pid: u64) {
    let mut success = false;
    let mut pids = VecDeque::from(vec![pid]);

    while !pids.is_empty() {
        let pid = pids.pop_front().unwrap();
        println!("Trying: {}", pid);
        if foreground_pid(pid).is_ok() {
            println!("{} success", pid);
            success = true;
        }
        for pid in get_children(pid) {
            pids.push_back(pid);
        }
    }

    println!("Success: {}", success);
}

fn get_children(pid: u64) -> Vec<u64> {
    let output = Command::new("pgrep")
        .arg("-P")
        .arg(pid.to_string())
        .output()
        .expect("pgrep failed")
        .stdout;
    let output = String::from_utf8(output).expect("pgrep output not utf-8");
    output
        .lines()
        .map(str::parse)
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .collect()
}

fn foreground_pid(pid: u64) -> Result<(), &'static str> {
    unsafe {
        let disp = XOpenDisplay(libc::PT_NULL as *const i8);
        let msg = CString::new("_NET_ACTIVE_WINDOW").expect("CString::new failed");
        let data = ClientMessageData::new();
        let client_msg = XClientMessageEvent {
            type_: ClientMessage,
            serial: 0,
            send_event: true.into(),
            message_type: XInternAtom(disp, msg.as_ptr(), false.into()),
            window: pid as Window,
            format: 32,
            display: disp,
            data,
        };
        let mask = SubstructureNotifyMask | SubstructureRedirectMask;
        if XSendEvent(
            disp,
            XDefaultRootWindow(disp),
            false.into(),
            mask,
            &mut client_msg.into(),
        ) != 0
        {
            return Err("XSendEvent faild");
        };
        XMapRaised(disp, pid as c_ulong);
    }
    Ok(())
}
