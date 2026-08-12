#![allow(clippy::unwrap_used, clippy::expect_used)]

use futures::TryStreamExt;
use ind_application::error::AppError;
use ind_domain::{DocumentType, DomainError, YoutubeIngestDocumentJob};
use ind_test_support::{DocumentFactory, TestDb, UserFactory};
use ind_worker::jobs::youtube::handle_youtube_ingest_document;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

type EnrichedYoutubeDocument = (
    Option<String>,
    Option<String>,
    Option<i32>,
    Option<String>,
    Option<String>,
);

mod common;

#[tokio::test]
async fn youtube_ingest_enriches_owned_document_assets_metadata_and_index_handoff() {
    let db = TestDb::new().await;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/youtubei/v1/player"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "videoDetails": {
                "title": "Boundary Video",
                "author": "Boundary Channel",
                "shortDescription": "A transcript-enriched product boundary.",
                "lengthSeconds": "212",
                "viewCount": "1200",
                "thumbnail": {"thumbnails": [
                    {"url": "https://img.example/small.jpg", "width": 120},
                    {"url": "https://img.example/large.jpg", "width": 1280}
                ]}
            },
            "captions": {"playerCaptionsTracklistRenderer": {"captionTracks": [{
                "baseUrl": format!("{}/api/timedtext", server.uri()),
                "vssId": ".en"
            }]}}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/timedtext"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"events":[{"tStartMs":0,"dDurationMs":2000,"segs":[{"utf8":"Surgical transcript content."}]}]}"#,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let owner = UserFactory::new().insert(db.pool()).await;
    let stranger = UserFactory::new().insert(db.pool()).await;
    let document = DocumentFactory::new(owner.id)
        .with_document_type(DocumentType::Video)
        .with_title("Placeholder")
        .insert(db.pool())
        .await;
    let mut worker = common::build_worker_ctx(&db).await;
    worker.youtube_player_base_url = Some(server.uri());
    let deps = worker.capture_jobs();
    let job = |user_id| YoutubeIngestDocumentJob {
        document_id: document.id,
        user_id,
        url: "https://www.youtube.com/watch?v=boundary123".into(),
    };

    let ownership_error = handle_youtube_ingest_document(&deps, job(stranger.id))
        .await
        .expect_err("another tenant must not enrich the document");
    assert!(ownership_error.to_string().contains("Document"));
    handle_youtube_ingest_document(&deps, job(owner.id))
        .await
        .unwrap();

    let assets: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT asset_kind, status, s3_key FROM archive_assets \
         WHERE document_id = $1 ORDER BY asset_kind",
    )
    .bind(document.id.into_uuid())
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert_eq!(assets.len(), 2);
    assert!(assets.iter().all(|asset| asset.1 == "completed"));
    assert!(assets.iter().any(|asset| asset.0 == "readable_html"));
    assert!(assets.iter().any(|asset| asset.0 == "extracted_text"));
    let storage = db.storage().await;
    let transcript_key = &assets
        .iter()
        .find(|asset| asset.0 == "extracted_text")
        .unwrap()
        .2;
    let transcript = storage.get_object(transcript_key).await.unwrap();
    let chunks = transcript.body.try_collect::<Vec<_>>().await.unwrap();
    assert_eq!(chunks.concat(), b"Surgical transcript content.");

    let enriched: EnrichedYoutubeDocument = sqlx::query_as(
        "SELECT d.title, d.lead_image_url, v.duration_seconds, v.channel_name, d.excerpt \
             FROM documents d JOIN document_video_metadata v ON v.document_id = d.id \
             WHERE d.id = $1",
    )
    .bind(document.id.into_uuid())
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(enriched.0.as_deref(), Some("Boundary Video"));
    assert_eq!(enriched.1.as_deref(), Some("https://img.example/large.jpg"));
    assert_eq!(enriched.2, Some(212));
    assert_eq!(enriched.3.as_deref(), Some("Boundary Channel"));
    assert_eq!(
        enriched.4.as_deref(),
        Some("A transcript-enriched product boundary.")
    );
    let reindex: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM job_outbox WHERE job_type = 'search.reindex_document' \
         AND payload->>'document_id' = $1",
    )
    .bind(document.id.to_string())
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(reindex, 1);
}

#[tokio::test]
async fn unavailable_youtube_video_records_terminal_readable_failure() {
    let db = TestDb::new().await;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/youtubei/v1/player"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "playabilityStatus": {
                "status": "ERROR",
                "reason": "Video unavailable"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let owner = UserFactory::new().insert(db.pool()).await;
    let document = DocumentFactory::new(owner.id)
        .with_document_type(DocumentType::Video)
        .with_title("Placeholder")
        .insert(db.pool())
        .await;
    let mut worker = common::build_worker_ctx(&db).await;
    worker.youtube_player_base_url = Some(server.uri());

    let error = handle_youtube_ingest_document(
        &worker.capture_jobs(),
        YoutubeIngestDocumentJob {
            document_id: document.id,
            user_id: owner.id,
            url: "https://www.youtube.com/watch?v=deleted123".into(),
        },
    )
    .await
    .expect_err("unavailable videos must terminate");
    assert!(matches!(
        error,
        AppError::Domain(DomainError::NotFound {
            entity: "YouTubeVideo",
            ..
        })
    ));

    let assets: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT asset_kind, status, s3_key, failed_reason FROM archive_assets \
         WHERE document_id = $1 ORDER BY asset_kind",
    )
    .bind(document.id.into_uuid())
    .fetch_all(db.pool())
    .await
    .unwrap();
    assert_eq!(
        assets,
        vec![(
            "readable_html".into(),
            "failed".into(),
            String::new(),
            Some("This YouTube video is unavailable, private, or deleted.".into()),
        )]
    );

    let readable_key = format!(
        "documents/{}/{}/readable.html",
        owner.id.into_uuid(),
        document.id.into_uuid()
    );
    assert!(!db.storage().await.exists(&readable_key).await.unwrap());
    let enrichment: i64 =
        sqlx::query_scalar("SELECT count(*) FROM document_video_metadata WHERE document_id = $1")
            .bind(document.id.into_uuid())
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(enrichment, 0);
    let reindex: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM job_outbox WHERE job_type = 'search.reindex_document' \
         AND payload->>'document_id' = $1",
    )
    .bind(document.id.to_string())
    .fetch_one(db.pool())
    .await
    .unwrap();
    assert_eq!(reindex, 0);
}

#[tokio::test]
async fn missing_youtube_details_without_terminal_status_remains_retryable() {
    let db = TestDb::new().await;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/youtubei/v1/player"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "playabilityStatus": {"status": "OK"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let owner = UserFactory::new().insert(db.pool()).await;
    let document = DocumentFactory::new(owner.id)
        .with_document_type(DocumentType::Video)
        .insert(db.pool())
        .await;
    let mut worker = common::build_worker_ctx(&db).await;
    worker.youtube_player_base_url = Some(server.uri());

    let error = handle_youtube_ingest_document(
        &worker.capture_jobs(),
        YoutubeIngestDocumentJob {
            document_id: document.id,
            user_id: owner.id,
            url: "https://www.youtube.com/watch?v=transient123".into(),
        },
    )
    .await
    .expect_err("missing details without a terminal status must retry");
    assert!(matches!(
        error,
        AppError::ExternalService { ref service, .. } if service == "youtube"
    ));

    let assets: i64 =
        sqlx::query_scalar("SELECT count(*) FROM archive_assets WHERE document_id = $1")
            .bind(document.id.into_uuid())
            .fetch_one(db.pool())
            .await
            .unwrap();
    assert_eq!(assets, 0);
}
