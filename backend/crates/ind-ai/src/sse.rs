use bytes::BytesMut;

use crate::error::AiError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsedSseEvent {
    Json(String),
    Done,
}

#[derive(Debug, Default)]
pub(crate) struct SseDecoder {
    buffer: BytesMut,
    data_lines: Vec<String>,
}

impl SseDecoder {
    pub(crate) fn push(&mut self, chunk: &[u8]) -> Result<Vec<ParsedSseEvent>, AiError> {
        self.buffer.extend_from_slice(chunk);
        let mut events = Vec::new();

        while let Some(position) = self.buffer.iter().position(|byte| *byte == b'\n') {
            let mut line = self.buffer.split_to(position + 1);
            line.truncate(position);

            if line.last() == Some(&b'\r') {
                line.truncate(line.len().saturating_sub(1));
            }

            let line = std::str::from_utf8(&line).map_err(AiError::from_decode)?;
            if line.is_empty() {
                if let Some(event) = self.finish_event() {
                    events.push(event);
                }
                continue;
            }

            if line.starts_with(':') {
                continue;
            }

            if let Some(data) = line.strip_prefix("data:") {
                let data = data.strip_prefix(' ').unwrap_or(data);
                self.data_lines.push(data.to_string());
            }
        }

        Ok(events)
    }

    fn finish_event(&mut self) -> Option<ParsedSseEvent> {
        if self.data_lines.is_empty() {
            return None;
        }

        let payload = std::mem::take(&mut self.data_lines).join("\n");
        if payload == "[DONE]" {
            Some(ParsedSseEvent::Done)
        } else {
            Some(ParsedSseEvent::Json(payload))
        }
    }
}
