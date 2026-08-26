#![allow(clippy::unwrap_used)]

const YT: &str = r#"<div class="yt-embed"><iframe width="560" height="315" src="https://www.youtube.com/embed/abc123" frameborder="0" allowfullscreen></iframe></div><section class="yt-transcript"><p><span class="t-seg" data-t="0:12">Hello</span></p></section>"#;

#[test]
fn reader_sanitizer_keeps_youtube_embed_and_reader_classes() {
    let out = ind_html::sanitize_reader_html(YT);
    assert!(
        out.contains(r#"src="https://www.youtube.com/embed/abc123""#),
        "{out}"
    );
    assert!(out.contains(r#"class="yt-embed""#), "{out}");
    assert!(out.contains(r#"data-t="0:12""#), "{out}");
}

#[test]
fn reader_sanitizer_drops_non_youtube_iframes() {
    let out = ind_html::sanitize_reader_html(
        r#"<p>a</p><iframe src="https://evil.example/x"></iframe><iframe src="javascript:alert(1)"></iframe><iframe></iframe>"#,
    );
    assert!(!out.contains("iframe"), "{out}");
}

#[test]
fn reader_sanitizer_still_strips_scripts_and_handlers() {
    let out = ind_html::sanitize_reader_html(r#"<p onclick="x()">a</p><script>1</script>"#);
    assert_eq!(out, "<p>a</p>");
}

#[test]
fn prepare_is_idempotent_on_youtube_html() {
    let once = ind_html::prepare_reader_html(YT).unwrap();
    let twice = ind_html::prepare_reader_html(&once).unwrap();
    assert_eq!(once, twice);
    assert!(once.contains("youtube.com/embed/abc123"));
    assert!(once.contains(r#"class="t-seg""#));
}
