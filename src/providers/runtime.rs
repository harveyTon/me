use crate::model::RuntimeInfo;
use std::{env, path::Path};

#[cfg(unix)]
use std::ffi::CStr;

pub fn collect() -> RuntimeInfo {
    RuntimeInfo {
        shell: shell_name(),
        pid: std::process::id(),
        ppid: parent_pid(),
        tty: tty_name(),
    }
}

fn shell_name() -> Option<String> {
    env::var("SHELL")
        .or_else(|_| env::var("ComSpec"))
        .ok()
        .and_then(|s| {
            Path::new(&s)
                .file_name()
                .map(|v| v.to_string_lossy().into_owned())
        })
}

#[cfg(unix)]
fn parent_pid() -> Option<u32> {
    Some(unsafe { libc::getppid() as u32 })
}

#[cfg(windows)]
fn parent_pid() -> Option<u32> {
    None
}

#[cfg(unix)]
fn tty_name() -> Option<String> {
    let ptr = unsafe { libc::ttyname(libc::STDIN_FILENO) };
    if ptr.is_null() {
        return None;
    }
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .ok()
            .map(|s| s.trim_start_matches("/dev/").to_owned())
    }
}

#[cfg(windows)]
fn tty_name() -> Option<String> {
    None
}
