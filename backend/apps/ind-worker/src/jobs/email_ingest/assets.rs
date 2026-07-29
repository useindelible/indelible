use ind_integrations::email::prepare_email_for_reader;

/// Convert a plain-text email body to reader-safe HTML so a text-only email still
/// produces a readable document asset (TASK-236). Blank lines become paragraph
/// breaks, single newlines become `<br>`, and the result is passed through
/// `prepare_email_for_reader` so it shares the same reader-safe shell as HTML bodies.
pub(crate) fn email_text_body_to_reader_html(text_body: &str) -> String {
    let escaped = text_body
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let paragraphs: String = escaped
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| format!("<p>{}</p>", p.replace('\n', "<br>")))
        .collect();
    let body = if paragraphs.is_empty() {
        "<p></p>".to_string()
    } else {
        paragraphs
    };
    prepare_email_for_reader(&body)
}
