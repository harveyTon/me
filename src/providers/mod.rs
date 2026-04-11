pub mod identity;
pub mod network;
pub mod runtime;

use crate::context;
use crate::model::MeInfo;

pub fn collect(config: &crate::config::ContextConfig, fast: bool, collect_network: bool) -> MeInfo {
    let identity = identity::collect();
    let runtime = runtime::collect();
    let context = if fast {
        context::detect_fast(config)
    } else {
        context::detect(config)
    };
    let ssh = context.ssh.is_some();
    MeInfo {
        privilege: if identity.uid == 0 { "root" } else { "user" }.into(),
        sudo: std::env::var_os("SUDO_USER").is_some() || std::env::var_os("SUDO_UID").is_some(),
        ssh,
        network: if collect_network {
            network::collect()
        } else {
            crate::model::NetworkInfo {
                local_ips: Vec::new(),
            }
        },
        identity,
        runtime,
        context,
    }
}
