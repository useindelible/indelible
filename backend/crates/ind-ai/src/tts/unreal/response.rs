use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;

use crate::tts::adapter::TtsAdapterError;
use crate::tts::http::classify_transport_error;

use super::UnrealSpeechAdapter;

impl UnrealSpeechAdapter {
    pub(super) async fn get_download(
        &self,
        url: &str,
    ) -> Result<reqwest::Response, TtsAdapterError> {
        // Unreal's asset host can close an idle pooled connection without a response. Retrying
        // these idempotent downloads is safe; the synthesis POST is intentionally excluded.
        match self.client.get(url).send().await {
            Ok(response) => Ok(response),
            Err(error) if error.is_request() || error.is_connect() => self
                .client
                .get(url)
                .send()
                .await
                .map_err(classify_transport_error),
            Err(error) => Err(classify_transport_error(error)),
        }
    }

    /// Turn a `/speech` response into raw audio bytes plus an optional
    /// `Content-Type` overriding the caller's default. Unreal can return the
    /// audio inline as binary (short texts) or as a JSON envelope carrying an
    /// `OutputUri` that points to the synthesized MP3. The adapter branches
    /// on the response content type so both shapes work transparently.
    pub(super) async fn decode_speech_response(
        &self,
        response: reqwest::Response,
    ) -> Result<UnrealSpeechAsset, TtsAdapterError> {
        let is_json = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_ascii_lowercase().starts_with("application/json"))
            .unwrap_or(false);
        if is_json {
            let envelope: UnrealSpeechEnvelope = response.json().await.map_err(|e| {
                TtsAdapterError::MalformedResponse(format!(
                    "failed to decode unreal speech envelope: {e}"
                ))
            })?;
            let output_uri = envelope.output_uri.ok_or_else(|| {
                TtsAdapterError::MalformedResponse(
                    "unreal speech envelope missing OutputUri".into(),
                )
            })?;
            let asset = self.get_download(&output_uri).await?;
            let status = asset.status();
            if !status.is_success() {
                let body = asset.text().await.unwrap_or_default();
                return Err(TtsAdapterError::ProviderError {
                    status_code: status.as_u16(),
                    message: format!("unreal OutputUri download failed: {body}"),
                });
            }
            let asset_content_type = asset
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
                .or(envelope.content_type);
            let bytes = asset.bytes().await.map_err(|e| {
                TtsAdapterError::MalformedResponse(format!(
                    "failed to read unreal OutputUri body: {e}"
                ))
            })?;
            Ok(UnrealSpeechAsset {
                bytes: bytes.to_vec(),
                content_type: asset_content_type,
                timestamps_uri: envelope.timestamps_uri,
            })
        } else {
            let response_content_type = response
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);
            let bytes = response.bytes().await.map_err(|e| {
                TtsAdapterError::MalformedResponse(format!(
                    "failed to read unreal speech body: {e}"
                ))
            })?;
            Ok(UnrealSpeechAsset {
                bytes: bytes.to_vec(),
                content_type: response_content_type,
                timestamps_uri: None,
            })
        }
    }
}

#[derive(Debug)]
pub(super) struct UnrealSpeechAsset {
    pub(super) bytes: Vec<u8>,
    pub(super) content_type: Option<String>,
    pub(super) timestamps_uri: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UnrealSpeechEnvelope {
    #[serde(default)]
    output_uri: Option<String>,
    #[serde(default)]
    content_type: Option<String>,
    #[serde(default, rename = "TimestampsUri")]
    timestamps_uri: Option<String>,
}

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    use super::*;

    async fn read_request(stream: &mut TcpStream) {
        let mut request = Vec::new();
        let mut buffer = [0_u8; 1024];
        while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            let read = stream.read(&mut buffer).await.unwrap();
            assert!(read > 0, "client closed before sending the request");
            request.extend_from_slice(&buffer[..read]);
        }
    }

    async fn write_ok(stream: &mut TcpStream) {
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: keep-alive\r\n\r\nok")
            .await
            .unwrap();
        stream.flush().await.unwrap();
    }

    #[tokio::test]
    async fn download_retries_after_a_pooled_connection_returns_no_response() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut pooled, _) = listener.accept().await.unwrap();
            read_request(&mut pooled).await;
            write_ok(&mut pooled).await;

            tokio::select! {
                _ = read_request(&mut pooled) => {}
                accepted = listener.accept() => {
                    let (mut fresh, _) = accepted.unwrap();
                    read_request(&mut fresh).await;
                }
            }
            drop(pooled);

            let (mut retry, _) = listener.accept().await.unwrap();
            read_request(&mut retry).await;
            write_ok(&mut retry).await;
        });

        let adapter = UnrealSpeechAdapter::new().unwrap();
        let url = format!("http://{address}/asset.mp3");
        let warm = adapter.client.get(&url).send().await.unwrap();
        assert_eq!(warm.bytes().await.unwrap(), "ok");

        let response = adapter.get_download(&url).await.unwrap();
        assert_eq!(response.bytes().await.unwrap(), "ok");
        server.await.unwrap();
    }
}
