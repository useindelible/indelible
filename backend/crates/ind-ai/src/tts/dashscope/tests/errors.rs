use super::super::*;
use super::support::{sample_persona, synth_request};
use crate::tts::http::{classify_status_error, parse_retry_after_ms};
use reqwest::StatusCode;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn provider_statuses_share_one_error_classification_contract() {
    for (status, retry, body, expected) in [
        (StatusCode::UNAUTHORIZED, None, "bad key", "auth"),
        (StatusCode::FORBIDDEN, None, "denied", "auth"),
        (
            StatusCode::FORBIDDEN,
            None,
            r#"{"code":"Throttling.RateQuota"}"#,
            "quota",
        ),
        (StatusCode::PAYMENT_REQUIRED, None, "credit", "quota"),
        (StatusCode::TOO_MANY_REQUESTS, Some(7000), "slow", "rate"),
        (StatusCode::BAD_REQUEST, None, "voice", "invalid"),
        (StatusCode::GATEWAY_TIMEOUT, None, "timeout", "timeout"),
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            None,
            "failed",
            "provider",
        ),
    ] {
        let error = classify_status_error(status, retry, body.into());
        assert!(match expected {
            "auth" => matches!(error, TtsAdapterError::AuthenticationFailed(_)),
            "quota" => matches!(error, TtsAdapterError::QuotaExhausted),
            "rate" => matches!(
                error,
                TtsAdapterError::RateLimited {
                    retry_after_ms: Some(7000)
                }
            ),
            "invalid" => matches!(error, TtsAdapterError::InvalidRequest(_)),
            "timeout" => matches!(error, TtsAdapterError::Timeout),
            "provider" => matches!(
                error,
                TtsAdapterError::ProviderError {
                    status_code: 500,
                    ..
                }
            ),
            _ => unreachable!(),
        });
    }
    assert_eq!(parse_retry_after_ms(Some("7")), Some(7000));
    assert_eq!(parse_retry_after_ms(Some("invalid")), None);
}

#[tokio::test]
async fn dashscope_rejects_missing_audio_and_api_key() {
    let adapter = DashScopeAdapter::new().unwrap();
    let persona = sample_persona();
    let mut request = synth_request(&persona, "hi");
    request.api_key = None;
    assert!(matches!(
        adapter.synthesize(request).await,
        Err(TtsAdapterError::AuthenticationFailed(_))
    ));

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "output": {"audio": {"data": ""}}
        })))
        .mount(&server)
        .await;
    let mut request = synth_request(&persona, "hi");
    let uri = server.uri();
    request.api_base = Some(&uri);
    assert!(matches!(
        adapter.synthesize(request).await,
        Err(TtsAdapterError::MalformedResponse(message)) if message.contains("neither data nor url")
    ));
}

#[tokio::test]
async fn dashscope_surfaces_signed_download_failures() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/signed/broken.mp3"))
        .respond_with(ResponseTemplate::new(500).set_body_string("signed url expired"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "output": {"audio": {"url": format!("{}/signed/broken.mp3", server.uri())}}
        })))
        .mount(&server)
        .await;

    let adapter = DashScopeAdapter::new().unwrap();
    let persona = sample_persona();
    let mut request = synth_request(&persona, "hi");
    let uri = server.uri();
    request.api_base = Some(&uri);
    assert!(matches!(
        adapter.synthesize(request).await,
        Err(TtsAdapterError::ProviderError {
            status_code: 500,
            ..
        })
    ));
}
