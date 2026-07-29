pub(super) fn resolve_path_from_chapter(opf_dir: &str, chapter_dir: &str, href: &str) -> String {
    if let Some(stripped) = href.strip_prefix('/') {
        return stripped.to_string();
    }

    let base = if chapter_dir.is_empty() {
        opf_dir.to_string()
    } else {
        format!("{}{}", opf_dir, chapter_dir)
    };

    normalize_path(&format!("{}{}", base, href))
}

pub(super) fn resolve_path(base_dir: &str, href: &str) -> String {
    if let Some(stripped) = href.strip_prefix('/') {
        return stripped.to_string();
    }
    normalize_path(&format!("{}{}", base_dir, href))
}

pub(super) fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    parts.join("/")
}
