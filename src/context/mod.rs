pub mod container;
pub mod git;
pub mod project;
pub mod ssh;

use crate::{config::ContextConfig, model::ContextInfo};

pub fn detect(config: &ContextConfig) -> ContextInfo {
    if !config.enabled {
        return ContextInfo::default();
    }
    ContextInfo {
        ssh: config.ssh.then(ssh::detect).flatten(),
        container: config.container.then(container::detect).flatten(),
        project: config.project.then(project::detect).flatten(),
        git: config.git.then(git::detect).flatten(),
    }
}

pub fn detect_fast(config: &ContextConfig) -> ContextInfo {
    if !config.enabled {
        return ContextInfo::default();
    }
    ContextInfo {
        ssh: config.ssh.then(ssh::detect).flatten(),
        container: config.container.then(container::detect).flatten(),
        project: config.project.then(project::detect_fast).flatten(),
        git: config.git.then(git::detect_fast).flatten(),
    }
}
