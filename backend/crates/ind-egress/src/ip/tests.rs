use super::is_blocked_ip;
use std::net::IpAddr;

fn ip(s: &str) -> IpAddr {
    s.parse().expect("valid ip literal")
}

#[test]
fn ip_classification_table_blocks_internal_and_allows_public_ranges() {
    let blocked = [
        "0.0.0.0",
        "0.1.2.3",
        "10.0.0.1",
        "10.255.255.255",
        "100.64.0.1",
        "100.127.255.255",
        "127.0.0.1",
        "127.1.2.3",
        "169.254.169.254", // AWS/GCP/DO metadata
        "169.254.0.1",
        "172.16.0.1",
        "172.31.255.255",
        "192.0.0.1",
        "192.0.2.5", // TEST-NET-1
        "192.168.1.1",
        "198.18.0.1", // benchmarking
        "198.19.255.255",
        "198.51.100.7", // TEST-NET-2
        "203.0.113.9",  // TEST-NET-3
        "224.0.0.1",    // multicast
        "239.255.255.255",
        "240.0.0.1", // reserved
        "255.255.255.255",
        "::1",                    // loopback
        "::",                     // unspecified
        "fc00::1",                // ULA
        "fd00:ec2::254",          // AWS IMDS IPv6
        "fe80::1",                // link-local
        "ff02::1",                // multicast
        "2001:db8::1",            // documentation
        "::ffff:127.0.0.1",       // IPv4-mapped loopback
        "::ffff:169.254.169.254", // IPv4-mapped metadata
        "::ffff:10.0.0.1",        // IPv4-mapped private
        "64:ff9b::7f00:1",        // NAT64 -> 127.0.0.1
        "64:ff9b::a9fe:a9fe",     // NAT64 -> 169.254.169.254
        "::7f00:1",               // IPv4-compatible -> 127.0.0.1
    ];
    for s in blocked {
        assert!(is_blocked_ip(ip(s)), "expected {s} to be blocked");
    }
    let allowed = [
        "1.1.1.1",
        "8.8.8.8",
        "93.184.216.34",
        "100.63.255.255",
        "100.128.0.1",
        "2606:4700:4700::1111",
        "2001:4860:4860::8888",
    ];
    for s in allowed {
        assert!(!is_blocked_ip(ip(s)), "expected {s} to be allowed");
    }
}
