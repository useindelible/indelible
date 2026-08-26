//! Process-level tuning applied before the async runtime starts.

// With THP=always, the kernel backs allocator arenas with 2 MiB pages and triples idle RSS.
// Must run before the runtime and allocator touch any large region.
#[cfg(target_os = "linux")]
pub fn disable_transparent_huge_pages() {
    if let Err(err) = rustix::thread::disable_transparent_huge_pages(true) {
        tracing::warn!(%err, "could not disable transparent huge pages");
    }
}

#[cfg(not(target_os = "linux"))]
pub fn disable_transparent_huge_pages() {}
