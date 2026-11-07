use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;

/// A valid EPUB with one spine item and one NCX entry per `(title, body)` chapter.
pub fn build_epub(title: &str, chapters: &[(&str, &str)]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let mut write = |path: &str, contents: &str| {
        zip.start_file(path, options).unwrap();
        zip.write_all(contents.as_bytes()).unwrap();
    };
    write("mimetype", "application/epub+zip");
    write(
        "META-INF/container.xml",
        r#"<?xml version="1.0"?>
<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0">
  <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>"#,
    );
    let (mut manifest, mut spine, mut nav) = (String::new(), String::new(), String::new());
    for (index, (chapter_title, _)) in chapters.iter().enumerate() {
        manifest.push_str(&format!(
            r#"<item id="ch{index}" href="ch{index}.xhtml" media-type="application/xhtml+xml"/>"#
        ));
        spine.push_str(&format!(r#"<itemref idref="ch{index}"/>"#));
        nav.push_str(&format!(
            r#"<navPoint id="nav{index}"><navLabel><text>{chapter_title}</text></navLabel><content src="ch{index}.xhtml"/></navPoint>"#
        ));
    }
    write(
        "OEBPS/content.opf",
        &format!(
            r#"<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>{title}</dc:title></metadata>
  <manifest>{manifest}<item id="toc" href="toc.ncx" media-type="application/x-dtbncx+xml"/></manifest>
  <spine toc="toc">{spine}</spine>
</package>"#
        ),
    );
    write(
        "OEBPS/toc.ncx",
        &format!(
            r#"<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/"><navMap>{nav}</navMap></ncx>"#
        ),
    );
    for (index, (chapter_title, body)) in chapters.iter().enumerate() {
        write(
            &format!("OEBPS/ch{index}.xhtml"),
            &format!(
                r#"<html xmlns="http://www.w3.org/1999/xhtml"><head><title>{chapter_title}</title></head><body><h1>{chapter_title}</h1><p>{body}</p></body></html>"#
            ),
        );
    }
    zip.finish().unwrap().into_inner()
}
