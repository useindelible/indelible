//! SSRF address classification.
//!
//! This is a deliberately boring, explicit list of the IP ranges a server-side
//! fetch must never reach. It exists as hand-written code (rather than a
//! third-party crate or `std` helpers) for two concrete reasons:
//!
//! - `std`'s convenient predicates for the non-obvious ranges — `is_global`,
//!   `is_shared` (CGNAT 100.64/10), `is_benchmarking` (198.18/15) — are still
//!   behind the unstable `#![feature(ip)]` flag, so they cannot be used on
//!   stable Rust. `std` only offers `is_private`/`is_loopback`/`is_link_local`.
//! - The IPv6 branches catch real, documented bypasses where a loopback or
//!   metadata IPv4 is disguised as IPv6 (`::ffff:127.0.0.1`, NAT64, and
//!   IPv4-compatible forms). We classify on the parsed `Ipv6Addr`, never on a
//!   bracketed string, which avoids the `[::1]`-style parsing pitfall.
//!
//! Keep it explicit and unit-tested rather than clever. Every range here is a
//! standard reserved block; do not remove entries to "simplify".

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Returns `true` when an address must never be the target of a server-side
/// fetch: loopback, private, link-local, CGNAT, benchmarking, documentation,
/// multicast, reserved, broadcast, and cloud-metadata ranges.
pub fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_v4(v4),
        IpAddr::V6(v6) => is_blocked_v6(v6),
    }
}

fn is_blocked_v4(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    match o[0] {
        0 => true,   // 0.0.0.0/8        "this network"
        10 => true,  // 10.0.0.0/8       private
        127 => true, // 127.0.0.0/8      loopback
        _ => {
            // 100.64.0.0/10            carrier-grade NAT
            (o[0] == 100 && (64..=127).contains(&o[1]))
                // 169.254.0.0/16       link-local (incl. cloud metadata 169.254.169.254)
                || (o[0] == 169 && o[1] == 254)
                // 172.16.0.0/12        private
                || (o[0] == 172 && (16..=31).contains(&o[1]))
                // 192.0.0.0/24         IETF protocol assignments
                || (o[0] == 192 && o[1] == 0 && o[2] == 0)
                // 192.0.2.0/24         TEST-NET-1
                || (o[0] == 192 && o[1] == 0 && o[2] == 2)
                // 192.168.0.0/16       private
                || (o[0] == 192 && o[1] == 168)
                // 198.18.0.0/15        benchmarking
                || (o[0] == 198 && (o[1] == 18 || o[1] == 19))
                // 198.51.100.0/24      TEST-NET-2
                || (o[0] == 198 && o[1] == 51 && o[2] == 100)
                // 203.0.113.0/24       TEST-NET-3
                || (o[0] == 203 && o[1] == 0 && o[2] == 113)
                // 224.0.0.0/4 multicast, 240.0.0.0/4 reserved, 255.255.255.255 broadcast
                || o[0] >= 224
        }
    }
}

fn is_blocked_v6(ip: Ipv6Addr) -> bool {
    if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
        return true;
    }

    let seg = ip.segments();

    // fc00::/7 unique-local
    if (seg[0] & 0xfe00) == 0xfc00 {
        return true;
    }
    // fe80::/10 link-local
    if (seg[0] & 0xffc0) == 0xfe80 {
        return true;
    }
    // 2001:db8::/32 documentation
    if seg[0] == 0x2001 && seg[1] == 0x0db8 {
        return true;
    }

    // ::ffff:0:0/96 IPv4-mapped — re-check the embedded v4.
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_blocked_v4(v4);
    }

    // 64:ff9b::/96 NAT64 — re-check the embedded v4.
    if seg[0] == 0x0064 && seg[1] == 0xff9b && seg[2..6] == [0, 0, 0, 0] {
        return is_blocked_v4(embedded_v4(seg));
    }

    // ::/96 IPv4-compatible (deprecated) — re-check the embedded v4.
    // ::1 and :: are already handled above by loopback/unspecified.
    if seg[0..6] == [0, 0, 0, 0, 0, 0] {
        return is_blocked_v4(embedded_v4(seg));
    }

    false
}

fn embedded_v4(seg: [u16; 8]) -> Ipv4Addr {
    Ipv4Addr::new(
        (seg[6] >> 8) as u8,
        (seg[6] & 0xff) as u8,
        (seg[7] >> 8) as u8,
        (seg[7] & 0xff) as u8,
    )
}

#[cfg(test)]
#[path = "ip/tests.rs"]
mod tests;
