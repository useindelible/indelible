#![allow(clippy::unwrap_used)]

use ind_html::{ArticleTocStatus, derive_article_toc, prepare_reader_html};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn oyo_derives_42_nested_entries() {
    let prepared = prepare_reader_html(&fixture("oyo.html")).unwrap();
    let toc = derive_article_toc(&prepared, "Oyo Empire - Wikipedia");
    assert_eq!(toc.status, ArticleTocStatus::Ready);
    assert!(!toc.truncated);
    assert_eq!(toc.entries.len(), 42);
    assert_eq!(toc.entries[0].title, "History");
    assert_eq!(toc.entries[0].depth, 0);
    assert_eq!(toc.entries[0].source_heading_index, 0);
    let ilari = toc.entries.iter().find(|e| e.title == "The Ilari").unwrap();
    assert_eq!(ilari.depth, 1);
    assert_eq!(ilari.id, "ind-toc-the-ilari");
    assert!(
        ilari.word_count >= 100,
        "Ilari section is several paragraphs, got {}",
        ilari.word_count
    );
    // h5 "Dahomey Wars" sits under h2 > h3 > h4.
    let dahomey = toc
        .entries
        .iter()
        .find(|e| e.title == "Dahomey Wars")
        .unwrap();
    assert_eq!(dahomey.depth, 3);
    // The article body is ~8.2k words; per-section counts must account for the
    // bulk of it (preamble before the first heading is deliberately uncounted).
    let total: u32 = toc.entries.iter().map(|e| e.word_count).sum();
    assert!(
        (6_000..=9_000).contains(&total),
        "section word counts sum to {total}"
    );
}

#[test]
fn lamp_dedupes_title_heading_and_keeps_source_ordinals() {
    let prepared = prepare_reader_html(&fixture("lamp.html")).unwrap();
    let toc = derive_article_toc(
        &prepared,
        "The Sci-Fi Nuclear Core Battery Lamp : 13 Steps (with Pictures) - Instructables",
    );
    assert_eq!(toc.status, ArticleTocStatus::Ready);
    assert_eq!(toc.entries.len(), 14);
    assert_eq!(toc.entries[0].title, "Supplies");
    // Ordinal 0 was the dropped title heading; the positional fallback must not
    // scroll to it.
    assert_eq!(toc.entries[0].source_heading_index, 1);
    assert!(toc.entries.iter().all(|e| e.depth == 0));
}

#[test]
fn quality_bounds() {
    let none0 = derive_article_toc("<p>no headings</p>", "T");
    assert_eq!(none0.status, ArticleTocStatus::None);
    assert!(none0.entries.is_empty());

    let none1 = derive_article_toc(r#"<h2 id="ind-toc-a">A</h2><p>x</p>"#, "T");
    assert_eq!(none1.status, ArticleTocStatus::None);
    assert!(none1.entries.is_empty());

    let two = derive_article_toc(
        r#"<h2 id="ind-toc-a">A</h2><p>x</p><h2 id="ind-toc-b">B</h2>"#,
        "T",
    );
    assert_eq!(two.status, ArticleTocStatus::Ready);
    assert_eq!(two.entries.len(), 2);

    let many: String = (0..201)
        .map(|i| format!("<h2 id=\"ind-toc-x{i}\">X {i}</h2><p>w</p>"))
        .collect();
    let capped = derive_article_toc(&many, "T");
    assert_eq!(capped.entries.len(), 200);
    assert!(capped.truncated);
}

#[test]
fn level_skip_clamps_depth() {
    let toc = derive_article_toc(
        r#"<h2 id="a">A</h2><h5 id="b">B</h5><h2 id="c">C</h2>"#,
        "T",
    );
    assert_eq!(
        toc.entries.iter().map(|e| e.depth).collect::<Vec<_>>(),
        vec![0, 1, 0]
    );
}

#[test]
fn empty_headings_consume_ordinals_but_produce_no_entries() {
    let toc = derive_article_toc(
        r#"<h2 id="a">A</h2><h3 id="e">   </h3><h2 id="b">B</h2>"#,
        "T",
    );
    assert_eq!(toc.entries.len(), 2);
    assert_eq!(toc.entries[1].source_heading_index, 2);
}

#[test]
fn title_prefix_dedupe_requires_minimum_length() {
    // A short first heading that happens to prefix the title must survive.
    let toc = derive_article_toc(
        r#"<h2 id="a">Intro</h2><p>x</p><h2 id="b">B</h2>"#,
        "Introduction to Systems",
    );
    assert_eq!(toc.entries.len(), 2);
    assert_eq!(toc.entries[0].title, "Intro");
}

#[test]
fn per_section_word_counts_split_on_any_heading_level() {
    let toc = derive_article_toc(
        r#"<h2 id="a">A</h2><p>one two three</p><h3 id="b">B</h3><p>four five</p><h2 id="c">C</h2><p>six</p>"#,
        "T",
    );
    assert_eq!(
        toc.entries.iter().map(|e| e.word_count).collect::<Vec<_>>(),
        vec![3, 2, 1]
    );
}
