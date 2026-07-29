use ind_domain::ObsidianExportSettings;

pub fn settings_from_config(raw: &serde_json::Value) -> ObsidianExportSettings {
    raw.get("obsidian_export")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

#[expect(
    clippy::expect_used,
    reason = "raw is coerced to a JSON object immediately above; ObsidianExportSettings serializes infallibly"
)]
pub fn write_settings_to_config(raw: &mut serde_json::Value, settings: &ObsidianExportSettings) {
    if !raw.is_object() {
        *raw = serde_json::json!({});
    }
    raw.as_object_mut().expect("object above").insert(
        "obsidian_export".to_string(),
        serde_json::to_value(settings).expect("ObsidianExportSettings serializes"),
    );
}
