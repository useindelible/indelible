use super::{clean_email_html, prepare_email_for_reader};

#[test]
fn cleaner_table_removes_email_scaffolding_without_damaging_article_content() {
    let cases = [
        (
            "raw head tags",
            "<meta charset=utf-8><style>.bad{}</style><script>bad()</script><noscript>fallback</noscript><p>kept</p>",
            vec!["meta", "style", "script", "noscript"],
            vec!["<p>kept</p>"],
        ),
        (
            "dimension and hidden pixels",
            "<p>kept</p><img src='one' width='1' height=1><img src=zero width=0 height='0'><img src=hidden style='display: none'>",
            vec!["src='one'", "src=zero", "src=hidden"],
            vec!["<p>kept</p>"],
        ),
        (
            "sized and path pixels",
            "<img src=styled style='width: 1px; height:1px'><img src='https://mail.example/open/x'><img src='https://mail.example/beacon/x'><img src='hero.jpg' width=640 height=480>",
            vec!["src=styled", "/open/x", "/beacon/x"],
            vec!["hero.jpg"],
        ),
        (
            "noise links",
            "<a href=mailto:help@example.com>Mail us</a><a href='https://click.trk.mailer.test/x'>Tracked text</a><a href=https://example.com/story>Story</a>",
            vec!["mailto:", "click.trk.mailer.test"],
            vec!["Mail us", "Tracked text", "href=https://example.com/story"],
        ),
        (
            "nested footer",
            "<main><h1>Issue</h1><p>Real article.</p><div><section>Manage preferences</section></div></main><footer>Unsubscribe</footer>",
            vec!["manage preferences", "unsubscribe"],
            vec!["<h1>Issue</h1>", "Real article."],
        ),
        (
            "article wrapper survives footer marker",
            "<table><tr><td><h2>Article</h2><p>Body</p><aside>View in browser</aside></td></tr></table>",
            vec!["view in browser"],
            vec!["<table>", "<h2>Article</h2>"],
        ),
    ];

    for (name, input, removed, kept) in cases {
        let cleaned = clean_email_html(input);
        let lower = cleaned.to_ascii_lowercase();
        for needle in removed {
            assert!(
                !lower.contains(needle),
                "{name}: retained {needle}: {cleaned}"
            );
        }
        for needle in kept {
            assert!(cleaned.contains(needle), "{name}: lost {needle}: {cleaned}");
        }
    }
}

#[test]
fn reader_pipeline_sanitizes_active_content_after_preserving_safe_markup() {
    let reader = prepare_email_for_reader(
        "<article><h1>Safe title</h1><a href='javascript:bad()' onclick='bad()'>link</a><img src='hero.jpg'></article>",
    );
    assert!(reader.contains("Safe title"));
    assert!(reader.contains("hero.jpg"));
    assert!(!reader.contains("javascript:"));
    assert!(!reader.contains("onclick"));
}
