use std::collections::{HashMap, HashSet};

use ind_application::AppError;
use ind_domain::DomainError;
use ind_integrations::obsidian::{
    MAX_PATH_SEGMENT_BYTES, ObsidianArtifactEntry, full_document_path_for_note_path,
    obsidian_content_hash, obsidian_link_for_path, stable_subject_path_suffix,
};
use tracing::error;

#[derive(Debug)]
pub(super) struct PendingObsidianEntry {
    pub(super) entry: ObsidianArtifactEntry,
    pub(super) generated_path_locked: bool,
    pub(super) generated_full_document_path_locked: bool,
}

pub(super) fn resolve_artifact_paths(
    mut pending_entries: Vec<PendingObsidianEntry>,
) -> Result<Vec<ObsidianArtifactEntry>, AppError> {
    disambiguate_note_paths(&mut pending_entries)?;
    disambiguate_full_document_paths(&mut pending_entries)?;

    let entries = pending_entries
        .into_iter()
        .map(|pending| pending.entry)
        .collect::<Vec<_>>();
    validate_unique_artifact_paths(&entries)?;
    Ok(entries)
}

fn disambiguate_note_paths(entries: &mut [PendingObsidianEntry]) -> Result<(), AppError> {
    let groups = path_groups(
        entries
            .iter()
            .map(|pending| pending.entry.file_path.as_str()),
    );
    let keep_indices =
        keep_indices_for_groups(entries, groups, |pending| pending.generated_path_locked)?;

    let mut occupied = HashSet::new();
    for index in &keep_indices {
        occupied.insert(entries[*index].entry.file_path.clone());
    }

    for (index, pending) in entries.iter_mut().enumerate() {
        if keep_indices.contains(&index) {
            continue;
        }
        let suffix = stable_subject_path_suffix(&pending.entry.subject_id);
        let path = unique_suffixed_markdown_path(&pending.entry.file_path, &suffix, &mut occupied)?;
        update_note_path(pending, path);
    }
    Ok(())
}

fn disambiguate_full_document_paths(entries: &mut [PendingObsidianEntry]) -> Result<(), AppError> {
    let indexed_paths = entries
        .iter()
        .enumerate()
        .filter_map(|(index, pending)| {
            pending
                .entry
                .full_document_text_path
                .as_deref()
                .map(|path| (index, path))
        })
        .collect::<Vec<_>>();
    let groups = path_groups(indexed_paths.iter().map(|(_, path)| *path));
    let mut remapped_groups = HashMap::new();
    for (path, group) in groups {
        remapped_groups.insert(
            path,
            group
                .into_iter()
                .map(|group_index| indexed_paths[group_index].0)
                .collect::<Vec<_>>(),
        );
    }
    let keep_indices = keep_indices_for_groups(entries, remapped_groups, |pending| {
        pending.generated_full_document_path_locked
    })?;

    let mut occupied = HashSet::new();
    for index in &keep_indices {
        if let Some(path) = entries[*index].entry.full_document_text_path.as_ref() {
            occupied.insert(path.clone());
        }
    }

    for (index, pending) in entries.iter_mut().enumerate() {
        if keep_indices.contains(&index) || pending.entry.full_document_text_path.is_none() {
            continue;
        }
        #[expect(
            clippy::expect_used,
            reason = "entries with full_document_text_path.is_none() are skipped via continue on the preceding line"
        )]
        let current_path = pending
            .entry
            .full_document_text_path
            .clone()
            .expect("checked above");
        let suffix = stable_subject_path_suffix(&pending.entry.subject_id);
        let path = unique_suffixed_markdown_path(&current_path, &suffix, &mut occupied)?;
        update_full_document_path(&mut pending.entry, path);
    }
    Ok(())
}

fn path_groups<'a>(paths: impl Iterator<Item = &'a str>) -> HashMap<String, Vec<usize>> {
    let mut groups = HashMap::new();
    for (index, path) in paths.enumerate() {
        groups
            .entry(path.to_string())
            .or_insert_with(Vec::new)
            .push(index);
    }
    groups
}

fn keep_indices_for_groups(
    entries: &[PendingObsidianEntry],
    groups: HashMap<String, Vec<usize>>,
    is_locked: impl Fn(&PendingObsidianEntry) -> bool,
) -> Result<HashSet<usize>, AppError> {
    let mut keep_indices = HashSet::new();
    for (path, group) in groups {
        if group.len() == 1 {
            keep_indices.insert(group[0]);
            continue;
        }

        let locked = group
            .iter()
            .copied()
            .filter(|index| is_locked(&entries[*index]))
            .collect::<Vec<_>>();
        if locked.len() > 1 {
            return Err(AppError::Domain(DomainError::Validation {
                field: "file_path".to_string(),
                message: format!(
                    "multiple already-delivered Obsidian entries share generated path `{path}`; clear/rebuild the affected export cursors"
                ),
            }));
        }

        #[expect(
            clippy::expect_used,
            reason = "a duplicate group always contains at least one member by construction"
        )]
        let keep = locked.into_iter().next().unwrap_or_else(|| {
            group
                .iter()
                .copied()
                .min_by_key(|index| entries[*index].entry.subject_id.clone())
                .expect("duplicate group is non-empty")
        });
        keep_indices.insert(keep);
    }
    Ok(keep_indices)
}

fn update_note_path(pending: &mut PendingObsidianEntry, path: String) {
    pending.entry.file_path = path;
    if pending.entry.full_document_text_path.is_some()
        && !pending.generated_full_document_path_locked
    {
        let full_document_path = full_document_path_for_note_path(&pending.entry.file_path);
        update_full_document_path(&mut pending.entry, full_document_path);
    }
}

fn update_full_document_path(entry: &mut ObsidianArtifactEntry, path: String) {
    let Some(old_path) = entry.full_document_text_path.replace(path.clone()) else {
        return;
    };
    if old_path == path {
        return;
    }

    let old_link = format!(
        "[Full document text]({})",
        obsidian_link_for_path(&old_path)
    );
    let new_link = format!("[Full document text]({})", obsidian_link_for_path(&path));
    if let Some(full_content) = entry.full_content.as_mut()
        && full_content.contains(&old_link)
    {
        *full_content = full_content.replacen(&old_link, &new_link, 1);
        entry.last_content_hash = Some(obsidian_content_hash(full_content));
    }
    if let Some(append_only_content) = entry.append_only_content.as_mut()
        && append_only_content.contains(&old_link)
    {
        *append_only_content = append_only_content.replacen(&old_link, &new_link, 1);
        entry.last_content_hash = Some(obsidian_content_hash(append_only_content));
    }
}

const MAX_PATH_SUFFIX_ATTEMPTS: u32 = 1000;

fn unique_suffixed_markdown_path(
    path: &str,
    suffix: &str,
    occupied: &mut HashSet<String>,
) -> Result<String, AppError> {
    for attempt in 0..MAX_PATH_SUFFIX_ATTEMPTS {
        let candidate_suffix = if attempt == 0 {
            suffix.to_string()
        } else {
            format!("{suffix}-{attempt}")
        };
        let candidate = append_markdown_path_suffix(path, &candidate_suffix);
        if occupied.insert(candidate.clone()) {
            return Ok(candidate);
        }
    }
    Err(AppError::Domain(DomainError::Validation {
        field: "file_path".to_string(),
        message: format!(
            "could not find a unique Obsidian path for `{path}` after \
             {MAX_PATH_SUFFIX_ATTEMPTS} suffix attempts"
        ),
    }))
}

fn append_markdown_path_suffix(path: &str, suffix: &str) -> String {
    let (folder, file_name) = path.rsplit_once('/').unwrap_or(("", path));
    let stem = file_name.strip_suffix(".md").unwrap_or(file_name);
    let (base_stem, trailing_label) = stem
        .strip_suffix(" Full Text")
        .map(|base| (base, " Full Text"))
        .unwrap_or((stem, ""));
    let suffix_text = format!(" - {suffix}");
    let max_base_bytes = MAX_PATH_SEGMENT_BYTES
        .saturating_sub(suffix_text.len())
        .saturating_sub(trailing_label.len());
    let base = truncate_utf8(base_stem, max_base_bytes).trim_end_matches([' ', '.', '-']);
    let file_name = if base.is_empty() {
        format!("Untitled{suffix_text}{trailing_label}.md")
    } else {
        format!("{base}{suffix_text}{trailing_label}.md")
    };
    if folder.is_empty() {
        file_name
    } else {
        format!("{folder}/{file_name}")
    }
}

fn truncate_utf8(input: &str, max_bytes: usize) -> &str {
    if input.len() <= max_bytes {
        return input;
    }
    let mut end = 0;
    for (idx, ch) in input.char_indices() {
        let next = idx + ch.len_utf8();
        if next > max_bytes {
            break;
        }
        end = next;
    }
    &input[..end]
}

fn validate_unique_artifact_paths(entries: &[ObsidianArtifactEntry]) -> Result<(), AppError> {
    let mut seen = HashMap::new();
    for entry in entries {
        validate_unique_artifact_path(&mut seen, &entry.file_path, &entry.subject_id)?;
        if let Some(path) = entry.full_document_text_path.as_deref() {
            validate_unique_artifact_path(&mut seen, path, &entry.subject_id)?;
        }
    }
    Ok(())
}

fn validate_unique_artifact_path(
    seen: &mut HashMap<String, String>,
    path: &str,
    subject_id: &str,
) -> Result<(), AppError> {
    if let Some(first_subject_id) = seen.insert(path.to_string(), subject_id.to_string()) {
        error!(
            artifact_path = path,
            first_subject_id,
            subject_id,
            "Obsidian artifact path collision survived collision resolution"
        );
        return Err(AppError::Domain(DomainError::Validation {
            field: "file_path".to_string(),
            message: format!(
                "Obsidian artifact path `{path}` was generated for both `{first_subject_id}` and `{subject_id}` after collision resolution"
            ),
        }));
    }
    Ok(())
}
