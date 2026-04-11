use crate::model::SystemIdentity;

#[cfg(unix)]
use std::ffi::CStr;

#[cfg(unix)]
pub fn collect() -> SystemIdentity {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    SystemIdentity {
        user: username(uid).unwrap_or_else(|| uid.to_string()),
        uid,
        gid,
        groups: groups(),
        host: hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown-host".into()),
    }
}

#[cfg(windows)]
pub fn collect() -> SystemIdentity {
    SystemIdentity {
        user: std::env::var("USERNAME").unwrap_or_else(|_| "unknown-user".into()),
        uid: 1,
        gid: 1,
        groups: Vec::new(),
        host: hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .or_else(|| std::env::var("COMPUTERNAME").ok())
            .unwrap_or_else(|| "unknown-host".into()),
    }
}

#[cfg(unix)]
fn username(uid: u32) -> Option<String> {
    let passwd = unsafe { libc::getpwuid(uid) };
    if passwd.is_null() {
        return None;
    }
    unsafe {
        CStr::from_ptr((*passwd).pw_name)
            .to_str()
            .ok()
            .map(str::to_owned)
    }
}

#[cfg(unix)]
fn group_name(gid: u32) -> Option<String> {
    let group = unsafe { libc::getgrgid(gid) };
    if group.is_null() {
        return None;
    }
    unsafe {
        CStr::from_ptr((*group).gr_name)
            .to_str()
            .ok()
            .map(str::to_owned)
    }
}

#[cfg(unix)]
fn groups() -> Vec<String> {
    let count = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    if count <= 0 {
        return Vec::new();
    }
    let mut raw = vec![0 as libc::gid_t; count as usize];
    let actual = unsafe { libc::getgroups(raw.len() as i32, raw.as_mut_ptr()) };
    if actual < 0 {
        return Vec::new();
    }
    raw.truncate(actual as usize);
    raw.into_iter()
        .map(|gid| group_name(gid).unwrap_or_else(|| gid.to_string()))
        .collect()
}
