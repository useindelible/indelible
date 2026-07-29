use std::io::Write;
use std::sync::Arc;

use ind_application::repos::prepared_content::PreparedContentProvider;
use ind_domain::{DocumentId, GenericJobEnvelope, JobOutboxId};
use ind_ingest::prepared_content::AssetBackedPreparedContentProvider;
use ind_persistence::repos::{
    PgDocumentAssetRepository, PgDocumentRepository, PgMilaConfigRepository,
};
use ind_test_support::{DocumentFactory, spawn_app};
use reqwest::StatusCode;
use reqwest::multipart::{Form, Part};

use super::common::{assert_json_response, build_worker_context};

#[tokio::test]
async fn pdf_upload_and_readwise_import_cross_http_storage_and_persistence() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);

    let upload_form = Form::new()
        .part(
            "file",
            Part::bytes(build_minimal_pdf("Indelible surgical upload coverage"))
                .file_name("surgical-coverage.pdf")
                .mime_str("application/pdf")
                .expect("valid PDF MIME type"),
        )
        .text("title", "Surgical PDF");
    let uploaded = assert_json_response(
        client
            .post_multipart("/api/v1/library/uploads", upload_form)
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(uploaded["document_type"], "pdf");
    assert_eq!(uploaded["source"], "manual");
    assert_eq!(uploaded["title"], "Surgical PDF");
    let document_id = uploaded["document_id"].as_str().expect("document id");
    let reader = assert_json_response(
        client
            .get(&format!("/api/v1/documents/{document_id}"))
            .await,
        StatusCode::OK,
    )
    .await;
    let assets = reader["available_assets"]
        .as_array()
        .expect("available assets");
    for expected in ["original_upload", "pdf", "extracted_text", "thumbnail"] {
        assert!(
            assets.iter().any(|asset| asset == expected),
            "missing {expected}: {assets:?}"
        );
    }

    let csv = b"Title,URL,ID,Document tags,Saved date,Reading progress,Location,Seen\n\
        Real Import,https://example.com/imported,abc123,[],2024-01-15 10:00:00+00:00,0.0,new,False\n";
    let import_form = Form::new().part(
        "library_csv",
        Part::bytes(csv.to_vec())
            .file_name("library.csv")
            .mime_str("text/csv")
            .expect("valid CSV MIME type"),
    );
    let import = assert_json_response(
        client
            .post_multipart("/api/v1/imports/readwise", import_form)
            .await,
        StatusCode::ACCEPTED,
    )
    .await;
    assert_eq!(import["status"], "awaiting_provider");
    let import_id = import["import_job_id"].as_str().expect("import job id");
    let persisted = assert_json_response(
        client.get(&format!("/api/v1/imports/{import_id}")).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(persisted["id"], import_id);
    assert_eq!(persisted["import_source"], "readwise_import");
    assert_eq!(persisted["status"], "awaiting_provider");
    let recent =
        assert_json_response(client.get("/api/v1/imports?limit=1").await, StatusCode::OK).await;
    assert_eq!(recent["jobs"][0]["id"], import_id);
}

#[tokio::test]
async fn epub_upload_sanitizes_chapters_extracts_cover_and_rejects_type_mismatch() {
    let app = spawn_app().await;
    let session = app.create_web_session().await;
    let client = app.authed_client(&session);
    let form = Form::new().part(
        "file",
        Part::bytes(build_minimal_epub())
            .file_name("boundary.epub")
            .mime_str("application/epub+zip")
            .expect("valid EPUB MIME type"),
    );
    let uploaded = assert_json_response(
        client.post_multipart("/api/v1/library/uploads", form).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(uploaded["document_type"], "book");
    assert_eq!(uploaded["title"], "Boundary EPUB");
    assert_eq!(uploaded["author"], "Ada Reader");
    let document_id = uploaded["document_id"].as_str().expect("document id");
    let original = client
        .get(&format!(
            "/api/v1/assets/documents/{document_id}/original_upload"
        ))
        .await;
    assert_eq!(original.status(), StatusCode::OK);
    assert_eq!(original.headers()["x-content-type-options"], "nosniff");
    assert_eq!(
        original.headers()[reqwest::header::CONTENT_TYPE],
        "application/epub+zip"
    );
    assert!(!original.bytes().await.unwrap().is_empty());
    let reader = assert_json_response(
        client
            .get(&format!("/api/v1/documents/{document_id}"))
            .await,
        StatusCode::OK,
    )
    .await;
    let assets = reader["available_assets"].as_array().expect("assets");
    for expected in ["original_upload", "epub", "extracted_text", "thumbnail"] {
        assert!(assets.iter().any(|asset| asset == expected));
    }
    let toc = assert_json_response(
        client
            .get(&format!("/api/v1/documents/{document_id}/epub/toc"))
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(toc["toc"][0]["title"], "Chapter One");
    let chapter = client
        .get(&format!("/api/v1/documents/{document_id}/epub/chapters/0"))
        .await;
    assert_eq!(chapter.status(), StatusCode::OK);
    let chapter = chapter.text().await.expect("chapter body");
    assert!(chapter.contains("Chapter One"));
    assert!(!chapter.contains("script"));

    let document: DocumentId = document_id.parse().unwrap();
    sqlx::query(
        "UPDATE archive_assets SET status = 'failed', failed_reason = 'boundary failure' \
         WHERE document_id = $1 AND asset_kind IN ('epub', 'extracted_text')",
    )
    .bind(document.into_uuid())
    .execute(app.pool())
    .await
    .unwrap();
    let reprocess = assert_json_response(
        client
            .post_json(
                &format!("/api/v1/documents/{document_id}/reprocess"),
                &serde_json::json!({}),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(reprocess["queued"], true);
    let (outbox_id, payload, dedupe_key): (uuid::Uuid, serde_json::Value, Option<String>) =
        sqlx::query_as(
            "SELECT id, payload, dedupe_key FROM job_outbox \
             WHERE job_type = 'document.reprocess' AND payload->>'document_id' = $1",
        )
        .bind(document_id)
        .fetch_one(app.pool())
        .await
        .unwrap();
    ind_worker::jobs::render::dispatch_generic_job(
        &build_worker_context(&app),
        GenericJobEnvelope {
            outbox_id: JobOutboxId::from_uuid(outbox_id),
            job_type: "document.reprocess".into(),
            payload,
            dedupe_key,
        },
    )
    .await
    .unwrap();
    let repaired: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM archive_assets \
         WHERE document_id = $1 AND asset_kind IN ('epub', 'extracted_text') AND status = 'completed'",
    )
    .bind(document.into_uuid())
    .fetch_one(app.pool())
    .await
    .unwrap();
    assert_eq!(repaired, 2);

    let prepared = AssetBackedPreparedContentProvider::new(
        Arc::new(PgDocumentRepository::new(app.pool().clone())),
        Arc::new(PgDocumentAssetRepository::new(app.pool().clone())),
        Arc::new(PgMilaConfigRepository::new(app.pool().clone())),
        Some(app.storage().await),
    )
    .load_for_document(document)
    .await
    .unwrap()
    .expect("reprocessed EPUB should prepare chapter content");
    assert_eq!(prepared.parents.len(), 1);
    assert_eq!(prepared.parents[0].title.as_deref(), Some("Chapter One"));
    assert!(
        prepared
            .root_text
            .contains("Durable sanitized book content")
    );
    assert!(!prepared.leaves.is_empty());

    let mismatch = Form::new().part(
        "file",
        Part::bytes(build_minimal_pdf("mismatch"))
            .file_name("not-an-epub.epub")
            .mime_str("application/pdf")
            .expect("valid PDF MIME type"),
    );
    assert_eq!(
        client
            .post_multipart("/api/v1/library/uploads", mismatch)
            .await
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn reprocess_rejects_a_document_owned_by_another_user() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let stranger = app.create_web_session().await;
    let document = DocumentFactory::new(owner.user.id).insert(app.pool()).await;

    let response = app
        .authed_client(&stranger)
        .post_json(
            &format!("/api/v1/documents/{}/reprocess", document.id),
            &serde_json::json!({}),
        )
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

fn build_minimal_epub() -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file("mimetype", options).unwrap();
    zip.write_all(b"application/epub+zip").unwrap();
    zip.start_file("META-INF/container.xml", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0"?>
<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0">
  <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>"#,
    )
    .unwrap();
    zip.start_file("OEBPS/content.opf", options).unwrap();
    zip.write_all(
        br#"<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Boundary EPUB</dc:title><dc:creator>Ada Reader</dc:creator>
    <meta name="cover" content="cover-image"/>
  </metadata>
  <manifest>
    <item id="cover-image" href="cover.gif" media-type="image/gif" properties="cover-image"/>
    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
    <item id="toc" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine toc="toc"><itemref idref="chapter"/></spine>
</package>"#,
    )
    .unwrap();
    zip.start_file("OEBPS/cover.gif", options).unwrap();
    zip.write_all(b"GIF89a\x01\x00\x01\x00\x80\x00\x00\x00\x00\x00\xff\xff\xff!\xf9\x04\x01\x00\x00\x00\x00,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02D\x01\x00")
        .unwrap();
    zip.start_file("OEBPS/toc.ncx", options).unwrap();
    zip.write_all(
        br#"<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/"><navMap>
<navPoint id="one"><navLabel><text>Chapter One</text></navLabel><content src="chapter.xhtml"/></navPoint>
</navMap></ncx>"#,
    )
    .unwrap();
    zip.start_file("OEBPS/chapter.xhtml", options).unwrap();
    zip.write_all(
        br#"<html xmlns="http://www.w3.org/1999/xhtml"><head><title>Chapter One</title>
<script>bad()</script></head><body><h1>Chapter One</h1><p>Durable sanitized book content.</p></body></html>"#,
    )
    .unwrap();
    zip.finish().unwrap().into_inner()
}

fn build_minimal_pdf(text: &str) -> Vec<u8> {
    let content = format!(
        "BT /F1 12 Tf ({}) Tj ET",
        text.replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)")
    );
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (index, object) in [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>"
            .to_string(),
        format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
    ]
    .into_iter()
    .enumerate()
    {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
    }
    let xref = pdf.len();
    pdf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
    );
    pdf
}
