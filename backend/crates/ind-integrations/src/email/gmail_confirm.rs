/// Detects Gmail "forward-from" confirmation emails. Google sends one of these
/// when a user sets up forwarding from a Gmail account to a third-party
/// address; the email contains a confirmation URL that, when fetched, completes
/// the handshake. The auto-handler skips the regular ingest pipeline for these.
pub fn is_gmail_confirmation(from_address: &str, subject: &str) -> bool {
    from_address
        .trim()
        .eq_ignore_ascii_case("forwarding-noreply@google.com")
        && subject
            .to_ascii_lowercase()
            .contains("gmail forwarding confirmation")
}

/// Returns the first `https://mail-settings.google.com/...` URL found in `body`,
/// validated as a parseable URL with the expected host. Strips trailing
/// punctuation that commonly bleeds in from surrounding prose. Returns `None`
/// if no qualifying URL is present.
pub fn extract_confirmation_url(body: &str) -> Option<String> {
    const PREFIX: &str = "https://mail-settings.google.com/";

    let mut search_from = 0;
    while let Some(rel) = body[search_from..].find(PREFIX) {
        let start = search_from + rel;
        let end = body[start..]
            .find(|c: char| c.is_whitespace() || c == '<' || c == '"' || c == '\'')
            .map(|n| start + n)
            .unwrap_or(body.len());

        let raw = &body[start..end];
        let trimmed = raw.trim_end_matches([')', ']', '}', ',', '.', ';', '!', '?']);
        let decoded = decode_url_entities(trimmed);

        if let Ok(parsed) = url::Url::parse(&decoded)
            && parsed.host_str() == Some("mail-settings.google.com")
        {
            return Some(decoded);
        }

        search_from = end;
    }
    None
}

/// Gmail's email link points at `mail-settings.google.com`, but the endpoint
/// that accepts programmatic confirmation expects the same path on
/// `mail.google.com` with an empty POST body.
pub fn confirmation_submit_url(confirmation_url: &str) -> Option<String> {
    let mut parsed = url::Url::parse(confirmation_url).ok()?;
    if parsed.scheme() != "https" || parsed.host_str() != Some("mail-settings.google.com") {
        return None;
    }
    parsed.set_host(Some("mail.google.com")).ok()?;
    Some(parsed.to_string())
}

fn decode_url_entities(raw: &str) -> String {
    raw.replace("&amp;", "&")
        .replace("&#38;", "&")
        .replace("&#x26;", "&")
        .replace("&#X26;", "&")
}
