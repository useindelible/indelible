use ind_domain::EntityType;

pub(super) fn normalize_tag(tag: &str) -> String {
    normalize_optional_text(tag)
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace())
        .to_ascii_lowercase()
}

pub(super) fn normalize_entity_name(name: &str) -> String {
    normalize_optional_text(name)
}

pub(super) fn normalize_optional_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn parse_entity_type(value: &str) -> Option<EntityType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "person" | "people" => Some(EntityType::Person),
        "organization" | "organisation" | "company" | "institution" => {
            Some(EntityType::Organization)
        }
        "location" | "place" | "city" | "country" | "region" => Some(EntityType::Location),
        "event" => Some(EntityType::Event),
        "work" | "topic" | "concept" | "product" | "project" | "technology" | "book" | "paper"
        | "article" | "movie" => Some(EntityType::Work),
        _ => None,
    }
}
