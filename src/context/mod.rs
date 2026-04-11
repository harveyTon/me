pub mod container;
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
    }
}
