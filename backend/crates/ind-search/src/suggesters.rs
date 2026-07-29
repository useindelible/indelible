use ind_domain::{SearchSuggestion, SearchSuggestionKind};

pub(crate) const IS_VALUES: &[&str] = &[
    "read",
    "unread",
    "archived",
    "favorited",
    "feed",
    "library",
    "blocked",
];
pub(crate) const HAS_VALUES: &[&str] = &["highlights", "notes", "unsubscribe"];
pub(crate) const PINNED_VALUES: &[&str] = &["true", "false"];

/// Filter values like author names or list ids can contain whitespace, which the
/// query parser only keeps as a single token when quoted. Mirror the `entity:`
/// branch and quote any value containing whitespace.
pub(crate) fn quote_if_needed(key: &str, value: &str) -> String {
    if value.chars().any(char::is_whitespace) {
        format!("{key}:\"{value}\"")
    } else {
        format!("{key}:{value}")
    }
}

pub(crate) fn push_static(
    suggestions: &mut Vec<SearchSuggestion>,
    key: &str,
    values: &[&str],
    prefix: &str,
    description: &str,
) {
    for value in values {
        if value.starts_with(prefix) {
            suggestions.push(SearchSuggestion {
                kind: SearchSuggestionKind::Filter,
                label: format!("{key}:{value}"),
                insert_text: format!("{key}:{value}"),
                description: Some(description.to_string()),
            });
        }
    }
}
