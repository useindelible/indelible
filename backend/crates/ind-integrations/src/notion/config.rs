use ind_domain::NotionExportSettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotionManagedTarget {
    pub database_id: String,
    pub data_source_id: String,
    pub property_ids: NotionPropertyIds,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NotionPropertyIds {
    pub title: String,
    pub author: String,
    pub url: String,
    pub canonical_url: String,
    pub source: String,
    pub saved_at: String,
    pub tags: String,
    pub category: String,
    pub reading_status: String,
    pub indelible_id: String,
    pub last_synced_at: String,
}

impl NotionPropertyIds {
    pub fn is_complete(&self) -> bool {
        [
            &self.title,
            &self.author,
            &self.url,
            &self.canonical_url,
            &self.source,
            &self.saved_at,
            &self.tags,
            &self.category,
            &self.reading_status,
            &self.indelible_id,
            &self.last_synced_at,
        ]
        .iter()
        .all(|v| !v.trim().is_empty())
    }
}

pub fn notion_settings_from_config(config: &serde_json::Value) -> NotionExportSettings {
    let defaults = NotionExportSettings::default();
    NotionExportSettings {
        export_automatically: config
            .get("export_automatically")
            .and_then(|v| v.as_bool())
            .unwrap_or(defaults.export_automatically),
        include_highlight_locations: config
            .get("include_highlight_locations")
            .and_then(|v| v.as_bool())
            .unwrap_or(defaults.include_highlight_locations),
        compact_layout: config
            .get("compact_layout")
            .and_then(|v| v.as_bool())
            .unwrap_or(defaults.compact_layout),
        selection_enabled: config
            .get("selection_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(defaults.selection_enabled),
    }
}

pub fn property_ids_from_config(config: &serde_json::Value) -> Option<NotionPropertyIds> {
    serde_json::from_value(config.get("property_ids")?.clone())
        .ok()
        .filter(NotionPropertyIds::is_complete)
}

pub fn write_settings_to_config(config: &mut serde_json::Value, settings: &NotionExportSettings) {
    config["export_automatically"] = serde_json::Value::Bool(settings.export_automatically);
    config["include_highlight_locations"] =
        serde_json::Value::Bool(settings.include_highlight_locations);
    config["compact_layout"] = serde_json::Value::Bool(settings.compact_layout);
    config["selection_enabled"] = serde_json::Value::Bool(settings.selection_enabled);
}

#[expect(
    clippy::expect_used,
    reason = "NotionPropertyIds is a plain owned struct; serde_json::to_value is infallible for it"
)]
pub fn write_managed_target_to_config(
    config: &mut serde_json::Value,
    target: &NotionManagedTarget,
) {
    config["database_id"] = serde_json::Value::String(target.database_id.clone());
    config["data_source_id"] = serde_json::Value::String(target.data_source_id.clone());
    config["property_ids"] =
        serde_json::to_value(&target.property_ids).expect("NotionPropertyIds should serialize");
    let settings = notion_settings_from_config(config);
    write_settings_to_config(config, &settings);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> NotionPropertyIds {
        let value = |name: &str| format!("id_{name}");
        NotionPropertyIds {
            title: value("title"),
            author: value("author"),
            url: value("url"),
            canonical_url: value("canonical"),
            source: value("source"),
            saved_at: value("saved"),
            tags: value("tags"),
            category: value("category"),
            reading_status: value("status"),
            indelible_id: value("indelible"),
            last_synced_at: value("synced"),
        }
    }

    #[test]
    fn managed_target_round_trip_requires_complete_property_ids() {
        let target = NotionManagedTarget {
            database_id: "database".into(),
            data_source_id: "source".into(),
            property_ids: ids(),
        };
        let mut config = serde_json::json!({"compact_layout": true});
        write_managed_target_to_config(&mut config, &target);
        assert_eq!(config["database_id"], "database");
        assert_eq!(property_ids_from_config(&config), Some(target.property_ids));
        config["property_ids"]["title"] = "".into();
        assert_eq!(property_ids_from_config(&config), None);
    }

    #[test]
    fn settings_use_defaults_for_wrong_types_and_preserve_explicit_values() {
        let defaults = notion_settings_from_config(&serde_json::json!({"compact_layout": "yes"}));
        let fields = |value: &NotionExportSettings| {
            (
                value.export_automatically,
                value.include_highlight_locations,
                value.compact_layout,
                value.selection_enabled,
            )
        };
        assert_eq!(fields(&defaults), fields(&NotionExportSettings::default()));
        let mut config = serde_json::json!({});
        let settings = NotionExportSettings {
            export_automatically: true,
            include_highlight_locations: false,
            compact_layout: true,
            selection_enabled: true,
        };
        write_settings_to_config(&mut config, &settings);
        assert_eq!(
            fields(&notion_settings_from_config(&config)),
            fields(&settings)
        );
    }
}
