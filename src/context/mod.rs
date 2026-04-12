pub mod container;
pub mod git;
pub mod project;
pub mod ssh;

use crate::{config::ContextConfig, model::ContextInfo};

pub fn detect(config: &ContextConfig) -> ContextInfo {
    detect_with(config, false)
}

pub fn detect_fast(config: &ContextConfig) -> ContextInfo {
    detect_with(config, true)
}

fn detect_with(config: &ContextConfig, fast: bool) -> ContextInfo {
    if !config.enabled {
        return ContextInfo::default();
    }
    ContextInfo {
        ssh: config.ssh.then(ssh::detect).flatten(),
        container: config.container.then(container::detect).flatten(),
        projects: if config.project {
            project::detect(fast)
        } else {
            Vec::new()
        },
        git: config.git.then(|| git::detect(fast)).flatten(),
    }
}
