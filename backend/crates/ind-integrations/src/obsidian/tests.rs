use super::*;
use chrono::{TimeZone, Utc};
use ind_domain::ItemType;

fn doc(item_type: ItemType) -> ObsidianRenderDocument {
    ObsidianRenderDocument {
        subject_id: "lib_00000000-0000-0000-0000-000000000000".into(),
        subject_kind: "library_entry".into(),
        title: "Example / Article".into(),
        full_title: "Example / Article".into(),
        url: Some("https://example.com".into()),
        author: Some("Ada Lovelace".into()),
        item_type,
        image_url: Some("https://example.com/cover.png".into()),
        summary: Some("Short summary".into()),
        full_document_text: Some("# Full text".into()),
        document_tags: vec!["research".into()],
        highlights: vec![ObsidianRenderHighlight {
            id: "hlt_00000000-0000-0000-0000-000000000000".into(),
            text: "Important text".into(),
            note: Some("Remember this".into()),
            color: "yellow".into(),
            tags: vec!["quote".into()],
            location: Some("Location 1".into()),
            location_url: Some("https://example.com/#h".into()),
            created_at: Utc::now(),
        }],
    }
}

fn render(settings: &ObsidianExportSettings, document: &ObsidianRenderDocument) -> String {
    render_document(
        settings,
        document,
        &ObsidianRenderCursor::default(),
        Utc.with_ymd_and_hms(2026, 5, 5, 9, 7, 0).unwrap(),
    )
    .unwrap()
    .unwrap()
    .entry
    .full_content
    .unwrap()
}

#[test]
fn category_mapping_matches_plan() {
    for (item_type, category) in [
        (ItemType::Book, "books"),
        (ItemType::Pdf, "books"),
        (ItemType::Tweet, "tweets"),
        (ItemType::Podcast, "podcasts"),
        (ItemType::Article, "articles"),
        (ItemType::Email, "articles"),
        (ItemType::Video, "articles"),
    ] {
        assert_eq!(category_for_item_type(item_type), category);
    }
}

#[test]
fn default_render_preserves_metadata_highlights_links_and_optional_sections() {
    let mut settings = ObsidianExportSettings {
        export_all_reader_documents: true,
        ..Default::default()
    };
    settings.properties_template = Some("indelible_id: {{title}}".into());
    let document = doc(ItemType::Article);
    let content = render(&settings, &document);
    for expected in [
        "## Metadata",
        "## Highlights",
        "[[Ada Lovelace]]",
        "#articles",
        "Important text",
        "Remember this",
        "[Location 1](https://example.com/#h)",
    ] {
        assert!(content.contains(expected), "missing {expected}: {content}");
    }
    assert!(!content.starts_with("# Example / Article"));

    let mut without_highlights = doc(ItemType::Article);
    without_highlights.image_url = None;
    without_highlights.highlights.clear();
    without_highlights.full_document_text = Some("Readable article body.".into());
    let rendered = render_document(
        &settings,
        &without_highlights,
        &ObsidianRenderCursor::default(),
        Utc::now(),
    )
    .unwrap()
    .unwrap();
    let content = rendered.entry.full_content.unwrap();
    assert!(content.contains("[Full document text]("));
    assert!(!content.contains("## Highlights"));
    assert!(rendered.entry.full_document_text.is_some());

    without_highlights.author = None;
    without_highlights.url = None;
    without_highlights.summary = None;
    without_highlights.document_tags.clear();
    without_highlights.full_document_text = None;
    let content = render(&Default::default(), &without_highlights);
    for absent in ["Author:", "URL:", "Summary:", "Document Tags:", "{{"] {
        assert!(!content.contains(absent), "unexpected {absent}: {content}");
    }
}

#[test]
fn template_language_features_render_documented_outputs() {
    let cases = [
        (
            ObsidianExportSettings {
                page_title_template: "# {{title}}".into(),
                ..Default::default()
            },
            "# Example / Article",
        ),
        (
            ObsidianExportSettings {
                metadata_template: "{{date|date('Y-m-d')}}|{{date|date('F j, Y')}}|{{time}}".into(),
                ..Default::default()
            },
            "2026-05-05|May 5, 2026|09:07",
        ),
        (
            ObsidianExportSettings {
                page_title_template: "{{ title if title else 'Untitled' }}".into(),
                metadata_template: String::new(),
                ..Default::default()
            },
            "Example / Article",
        ),
        (
            ObsidianExportSettings {
                metadata_template: "{% for tag in document_tags %}[{{tag}}]{% endfor %}".into(),
                highlight_header_template: String::new(),
                highlight_template: "{% for tag in highlight_tags %}#{{tag}}{% endfor %}".into(),
                ..Default::default()
            },
            "[research]",
        ),
    ];
    for (settings, expected) in cases {
        let content = render(&settings, &doc(ItemType::Article));
        assert!(content.contains(expected), "missing {expected}: {content}");
    }

    let mut settings = ObsidianExportSettings {
        file_name_template: Some(
            "{% set name = author|trim %}{{name.split(\" \")|last}} -- {{ title }}".into(),
        ),
        ..Default::default()
    };
    settings.category_folder_templates.insert(
        "articles".into(),
        "{% set folder = category|trim %}{{ folder }}-vault".into(),
    );
    let rendered = render_document(
        &settings,
        &doc(ItemType::Article),
        &ObsidianRenderCursor::default(),
        Utc::now(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        rendered.entry.file_path,
        "Indelible/articles-vault/Lovelace -- Example - Article.md"
    );
}

#[test]
fn highlight_template_exposes_all_public_variables() {
    let settings = ObsidianExportSettings {
        metadata_template: String::new(),
        highlight_header_template: String::new(),
        highlight_template: "{{highlight_text}}|{{highlight_note}}|{{highlight_location}}|{{highlight_location_url}}|{{color}}|{% for tag in highlight_tags %}{{tag}}{% endfor %}".into(),
        ..Default::default()
    };
    assert!(
        render(&settings, &doc(ItemType::Article)).contains(
            "Important text|Remember this|Location 1|https://example.com/#h|yellow|quote"
        )
    );
}

#[test]
fn invalid_templates_return_section_named_errors() {
    let settings = ObsidianExportSettings {
        page_title_template: "{{ title".into(),
        ..Default::default()
    };
    let error = render_document(
        &settings,
        &doc(ItemType::Article),
        &ObsidianRenderCursor::default(),
        Utc::now(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ObsidianRenderError::Template {
            name: "page_title",
            ..
        }
    ));
}

#[test]
fn full_document_text_removes_extractor_title_boilerplate_without_eating_real_prose() {
    let cases = [
        ("", "body", None),
        ("Title", "", None),
        ("Only Title", "Only Title", None),
        (
            "This even smaller credit card-sized e-reader has one tragic flaw",
            "This even smaller credit card-sized e-reader has one tragic flaw Andrew Liszewski There is a new reader.",
            Some("Andrew Liszewski There is a new reader."),
        ),
        (
            "Google Home latest update",
            "Google Home latest update Google Home latest update Emma Roth Google shipped it.",
            Some("Emma Roth Google shipped it."),
        ),
        (
            "Audit System - NewStarT (Custom OWL)",
            "Audit System - NewStarT (Custom OWL) Daniēls Zeps NewStarT Audit is open source.",
            Some("Daniēls Zeps NewStarT Audit is open source."),
        ),
        (
            "404: Page Not Found",
            "404 / page not found Looks like this page is unpublished.",
            Some("Looks like this page is unpublished."),
        ),
        (
            "AI",
            "AI is changing how developers write software.",
            Some("AI is changing how developers write software."),
        ),
        (
            "Flux language",
            "I have been working on Flux language and wanted to explain it.",
            Some("I have been working on Flux language and wanted to explain it."),
        ),
    ];

    for (title, body, expected) in cases {
        assert_eq!(format_full_document_text(title, body).as_deref(), expected);
    }
}
