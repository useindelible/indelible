use super::opf::looks_like_isbn;
use super::paths::normalize_path;
use super::text::count_words;
use super::*;
use std::io::Write;

fn create_epub_with_chapters(title: &str, author: &str, chapters: &[(&str, &str)]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let cursor = Cursor::new(&mut buf);
        let mut zip = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip.start_file("mimetype", options).unwrap();
        zip.write_all(b"application/epub+zip").unwrap();

        zip.start_file("META-INF/container.xml", options).unwrap();
        zip.write_all(
            br#"<?xml version="1.0"?>
<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
        )
        .unwrap();

        let mut manifest_items = String::new();
        let mut spine_items = String::new();
        let mut ncx_points = String::new();

        for (i, (ch_title, _)) in chapters.iter().enumerate() {
            manifest_items.push_str(&format!(
                r#"    <item id="ch{i}" href="ch{i}.xhtml" media-type="application/xhtml+xml"/>"#
            ));
            manifest_items.push('\n');
            spine_items.push_str(&format!(r#"    <itemref idref="ch{i}"/>"#));
            spine_items.push('\n');
            ncx_points.push_str(&format!(
                    r#"    <navPoint id="navpoint-{i}"><navLabel><text>{ch_title}</text></navLabel><content src="ch{i}.xhtml"/></navPoint>"#
                ));
            ncx_points.push('\n');
        }

        let opf = format!(
            r#"<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>{title}</dc:title>
    <dc:creator>{author}</dc:creator>
    <dc:publisher>Test Publisher</dc:publisher>
    <dc:language>en</dc:language>
    <dc:identifier opf:scheme="ISBN">978-0-123456-78-9</dc:identifier>
  </metadata>
  <manifest>
{manifest_items}    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine toc="ncx">
{spine_items}  </spine>
</package>"#
        );
        zip.start_file("OEBPS/content.opf", options).unwrap();
        zip.write_all(opf.as_bytes()).unwrap();

        let ncx = format!(
            r#"<?xml version="1.0"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/">
  <navMap>
{ncx_points}  </navMap>
</ncx>"#
        );
        zip.start_file("OEBPS/toc.ncx", options).unwrap();
        zip.write_all(ncx.as_bytes()).unwrap();

        for (i, (_, body)) in chapters.iter().enumerate() {
            let xhtml = format!(
                r#"<?xml version="1.0"?>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>Chapter</title><style>body {{ color: red; }}</style></head>
<body>
<h1>Chapter {}</h1>
<p>{body}</p>
</body>
</html>"#,
                i + 1
            );
            zip.start_file(format!("OEBPS/ch{i}.xhtml"), options)
                .unwrap();
            zip.write_all(xhtml.as_bytes()).unwrap();
        }

        zip.finish().unwrap();
    }
    buf
}

#[test]
fn process_minimal_epub() {
    let chapters = &[
        (
            "Introduction",
            "This is the introduction with some words to count for page estimation.",
        ),
        (
            "Chapter One",
            "The first chapter has more content. It discusses many topics in great detail over several paragraphs.",
        ),
    ];
    let data = create_epub_with_chapters("Test Book", "Test Author", chapters);
    let result = process_epub(&data).unwrap();

    assert_eq!(result.metadata.title.as_deref(), Some("Test Book"));
    assert_eq!(result.metadata.author.as_deref(), Some("Test Author"));
    assert_eq!(result.metadata.publisher.as_deref(), Some("Test Publisher"));
    assert_eq!(result.metadata.language.as_deref(), Some("en"));
    assert_eq!(result.metadata.isbn.as_deref(), Some("978-0-123456-78-9"));
    assert_eq!(result.metadata.total_chapters, 2);
    assert!(result.metadata.total_words > 0);

    assert_eq!(result.chapters.len(), 2);
    assert_eq!(result.toc.len(), 2);
    assert_eq!(result.toc[0].start_page, 1);
}

#[test]
fn sanitizes_script_and_style_tags() {
    let chapters = &[(
        "Test",
        r#"<script>alert('xss')</script>Clean text<style>.bad{}</style>"#,
    )];
    let data = create_epub_with_chapters("Test", "Author", chapters);
    let result = process_epub(&data).unwrap();

    let html = &result.chapters[0].html;
    assert!(!html.contains("<script"));
    assert!(!html.contains("<style"));
    assert!(html.contains("Clean text"));
}

#[test]
fn page_estimation_consistency() {
    let long_text = "word ".repeat(1000);
    let chapters = &[
        ("Chapter 1", long_text.as_str()),
        ("Chapter 2", long_text.as_str()),
    ];
    let data = create_epub_with_chapters("Test", "Author", chapters);
    let result = process_epub(&data).unwrap();

    assert_eq!(result.toc[0].start_page, 1);
    let ch1_pages = result.toc[0].word_count / WORDS_PER_PAGE;
    assert!(result.toc[1].start_page > ch1_pages);

    let total_from_words = result.metadata.total_words.div_ceil(WORDS_PER_PAGE);
    assert_eq!(result.metadata.estimated_pages, total_from_words);
}

#[test]
fn invalid_zip_returns_error() {
    let result = process_epub(b"not a zip file");
    assert!(result.is_err());
}

#[test]
fn isbn_detection() {
    assert!(looks_like_isbn("978-0-123456-78-9"));
    assert!(looks_like_isbn("0123456789"));
    assert!(looks_like_isbn("012345678X"));
    assert!(!looks_like_isbn("abc"));
    assert!(!looks_like_isbn("12345"));
}

#[test]
fn normalize_path_handles_relative() {
    assert_eq!(normalize_path("OEBPS/../images/foo.png"), "images/foo.png");
    assert_eq!(normalize_path("OEBPS/./ch1.xhtml"), "OEBPS/ch1.xhtml");
    assert_eq!(normalize_path("a/b/c"), "a/b/c");
}

#[test]
fn count_words_strips_tags() {
    assert!(count_words("<h1>Title</h1><p>Some body text here</p>") >= 5);
}
