use serde::{Deserialize, Serialize};

use crate::DomainError;
use crate::id::{DocumentId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TtsElementKind {
    Title,
    Heading,
    Paragraph,
    Blockquote,
    ListItem,
    Code,
    Caption,
}

#[derive(Debug, Clone)]
pub struct TtsSpokenElement {
    pub element_index: i32,
    pub kind: TtsElementKind,
    pub text: String,
    pub char_start: i32,
    pub char_end: i32,
    pub chunk_id: String,
}

#[derive(Debug, Clone)]
pub struct TtsChunkHint {
    pub chunk_id: String,
    pub start_element_index: i32,
    pub end_element_index: i32,
}

#[derive(Debug, Clone)]
pub struct TtsElementFeed {
    pub document_id: DocumentId,
    pub title: String,
    pub chunk_hints: Vec<TtsChunkHint>,
    pub elements: Vec<TtsSpokenElement>,
}

#[async_trait::async_trait]
pub trait TtsElementSource: Send + Sync {
    async fn elements(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> Result<TtsElementFeed, DomainError>;
}
