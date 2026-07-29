use std::collections::HashMap;
use std::sync::Arc;

use ind_domain::{AiOutput, AiOutputType, DocumentId};

use crate::AppError;
use crate::repos::ai_output::AiOutputRepository;

#[async_trait::async_trait]
pub trait ExportSummaryProvider: Send + Sync {
    async fn summary_for_document(
        &self,
        document_id: DocumentId,
        excerpt: Option<&str>,
    ) -> Result<Option<String>, AppError>;

    async fn summaries_for_documents(
        &self,
        sources: &[DocumentSummarySource],
    ) -> Result<HashMap<DocumentId, Option<String>>, AppError> {
        let mut summaries = HashMap::with_capacity(sources.len());
        for source in sources {
            let summary = self
                .summary_for_document(source.document_id, source.excerpt.as_deref())
                .await?;
            summaries.insert(source.document_id, summary);
        }
        Ok(summaries)
    }
}

#[derive(Debug, Clone)]
pub struct DocumentSummarySource {
    pub document_id: DocumentId,
    pub excerpt: Option<String>,
}

pub struct StoredExportSummaryProvider {
    output_repo: Arc<dyn AiOutputRepository>,
}

impl StoredExportSummaryProvider {
    pub fn new(output_repo: Arc<dyn AiOutputRepository>) -> Self {
        Self { output_repo }
    }
}

#[async_trait::async_trait]
impl ExportSummaryProvider for StoredExportSummaryProvider {
    async fn summary_for_document(
        &self,
        document_id: DocumentId,
        excerpt: Option<&str>,
    ) -> Result<Option<String>, AppError> {
        summary_from_sources(self.output_repo.as_ref(), document_id, excerpt).await
    }

    async fn summaries_for_documents(
        &self,
        sources: &[DocumentSummarySource],
    ) -> Result<HashMap<DocumentId, Option<String>>, AppError> {
        let document_ids: Vec<_> = sources.iter().map(|source| source.document_id).collect();
        let outputs = self
            .output_repo
            .list_for_documents(&document_ids, Some(AiOutputType::Summary))
            .await?;
        let mut outputs_by_document = HashMap::<DocumentId, Vec<AiOutput>>::new();
        for output in outputs {
            if let Some(document_id) = output.document_id {
                outputs_by_document
                    .entry(document_id)
                    .or_default()
                    .push(output);
            }
        }

        let mut summaries = HashMap::with_capacity(sources.len());
        for source in sources {
            let summary = outputs_by_document
                .get(&source.document_id)
                .and_then(|outputs| outputs.iter().find_map(summary_from_output))
                .or_else(|| normalized_summary(source.excerpt.as_deref()));
            summaries.insert(source.document_id, summary);
        }
        Ok(summaries)
    }
}

async fn summary_from_sources(
    output_repo: &dyn AiOutputRepository,
    document_id: DocumentId,
    excerpt: Option<&str>,
) -> Result<Option<String>, AppError> {
    let summary = output_repo
        .list_for_document(document_id, Some(AiOutputType::Summary))
        .await?
        .iter()
        .find_map(summary_from_output);

    Ok(summary.or_else(|| normalized_summary(excerpt)))
}

fn summary_from_output(output: &AiOutput) -> Option<String> {
    match &output.content {
        serde_json::Value::String(value) => normalized_summary(Some(value)),
        serde_json::Value::Object(map) => ["summary", "text", "content"]
            .iter()
            .filter_map(|key| map.get(*key))
            .filter_map(serde_json::Value::as_str)
            .find_map(|value| normalized_summary(Some(value))),
        _ => None,
    }
}

fn normalized_summary(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
