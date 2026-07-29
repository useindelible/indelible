use bytes::Bytes;
use ind_application::repos::document_asset::DocumentAssetRepository;
use ind_domain::{ArchiveAssetKind, ArchiveAssetStatus, NewDocumentAsset};
use ind_persistence::repos::PgDocumentAssetRepository;
use ind_test_support::{DocumentFactory, spawn_app};
use reqwest::StatusCode;
use serde_json::json;

use super::common::{assert_json_response as response, assert_status};

#[tokio::test]
async fn tts_journey_synthesizes_scoped_audio_and_persists_playback() {
    let app = spawn_app().await;
    let owner = app.create_web_session().await;
    let other = app.create_web_session().await;
    let client = app.authed_client(&owner);
    let document = DocumentFactory::new(owner.user.id)
        .with_title("TTS Journey")
        .insert(app.pool())
        .await;
    let upload = app
        .storage()
        .await
        .upload(
            &format!("tests/tts/{}/readable.html", document.id),
            "text/html",
            Bytes::from("<article><h1>One</h1><p>Alpha.</p><h2>Two</h2><p>Beta.</p></article>"),
        )
        .await
        .unwrap();
    PgDocumentAssetRepository::new(app.pool().clone())
        .upsert_document_asset(NewDocumentAsset {
            document_id: document.id,
            asset_kind: ArchiveAssetKind::ReadableHtml,
            s3_key: upload.key,
            s3_bucket: upload.bucket,
            content_type: "text/html".into(),
            size_bytes: upload.size_bytes,
            status: ArchiveAssetStatus::Completed,
            failed_reason: None,
        })
        .await
        .unwrap();

    let persona = response(
        client
            .post_json(
                "/api/v1/tts/voice-personas",
                &json!({"display_name": "Narrator", "provider": "mock"}),
            )
            .await,
        StatusCode::CREATED,
    )
    .await;
    let persona = persona["id"].as_str().unwrap();
    let personas = response(
        client.get("/api/v1/tts/voice-personas").await,
        StatusCode::OK,
    )
    .await;
    let personas = personas["personas"].as_array().unwrap();
    assert!(personas.iter().any(|row| row["id"] == persona));
    assert!(
        personas.iter().any(|row| {
            row["provider"] == "unreal_speech" && row["provider_voice_id"] == "Sierra"
        })
    );
    let manifest = response(
        client
            .post_json(
                &format!("/api/v1/documents/{}/tts/sessions", document.id),
                &json!({
                    "voice_persona_id": persona, "generation_scope": "section",
                    "audio_format": "mp3", "sample_rate": 24000
                }),
            )
            .await,
        StatusCode::OK,
    )
    .await;
    let session = manifest["session"]["id"].as_str().unwrap();
    let chunk = &manifest["chunks"][0];
    assert_eq!(chunk["timing_source"], "provider_transcript");
    let chunk_id = chunk["chunk_id"].as_str().unwrap();
    let element = chunk["timings"][0]["element_index"].as_i64().unwrap();
    let chunk_path = format!(
        "/api/v1/documents/{}/tts/chunks/{chunk_id}?session_id={session}",
        document.id
    );
    assert_eq!(client.get(&chunk_path).await.status(), StatusCode::OK);
    assert_status(
        app.authed_client(&other).get(&chunk_path).await,
        StatusCode::NOT_FOUND,
    )
    .await;
    let timestamp = response(
        client.get(&format!(
            "/api/v1/documents/{}/tts/timestamp?session_id={session}&chunk_id={chunk_id}&element_index={element}",
            document.id
        )).await,
        StatusCode::OK,
    ).await;
    assert_eq!(timestamp["element_index"], element);
    assert!(
        !client
            .get(chunk["audio_url"].as_str().unwrap())
            .await
            .bytes()
            .await
            .unwrap()
            .is_empty()
    );

    let playback_path = format!("/api/v1/documents/{}/playback", document.id);
    assert_status(
        client
            .patch_json(
                &playback_path,
                &json!({
                    "playback_kind": "tts", "position_seconds": 42.5, "playback_speed": 1.25,
                    "element_index": element, "tts_chunk_id": chunk_id,
                    "tts_voice_persona_id": persona, "is_playing": false
                }),
            )
            .await,
        StatusCode::NO_CONTENT,
    )
    .await;
    let playback = response(
        client.get(&format!("{playback_path}?kind=tts")).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(playback["position_seconds"], 42.5);
    assert_eq!(playback["tts_chunk_id"], chunk_id);
}
