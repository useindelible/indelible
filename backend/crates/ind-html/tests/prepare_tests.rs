#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use lol_html::html_content::Element;
use lol_html::{RewriteStrSettings, element, rewrite_str};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn lol_html_can_set_attributes_via_selector() {
    let settings = RewriteStrSettings::new().append_element_content_handler(element!(
        "h2",
        |el: &mut Element| {
            el.set_attribute("id", "ind-toc-history")?;
            Ok(())
        }
    ));
    let out = rewrite_str("<h2>History</h2>", settings).unwrap();
    assert_eq!(out, r#"<h2 id="ind-toc-history">History</h2>"#);
}

#[test]
fn prepare_injects_heading_ids_on_oyo() {
    let out = ind_html::prepare_reader_html(&fixture("oyo.html")).unwrap();
    assert!(out.contains(r#"<h3 id="ind-toc-the-ilari">"#));
    assert_eq!(
        out.matches("<h3").count(),
        fixture("oyo.html").matches("<h3").count()
    );
}

#[test]
fn prepare_is_byte_idempotent_on_fixtures() {
    for f in ["oyo.html", "lamp.html"] {
        let once = ind_html::prepare_reader_html(&fixture(f)).unwrap();
        let twice = ind_html::prepare_reader_html(&once).unwrap();
        assert_eq!(twice, once, "{f} not idempotent");
    }
}

#[test]
fn prepare_prefixes_generic_id_targets() {
    let out =
        ind_html::prepare_reader_html(r##"<div id="notes">n</div><a href="#notes">see notes</a>"##)
            .unwrap();
    assert!(out.contains(r#"id="ind-notes""#), "{out}");
    assert!(out.contains(r##"href="#ind-notes""##), "{out}");
}

#[test]
fn injected_slugs_avoid_collision_with_prefixed_existing_ids() {
    // Existing id "toc-history" prefixes to "ind-toc-history"; a heading titled
    // "History" would slug to the same final id — dedupe must fire against the
    // final namespace, not the raw pre-prefix id set.
    let out =
        ind_html::prepare_reader_html(r#"<p id="toc-history">x</p><h2>History</h2><h2>Real</h2>"#)
            .unwrap();
    assert!(out.contains(r#"<p id="ind-toc-history">"#), "{out}");
    assert!(
        out.contains(r#"<h2 id="ind-toc-history-2">History</h2>"#),
        "{out}"
    );
    let doc = scraper::Html::parse_document(&out);
    let sel = scraper::Selector::parse("[id]").unwrap();
    let ids: Vec<String> = doc
        .select(&sel)
        .map(|e| e.value().attr("id").unwrap().to_string())
        .collect();
    let unique: HashSet<&String> = ids.iter().collect();
    assert_eq!(ids.len(), unique.len(), "duplicate final ids: {ids:?}");
}

#[test]
fn prepare_resolves_every_oyo_fragment_link() {
    let out = ind_html::prepare_reader_html(&fixture("oyo.html")).unwrap();
    let doc = scraper::Html::parse_document(&out);
    let id_sel = scraper::Selector::parse("[id]").unwrap();
    let ids: HashSet<String> = doc
        .select(&id_sel)
        .map(|e| e.value().attr("id").unwrap().to_string())
        .collect();
    let a_sel = scraper::Selector::parse(r##"a[href^="#"]"##).unwrap();
    let hrefs: Vec<String> = doc
        .select(&a_sel)
        .map(|e| e.value().attr("href").unwrap()[1..].to_string())
        .collect();
    // The source has 336 fragment hrefs; 6 point at targets extraction removed
    // (3 CITEREF bibliography entries, 3 backlinks to dropped citations). Those
    // are neutralized (href stripped), and every surviving link must resolve.
    assert_eq!(hrefs.len(), 330);
    let dead: Vec<&String> = hrefs.iter().filter(|h| !ids.contains(*h)).collect();
    assert!(
        dead.is_empty(),
        "{} unresolved fragments: {:?}",
        dead.len(),
        &dead[..dead.len().min(20)]
    );
    for neutralized in [
        "ind-fnref:1-5",
        "ind-fnref:2",
        "ind-fnref:3-2",
        "ind-CITEREFBosman1704",
        "ind-CITEREFSnelgrave1734",
        "ind-CITEREFLander1830",
    ] {
        assert!(
            !out.contains(&format!("href=\"#{neutralized}\"")),
            "orphaned link survived: {neutralized}"
        );
    }
}

#[test]
fn prepare_preserves_and_prefixes_existing_ids_and_still_sanitizes() {
    let out = ind_html::prepare_reader_html(
        r##"<h2 id="History">History</h2><a href="#History">go</a><script>evil()</script><p onclick="x()">t</p>"##,
    )
    .unwrap();
    assert!(out.contains(r#"id="ind-History""#), "{out}");
    assert!(out.contains(r##"href="#ind-History""##), "{out}");
    assert!(!out.contains("script"), "{out}");
    assert!(!out.contains("onclick"), "{out}");
}

#[test]
fn duplicate_heading_slugs_get_suffixes() {
    let out =
        ind_html::prepare_reader_html("<h2>Structure</h2><p>a</p><h2>Structure</h2>").unwrap();
    assert!(out.contains(r#"id="ind-toc-structure""#), "{out}");
    assert!(out.contains(r#"id="ind-toc-structure-2""#), "{out}");
}

#[test]
fn unsluggable_headings_fall_back_to_ordinal_ids() {
    let out = ind_html::prepare_reader_html("<h2>!!!</h2><p>a</p><h2>???</h2>").unwrap();
    assert!(out.contains(r#"id="ind-toc-0""#), "{out}");
    assert!(out.contains(r#"id="ind-toc-1""#), "{out}");
}

#[test]
fn footnote_lis_and_citations_get_inferred_ids() {
    let html = r##"
        <p>Claim<a href="#fn:1">[1]</a> and again<a href="#fn:1">[1]</a>.</p>
        <ol>
            <li>Source. <a href="#fnref:1">back</a> <a href="#fnref:1-2">back2</a></li>
        </ol>"##;
    let out = ind_html::prepare_reader_html(html).unwrap();
    let doc = scraper::Html::parse_document(&out);
    let a_sel = scraper::Selector::parse("a[href]").unwrap();
    let anchors: Vec<(String, Option<String>)> = doc
        .select(&a_sel)
        .map(|a| {
            (
                a.value().attr("href").unwrap().to_string(),
                a.value().attr("id").map(str::to_string),
            )
        })
        .collect();
    assert!(
        anchors.contains(&("#ind-fn:1".into(), Some("ind-fnref:1".into()))),
        "{anchors:?}"
    );
    assert!(
        anchors.contains(&("#ind-fn:1".into(), Some("ind-fnref:1-2".into()))),
        "{anchors:?}"
    );
    assert!(out.contains(r#"<li id="ind-fn:1">"#), "{out}");
    assert!(out.contains(r##"href="#ind-fnref:1""##), "{out}");
}
