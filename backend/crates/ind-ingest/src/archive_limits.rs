use std::io::Read;

/// Why an archive entry could not be read within the decompression budget.
#[derive(Debug, thiserror::Error)]
pub enum ArchiveReadError {
    #[error("archive entry exceeds the decompression budget")]
    BudgetExceeded,
    #[error("failed to read archive entry: {0}")]
    Io(#[from] std::io::Error),
}

/// Limits for in-memory archive (zip) extraction, sized to comfortably fit the
/// largest legitimate EPUB / import while refusing decompression bombs.
#[derive(Debug, Clone, Copy)]
pub struct ArchiveLimits {
    pub max_entries: usize,
    pub max_entry_bytes: u64,
    pub max_total_bytes: u64,
}

impl ArchiveLimits {
    /// EPUB extraction. The compressed input is already capped at
    /// `MAX_UPLOAD_BYTES`, but a small zip can still inflate enormously, so the
    /// decompressed output is bounded too (all chapters plus base64-inlined
    /// images are held in memory at once).
    pub const EPUB: Self = Self {
        max_entries: 5_000,
        max_entry_bytes: 25 * 1024 * 1024,
        max_total_bytes: 200 * 1024 * 1024,
    };

    /// Readwise / import archives processed by the worker.
    pub const IMPORT: Self = Self {
        max_entries: 50_000,
        max_entry_bytes: 64 * 1024 * 1024,
        max_total_bytes: 512 * 1024 * 1024,
    };
}

/// Tracks the remaining decompression budget across an archive's entries so a
/// bomb — one huge entry, or many medium entries — cannot exhaust memory. The
/// per-entry and total caps are enforced against the *actual* decompressed
/// bytes (via [`Read::take`]), not the zip header's declared size, which a bomb
/// can understate.
#[derive(Debug)]
pub struct ArchiveReadBudget {
    max_entry_bytes: u64,
    remaining_total: u64,
}

impl ArchiveReadBudget {
    pub fn new(limits: ArchiveLimits) -> Self {
        Self {
            max_entry_bytes: limits.max_entry_bytes,
            remaining_total: limits.max_total_bytes,
        }
    }

    /// Read `reader` to the end, refusing to decompress past the per-entry cap or
    /// the remaining total budget. Returns [`ArchiveReadError::BudgetExceeded`]
    /// when either limit would be exceeded so the caller can decide whether to
    /// skip the entry (EPUB) or fail the whole archive (import); never buffers
    /// past the cap, so it cannot OOM.
    pub fn read_capped<R: Read>(&mut self, reader: &mut R) -> Result<Vec<u8>, ArchiveReadError> {
        let cap = self.max_entry_bytes.min(self.remaining_total);
        let mut buf = Vec::new();
        // take(cap + 1): reading more than `cap` bytes means the entry is over
        // budget; the +1 lets us detect that without unbounded buffering.
        reader.take(cap.saturating_add(1)).read_to_end(&mut buf)?;
        if buf.len() as u64 > cap {
            return Err(ArchiveReadError::BudgetExceeded);
        }
        self.remaining_total -= buf.len() as u64;
        Ok(buf)
    }
}
