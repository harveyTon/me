use crate::model::NetworkInfo;
use std::net::IpAddr;

pub fn collect() -> NetworkInfo {
    let mut ips = local_ip_address::list_afinet_netifas()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(_, ip)| match ip {
            IpAddr::V4(v4) if !v4.is_loopback() && !v4.is_link_local() => Some(v4.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    ips.sort();
    ips.dedup();
    NetworkInfo { local_ips: ips }
}
