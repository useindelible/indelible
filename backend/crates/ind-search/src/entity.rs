use ind_domain::{EntityType, SearchEntityChip, SearchFilter};

pub(crate) fn entity_filter_values(parsed: &ind_domain::ParsedSearchQuery) -> Vec<String> {
    parsed
        .filters
        .iter()
        .filter_map(|filter| match filter {
            SearchFilter::Entity {
                value,
                negated: false,
            } => Some(value.to_lowercase()),
            _ => None,
        })
        .collect()
}

pub(crate) fn chip_matches_query(
    chip: &SearchEntityChip,
    entity_filters: &[String],
    normalized_text_query: Option<&str>,
) -> bool {
    let chip_name = chip.name.to_lowercase();
    entity_filters.iter().any(|value| value == &chip_name)
        || normalized_text_query
            .map(|query| chip_name.contains(query) || query.contains(&chip_name))
            .unwrap_or(false)
}

pub(crate) fn entity_type_label(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::Person => "Person",
        EntityType::Organization => "Organization",
        EntityType::Location => "Location",
        EntityType::Event => "Event",
        EntityType::Work => "Work",
    }
}
