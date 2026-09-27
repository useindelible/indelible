ALTER TABLE archive_assets
    ADD COLUMN format_version integer NOT NULL DEFAULT 1,
    ADD COLUMN format_issue text,
    ADD CONSTRAINT chk_archive_assets_format_issue
        CHECK (format_issue IN ('missing_chapter', 'needs_reupload'));

CREATE INDEX idx_archive_assets_epub_format_pending ON archive_assets (id)
    WHERE asset_kind = 'epub' AND format_version < 2 AND status = 'completed'
      AND format_issue IS NULL;
