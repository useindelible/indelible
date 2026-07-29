use std::io::Cursor;

use zip::ZipArchive;

use crate::archive_limits::ArchiveReadBudget;

pub(super) fn read_zip_text(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    name: &str,
    budget: &mut ArchiveReadBudget,
) -> Option<String> {
    let mut file = archive.by_name(name).ok()?;
    // EPUB tolerates a missing/over-budget entry by skipping it (None), keeping a
    // bomb from OOMing the upload while still producing a partial book.
    let bytes = budget.read_capped(&mut file).ok()?;
    String::from_utf8(bytes).ok()
}

pub(super) fn read_zip_bytes(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    name: &str,
    budget: &mut ArchiveReadBudget,
) -> Option<Vec<u8>> {
    let mut file = archive.by_name(name).ok()?;
    budget.read_capped(&mut file).ok()
}
