use crate::model::NetworkInfo;
use std::net::IpAddr;

pub fn collect() -> NetworkInfo {
    collect_from(local_ip_address::list_afinet_netifas())
}

fn collect_from(result: Result<Vec<(String, IpAddr)>, local_ip_address::Error>) -> NetworkInfo {
    let Ok(interfaces) = result else {
        return NetworkInfo::default();
    };

    let mut ipv4_local_ips = Vec::new();
    let mut ipv6_local_ips = Vec::new();
    for (_, ip) in interfaces {
        match ip {
            IpAddr::V4(v4) if !v4.is_loopback() && !v4.is_link_local() => {
                ipv4_local_ips.push(v4.to_string());
            }
            IpAddr::V6(v6)
                if !v6.is_loopback() && !v6.is_unspecified() && !v6.is_unicast_link_local() =>
            {
                ipv6_local_ips.push(v6.to_string());
            }
            _ => {}
        }
    }

    ipv4_local_ips.sort();
    ipv4_local_ips.dedup();
    ipv6_local_ips.sort();
    ipv6_local_ips.dedup();
    NetworkInfo {
        ipv4_local_ips,
        ipv6_local_ips,
    }
}

#[cfg(test)]
mod tests {
    use super::collect_from;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn splits_ipv4_and_ipv6_addresses() {
        let collected = collect_from(Ok(vec![
            (
                "en0".to_string(),
                IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
            ),
            (
                "en0v6".to_string(),
                IpAddr::V6(Ipv6Addr::new(0xfd12, 0, 0, 0, 0, 0, 0, 1)),
            ),
        ]));

        assert_eq!(collected.ipv4_local_ips, vec!["192.168.1.10"]);
        assert_eq!(collected.ipv6_local_ips, vec!["fd12::1"]);
    }

    #[test]
    fn quietly_returns_empty_network_info_on_collection_failure() {
        let collected = collect_from(Err(local_ip_address::Error::LocalIpAddressNotFound));
        assert!(collected.ipv4_local_ips.is_empty());
        assert!(collected.ipv6_local_ips.is_empty());
    }
}
