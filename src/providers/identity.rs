use crate::model::SystemIdentity;

#[cfg(unix)]
use std::ffi::CStr;

#[cfg(unix)]
pub fn collect() -> SystemIdentity {
    // SAFETY: `getuid`/`getgid` are thread-safe libc calls with no preconditions.
    let uid = unsafe { libc::getuid() };
    // SAFETY: `getgid` is thread-safe and takes no arguments.
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
    // SAFETY: `getpwuid` returns either null or a valid pointer managed by libc
    // for the current process. We only read from the struct when the pointer is non-null.
    let passwd = unsafe { libc::getpwuid(uid) };
    if passwd.is_null() || unsafe { (*passwd).pw_name.is_null() } {
        return None;
    }
    // SAFETY: `pw_name` is checked for null above and points to a NUL-terminated C string.
    unsafe {
        CStr::from_ptr((*passwd).pw_name)
            .to_str()
            .ok()
            .map(str::to_owned)
    }
}

#[cfg(unix)]
fn group_name(gid: u32) -> Option<String> {
    // SAFETY: `getgrgid` returns either null or a valid pointer managed by libc.
    let group = unsafe { libc::getgrgid(gid) };
    if group.is_null() || unsafe { (*group).gr_name.is_null() } {
        return None;
    }
    // SAFETY: `gr_name` is checked for null above and points to a NUL-terminated C string.
    unsafe {
        CStr::from_ptr((*group).gr_name)
            .to_str()
            .ok()
            .map(str::to_owned)
    }
}

#[cfg(unix)]
fn groups() -> Vec<String> {
    // SAFETY: passing a null pointer with size 0 is the documented probe call for `getgroups`.
    let count = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    if count <= 0 {
        return Vec::new();
    }
    let Ok(count) = usize::try_from(count) else {
        return Vec::new();
    };
    let mut raw = vec![0 as libc::gid_t; count];
    // SAFETY: `raw` is allocated with `count` entries, and we pass its pointer and length.
    let actual = unsafe { libc::getgroups(raw.len() as i32, raw.as_mut_ptr()) };
    if actual < 0 {
        return Vec::new();
    }
    raw.truncate(actual as usize);
    raw.into_iter()
        .map(|gid| group_name(gid).unwrap_or_else(|| gid.to_string()))
        .collect()
}
