use crate::error::AppError;
use ind_domain::{DomainError, UserId};

use super::{FeedService, OpmlImportResult, SubscribeInput};

impl FeedService {
    pub async fn import_opml(
        &self,
        user_id: UserId,
        opml_xml: &str,
    ) -> Result<OpmlImportResult, AppError> {
        let urls = self.opml_parser.parse_feed_urls(opml_xml).map_err(|e| {
            AppError::Domain(DomainError::Validation {
                field: "opml".into(),
                message: e.to_string(),
            })
        })?;

        let mut created = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();

        for url in urls {
            let input = SubscribeInput {
                url: url.clone(),
                title_override: None,
                poll_interval_override_minutes: None,
            };
            match self.subscribe(user_id, input).await {
                Ok(_) => created += 1,
                Err(AppError::Domain(DomainError::Conflict { .. })) => skipped += 1,
                Err(e) => errors.push(format!("{url}: {e}")),
            }
        }

        Ok(OpmlImportResult {
            created,
            skipped,
            errors,
        })
    }
}
