use crate::model::ContainerContext;
use std::{fs, path::Path};

pub fn detect() -> Option<ContainerContext> {
    if Path::new("/.dockerenv").exists() {
        return Some(ContainerContext {
            kind: "docker".into(),
            id: cgroup_id(),
        });
    }
    cgroup_id().map(|id| ContainerContext {
        kind: "container".into(),
        id: Some(id),
    })
}

fn cgroup_id() -> Option<String> {
    let cgroup = fs::read_to_string("/proc/1/cgroup").ok()?;
    cgroup
        .split(|c: char| !c.is_ascii_hexdigit())
        .find(|part| part.len() >= 12)
        .map(|part| part[..12].to_owned())
}
