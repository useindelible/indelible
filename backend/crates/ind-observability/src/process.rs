//! Process-level tuning applied before the async runtime starts.

// With THP=always, the kernel backs allocator arenas with 2 MiB pages and triples idle RSS.
// Must run before the runtime and allocator touch any large region; tracing is not up yet,
// so a failure goes straight to stderr.
#[cfg(target_os = "linux")]
pub fn disable_transparent_huge_pages() {
    if let Err(err) = rustix::thread::disable_transparent_huge_pages(true) {
        eprintln!("could not disable transparent huge pages: {err}");
    }
}

#[cfg(not(target_os = "linux"))]
pub fn disable_transparent_huge_pages() {}
