use std::sync::Arc;

use chrono::Utc;
use futures::StreamExt;
use futures::future::BoxFuture;
use futures::stream;

use ind_application::AppError;
use ind_domain::{MessageRole, MilaMessage, MilaMessageId, MilaSessionId, UserId};
use uuid::Uuid;

use crate::ChatCompletionStream;
use crate::content::map_ai_error;

use super::{ChatSessions, MilaChatDelta, MilaChatStream};

pub(super) struct ChatStreamState {
    pub(super) upstream: ChatCompletionStream,
    pub(super) session_repo: Arc<dyn ChatSessions>,
    pub(super) user_id: UserId,
    pub(super) session_id: MilaSessionId,
    pub(super) assistant_text: String,
    pub(super) source_chunk_ids: Vec<Uuid>,
    pub(super) source_label_count: usize,
    pub(super) pending_warning: Option<String>,
    pub(super) finished: bool,
}

pub(super) fn wrap_stream(state: ChatStreamState) -> MilaChatStream {
    stream::unfold(state, |mut state| async move {
        if state.finished {
            return None;
        }

        if let Some(retrieval_degraded) = state.pending_warning.take() {
            return Some((
                Ok(MilaChatDelta {
                    content: String::new(),
                    retrieval_degraded: Some(retrieval_degraded),
                }),
                state,
            ));
        }

        loop {
            match state.upstream.next().await {
                Some(Ok(chunk)) => {
                    let delta = extract_delta_content(&chunk);
                    if !delta.is_empty() {
                        state.assistant_text.push_str(&delta);
                        return Some((
                            Ok(MilaChatDelta {
                                content: delta,
                                retrieval_degraded: None,
                            }),
                            state,
                        ));
                    }
                }
                Some(Err(err)) => {
                    state.finished = true;
                    return Some((Err(map_ai_error(err)), state));
                }
                None => {
                    let result = persist_completed_turn(
                        Arc::clone(&state.session_repo),
                        state.user_id,
                        state.session_id,
                        sanitize_source_tokens(&state.assistant_text, state.source_label_count),
                        state.source_chunk_ids.clone(),
                    )
                    .await;
                    state.finished = true;

                    match result {
                        Ok(()) => return None,
                        Err(err) => return Some((Err(err), state)),
                    }
                }
            }
        }
    })
    .boxed()
}

fn persist_completed_turn(
    session_repo: Arc<dyn ChatSessions>,
    user_id: UserId,
    session_id: MilaSessionId,
    assistant_text: String,
    source_chunk_ids: Vec<Uuid>,
) -> BoxFuture<'static, Result<(), AppError>> {
    Box::pin(async move {
        let assistant_message = MilaMessage {
            id: MilaMessageId::new(),
            session_id,
            role: MessageRole::Assistant,
            content: assistant_text,
            source_chunks: source_chunk_ids,
            created_at: Utc::now(),
        };
        session_repo
            .insert_message(user_id, &assistant_message)
            .await?;
        session_repo
            .touch_session(session_id, user_id, assistant_message.created_at)
            .await
    })
}

fn extract_delta_content(chunk: &crate::ChatCompletionChunk) -> String {
    chunk
        .choices
        .iter()
        .filter_map(|choice| choice.delta.content.as_deref())
        .collect::<Vec<_>>()
        .join("")
}

/// Drop `[S<n>]` tokens whose label was never offered to the model (`n` outside
/// `1..=max_label`), along with placeholder forms such as `[S_]`, `[Sn]` and `[S]`. Models
/// occasionally invent labels or reach for a placeholder when asked to cite without being
/// given any; persisting either would surface raw tokens in every client, since source-ref
/// resolution only knows the offered labels.
pub(super) fn sanitize_source_tokens(content: &str, max_label: usize) -> String {
    let mut result = String::with_capacity(content.len());
    let mut last_end = 0;

    for (start, end, label) in source_tokens(content) {
        if label >= 1 && label <= max_label {
            continue;
        }
        let prefix = &content[last_end..start];
        result.push_str(prefix.trim_end_matches(' '));
        last_end = end;
    }

    result.push_str(&content[last_end..]);
    result
}

/// Byte ranges and numeric labels of source tokens: `[S<digits>]` plus the placeholder forms
/// `[S]`, `[S_]` and `[Sn]`. Placeholders report label 0, which is never inside `1..=max_label`,
/// so they are always dropped. Delimiters are ASCII, so the returned indices are valid UTF-8
/// boundaries.
fn source_tokens(content: &str) -> Vec<(usize, usize, usize)> {
    let bytes = content.as_bytes();
    let mut tokens = Vec::new();
    let mut cursor = 0;

    // The shortest token is the 3-byte `[S]`.
    while cursor + 2 < bytes.len() {
        if bytes[cursor] != b'[' || bytes[cursor + 1] != b'S' {
            cursor += 1;
            continue;
        }

        let label_start = cursor + 2;
        let mut label_end = label_start;
        while label_end < bytes.len() && is_source_label_byte(bytes[label_end]) {
            label_end += 1;
        }

        if label_end >= bytes.len() || bytes[label_end] != b']' {
            cursor += 1;
            continue;
        }

        let label = content[label_start..label_end]
            .parse::<usize>()
            .unwrap_or(0);
        tokens.push((cursor, label_end + 1, label));
        cursor = label_end + 1;
    }

    tokens
}

/// Digits form real labels; `_` and `n` are the placeholders models substitute for one.
fn is_source_label_byte(byte: u8) -> bool {
    byte.is_ascii_digit() || byte == b'_' || byte == b'n' || byte == b'N'
}
