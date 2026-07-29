#[derive(Clone, Copy)]
pub(super) struct OperationContract {
    pub(super) method: &'static str,
    pub(super) path: &'static str,
    pub(super) permission: &'static str,
}

#[derive(Clone, Copy)]
pub(super) struct CompositeOperationContract {
    pub(super) method: &'static str,
    pub(super) path: &'static str,
    pub(super) permissions: &'static [&'static str],
}

impl OperationContract {
    const fn new(method: &'static str, path: &'static str, permission: &'static str) -> Self {
        Self {
            method,
            path,
            permission,
        }
    }
}

impl CompositeOperationContract {
    const fn new(
        method: &'static str,
        path: &'static str,
        permissions: &'static [&'static str],
    ) -> Self {
        Self {
            method,
            path,
            permissions,
        }
    }
}

#[rustfmt::skip]
pub(super) const LIBRARY_OPERATIONS: &[OperationContract] = &[
    OperationContract::new("get", "/api/v1/library", "library:read"),
    OperationContract::new("post", "/api/v1/library", "library:write"),
    OperationContract::new("post", "/api/v1/library/uploads", "library:write"),
    OperationContract::new("post", "/api/v1/library/query", "library:read"),
    OperationContract::new("post", "/api/v1/library/from-delivery", "library:write"),
    OperationContract::new("get", "/api/v1/library/count", "library:read"),
    OperationContract::new("get", "/api/v1/library/counts", "library:read"),
    OperationContract::new("get", "/api/v1/library/trash", "library:read"),
    OperationContract::new("post", "/api/v1/library/trash/empty", "library:write"),
    OperationContract::new("get", "/api/v1/library/{library_entry_id}", "library:read"),
    OperationContract::new("delete", "/api/v1/library/{library_entry_id}", "library:write"),
    OperationContract::new("post", "/api/v1/library/{library_entry_id}/restore", "library:write"),
    OperationContract::new("post", "/api/v1/library/{library_entry_id}/purge", "library:write"),
    OperationContract::new("get", "/api/v1/library/{library_entry_id}/tags", "library:read"),
    OperationContract::new("put", "/api/v1/library/{library_entry_id}/tags", "library:write"),
    OperationContract::new("post", "/api/v1/library/{library_entry_id}/triage", "library:write"),
    OperationContract::new("post", "/api/v1/library/{library_entry_id}/favorite", "library:write"),
    OperationContract::new("post", "/api/v1/library/{library_entry_id}/shortlist", "library:write"),
    OperationContract::new("get", "/api/v1/documents/{document_id}", "library:read"),
    OperationContract::new("post", "/api/v1/documents/{document_id}/reprocess", "library:write"),
    OperationContract::new("get", "/api/v1/documents/{document_id}/toc", "library:read"),
    OperationContract::new("get", "/api/v1/documents/{document_id}/entities", "library:read"),
    OperationContract::new("get", "/api/v1/documents/{document_id}/highlights", "library:read"),
    OperationContract::new("post", "/api/v1/documents/{document_id}/highlights", "library:write"),
    OperationContract::new("get", "/api/v1/documents/{document_id}/note", "library:read"),
    OperationContract::new("put", "/api/v1/documents/{document_id}/note", "library:write"),
    OperationContract::new("patch", "/api/v1/documents/{document_id}/progress", "library:write"),
    OperationContract::new("get", "/api/v1/documents/{document_id}/epub/toc", "library:read"),
    OperationContract::new("get", "/api/v1/documents/{document_id}/epub/chapters/{chapter_index}", "library:read"),
    OperationContract::new("get", "/api/v1/collections", "library:read"),
    OperationContract::new("post", "/api/v1/collections", "library:write"),
    OperationContract::new("get", "/api/v1/collections/{id}", "library:read"),
    OperationContract::new("patch", "/api/v1/collections/{id}", "library:write"),
    OperationContract::new("delete", "/api/v1/collections/{id}", "library:write"),
    OperationContract::new("get", "/api/v1/collections/{id}/children", "library:read"),
    OperationContract::new("get", "/api/v1/collections/{id}/entries", "library:read"),
    OperationContract::new("post", "/api/v1/collections/{id}/entries", "library:write"),
    OperationContract::new("delete", "/api/v1/collections/{id}/entries/{library_entry_id}", "library:write"),
    OperationContract::new("get", "/api/v1/highlights/recent", "library:read"),
    OperationContract::new("patch", "/api/v1/highlights/{highlight_id}", "library:write"),
    OperationContract::new("delete", "/api/v1/highlights/{highlight_id}", "library:write"),
    OperationContract::new("put", "/api/v1/highlights/{highlight_id}/note", "library:write"),
    OperationContract::new("delete", "/api/v1/highlights/{highlight_id}/note", "library:write"),
    OperationContract::new("get", "/api/v1/highlights/{highlight_id}/tags", "library:read"),
    OperationContract::new("put", "/api/v1/highlights/{highlight_id}/tags", "library:write"),
    OperationContract::new("get", "/api/v1/tags", "library:read"),
    OperationContract::new("post", "/api/v1/tags", "library:write"),
    OperationContract::new("post", "/api/v1/tags/merge", "library:write"),
    OperationContract::new("get", "/api/v1/tags/{id}", "library:read"),
    OperationContract::new("patch", "/api/v1/tags/{id}", "library:write"),
    OperationContract::new("delete", "/api/v1/tags/{id}", "library:write"),
    OperationContract::new("get", "/api/v1/tags/{id}/entries", "library:read"),
    OperationContract::new("get", "/api/v1/tags/{id}/highlights", "library:read"),
    OperationContract::new("get", "/api/v1/smart-lists", "library:read"),
    OperationContract::new("post", "/api/v1/smart-lists", "library:write"),
    OperationContract::new("get", "/api/v1/smart-lists/{id}", "library:read"),
    OperationContract::new("patch", "/api/v1/smart-lists/{id}", "library:write"),
    OperationContract::new("delete", "/api/v1/smart-lists/{id}", "library:write"),
    OperationContract::new("get", "/api/v1/smart-lists/{id}/entries", "library:read"),
    OperationContract::new("patch", "/api/v1/smart-lists/{id}/pin", "library:write"),
    OperationContract::new("get", "/api/v1/entities", "library:read"),
    OperationContract::new("get", "/api/v1/entities/{id}", "library:read"),
    OperationContract::new("patch", "/api/v1/entities/{id}", "library:write"),
    OperationContract::new("get", "/api/v1/entities/{id}/documents", "library:read"),
    OperationContract::new("post", "/api/v1/entities/{id}/merge", "library:write"),
    OperationContract::new("get", "/api/v1/search", "library:read"),
    OperationContract::new("get", "/api/v1/search/suggestions", "library:read"),
    OperationContract::new("get", "/api/v1/search/recent", "library:read"),
    OperationContract::new("delete", "/api/v1/search/recent", "library:write"),
    OperationContract::new("delete", "/api/v1/search/recent/{recent_search_id}", "library:write"),
    OperationContract::new("get", "/api/v1/imports", "library:read"),
    OperationContract::new("post", "/api/v1/imports/{slug}", "library:write"),
    OperationContract::new("get", "/api/v1/imports/{slug}", "library:read"),
    OperationContract::new("delete", "/api/v1/imports/{slug}/rollback", "library:write"),
];

#[rustfmt::skip]
pub(super) const FEED_OPERATIONS: &[OperationContract] = &[
    OperationContract::new("get", "/api/v1/feeds/search", "feeds:read"),
    OperationContract::new("post", "/api/v1/feeds/subscriptions", "feeds:write"),
    OperationContract::new("get", "/api/v1/feeds/subscriptions", "feeds:read"),
    OperationContract::new("post", "/api/v1/feeds/subscriptions/opml", "feeds:write"),
    OperationContract::new("patch", "/api/v1/feeds/subscriptions/{id}", "feeds:write"),
    OperationContract::new("delete", "/api/v1/feeds/subscriptions/{id}", "feeds:write"),
    OperationContract::new("post", "/api/v1/feeds/subscriptions/{id}/retry", "feeds:write"),
    OperationContract::new("get", "/api/v1/feeds/deliveries", "feeds:read"),
    OperationContract::new("get", "/api/v1/feeds/deliveries/stats", "feeds:read"),
    OperationContract::new("post", "/api/v1/feeds/deliveries/mark-all-seen", "feeds:write"),
    OperationContract::new("post", "/api/v1/feeds/deliveries/read-ahead", "feeds:write"),
    OperationContract::new("get", "/api/v1/feeds/deliveries/{delivery_id}", "feeds:read"),
    OperationContract::new("post", "/api/v1/feeds/deliveries/{delivery_id}/seen", "feeds:write"),
    OperationContract::new("post", "/api/v1/feeds/deliveries/{delivery_id}/dismiss", "feeds:write"),
    OperationContract::new("post", "/api/v1/feeds/deliveries/{delivery_id}/prepare", "feeds:write"),
    OperationContract::new("get", "/api/v1/email-aliases", "feeds:read"),
    OperationContract::new("post", "/api/v1/email-aliases", "feeds:write"),
    OperationContract::new("delete", "/api/v1/email-aliases/{id}", "feeds:write"),
    OperationContract::new("get", "/api/v1/email-senders", "feeds:read"),
    OperationContract::new("patch", "/api/v1/email-senders/{id}", "feeds:write"),
    OperationContract::new("post", "/api/v1/email-senders/{id}/unsubscribe", "feeds:write"),
];

#[rustfmt::skip]
pub(super) const INTEGRATION_OPERATIONS: &[OperationContract] = &[
    OperationContract::new("get", "/api/v1/integrations", "integrations:read"),
    OperationContract::new("post", "/api/v1/integrations/{provider}/authorize", "integrations:write"),
    OperationContract::new("delete", "/api/v1/integrations/{id}", "integrations:write"),
    OperationContract::new("post", "/api/v1/integrations/{id}/sync", "integrations:write"),
    OperationContract::new("get", "/api/v1/integrations/{id}/notion/settings", "integrations:read"),
    OperationContract::new("patch", "/api/v1/integrations/{id}/notion/settings", "integrations:write"),
    OperationContract::new("get", "/api/v1/integrations/{id}/notion/export-entries", "integrations:read"),
    OperationContract::new("patch", "/api/v1/integrations/{id}/notion/export-entries", "integrations:write"),
    OperationContract::new("post", "/api/v1/integrations/{id}/notion/export-entries/{library_entry_id}/refresh", "integrations:write"),
    OperationContract::new("get", "/api/v1/integrations/{id}/obsidian/settings", "integrations:read"),
    OperationContract::new("patch", "/api/v1/integrations/{id}/obsidian/settings", "integrations:write"),
    OperationContract::new("post", "/api/v1/integrations/{id}/obsidian/preview", "integrations:read"),
    OperationContract::new("post", "/api/v1/integrations/obsidian/setup", "integrations:write"),
];

#[rustfmt::skip]
pub(super) const WEBHOOK_OPERATIONS: &[OperationContract] = &[
    OperationContract::new("get", "/api/v1/webhooks", "webhooks:read"),
    OperationContract::new("post", "/api/v1/webhooks", "webhooks:write"),
    OperationContract::new("patch", "/api/v1/webhooks/{webhook_id}", "webhooks:write"),
    OperationContract::new("delete", "/api/v1/webhooks/{webhook_id}", "webhooks:write"),
    OperationContract::new("post", "/api/v1/webhooks/{webhook_id}/rotate-secret", "webhooks:write"),
    OperationContract::new("post", "/api/v1/webhooks/{webhook_id}/test", "webhooks:write"),
    OperationContract::new("get", "/api/v1/webhooks/{webhook_id}/deliveries", "webhooks:read"),
];

#[rustfmt::skip]
pub(super) const AI_OPERATIONS: &[CompositeOperationContract] = &[
    CompositeOperationContract::new("get", "/api/v1/mila/status", &["ai:read"]),
    CompositeOperationContract::new("get", "/api/v1/mila/config", &["ai:read"]),
    CompositeOperationContract::new("post", "/api/v1/mila/config", &["ai:write"]),
    CompositeOperationContract::new("post", "/api/v1/mila/config/reindex", &["ai:write", "ai:use", "library:read"]),
    CompositeOperationContract::new("post", "/api/v1/mila/config/test", &["ai:use"]),
    CompositeOperationContract::new("get", "/api/v1/mila/presets", &["ai:read"]),
    CompositeOperationContract::new("post", "/api/v1/mila/presets", &["ai:write"]),
    CompositeOperationContract::new("patch", "/api/v1/mila/presets/{preset_id}", &["ai:write"]),
    CompositeOperationContract::new("delete", "/api/v1/mila/presets/{preset_id}", &["ai:write"]),
    CompositeOperationContract::new("get", "/api/v1/mila/sessions", &["ai:read"]),
    CompositeOperationContract::new("post", "/api/v1/mila/sessions", &["ai:write"]),
    CompositeOperationContract::new("get", "/api/v1/mila/sessions/{session_id}/messages", &["ai:read"]),
    CompositeOperationContract::new("delete", "/api/v1/mila/sessions/{session_id}", &["ai:write"]),
    CompositeOperationContract::new("get", "/api/v1/mila/stream", &["ai:use", "library:read"]),
    CompositeOperationContract::new("get", "/api/v1/tts/voice-personas", &["ai:read"]),
    CompositeOperationContract::new("post", "/api/v1/tts/voice-personas", &["ai:write", "ai:use"]),
    CompositeOperationContract::new("post", "/api/v1/documents/{document_id}/tts/sessions", &["ai:use", "library:read"]),
    CompositeOperationContract::new("get", "/api/v1/documents/{document_id}/tts/chunks/{chunk_id}", &["ai:read", "library:read"]),
    CompositeOperationContract::new("get", "/api/v1/documents/{document_id}/tts/timestamp", &["ai:read", "library:read"]),
    CompositeOperationContract::new("patch", "/api/v1/documents/{document_id}/playback", &["ai:write"]),
    CompositeOperationContract::new("get", "/api/v1/documents/{document_id}/playback", &["ai:read"]),
];

#[rustfmt::skip]
pub(super) const OBSIDIAN_SYNC_OPERATIONS: &[CompositeOperationContract] = &[
    CompositeOperationContract::new("post", "/api/v1/export/obsidian/runs", &["obsidian:sync"]),
    CompositeOperationContract::new("get", "/api/v1/export/obsidian/runs/{run_id}", &["obsidian:sync"]),
    CompositeOperationContract::new("get", "/api/v1/export/obsidian/artifacts/{artifact_id}", &["obsidian:sync"]),
    CompositeOperationContract::new("post", "/api/v1/export/obsidian/runs/{run_id}/ack", &["obsidian:sync"]),
    CompositeOperationContract::new("post", "/api/v1/export/obsidian/refresh", &["obsidian:sync"]),
    CompositeOperationContract::new("post", "/api/v1/export/obsidian/rename", &["obsidian:sync"]),
];

#[rustfmt::skip]
pub(super) const JWT_ONLY_OPERATIONS: &[(&str, &str)] = &[
    ("get", "/api/v1/me"),
    ("patch", "/api/v1/me"),
    ("delete", "/api/v1/me"),
    ("post", "/api/v1/me/password"),
    ("post", "/api/v1/me/email"),
    ("post", "/api/v1/me/avatar"),
    ("get", "/api/v1/onboarding"),
    ("post", "/api/v1/onboarding/steps/{step}/complete"),
    ("post", "/api/v1/onboarding/skip"),
    ("get", "/api/v1/home"),
    ("get", "/api/v1/settings/home"),
    ("patch", "/api/v1/settings/home"),
    ("get", "/api/v1/settings/preferences"),
    ("patch", "/api/v1/settings/preferences"),
    ("get", "/api/v1/settings/notifications"),
    ("patch", "/api/v1/settings/notifications"),
    ("get", "/api/v1/settings/archival"),
    ("patch", "/api/v1/settings/archival"),
    ("get", "/api/v1/events/stream"),
];

#[rustfmt::skip]
pub(super) const EXTENSION_JWT_OPERATIONS: &[(&str, &str)] = &[
    ("get", "/api/v1/extension/check-url"),
    ("get", "/api/v1/extension/entries/{library_entry_id}/highlights"),
    ("post", "/api/v1/extension/entries/{library_entry_id}/highlights"),
    ("get", "/api/v1/extension/entries/{library_entry_id}/assets/{asset_kind}"),
    ("get", "/api/v1/extension/entries/{library_entry_id}"),
    ("patch", "/api/v1/extension/entries/{library_entry_id}"),
    ("put", "/api/v1/extension/entries/{library_entry_id}/note"),
    ("put", "/api/v1/extension/entries/{library_entry_id}/tags"),
    ("post", "/api/v1/extension/quick-save"),
    ("post", "/api/v1/extension/reader-save"),
    ("post", "/api/v1/extension/full-archive"),
];
