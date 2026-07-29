use ind_application::error::AppError;
use ind_domain::{DomainError, IntegrationOAuthProvider};

use crate::context::NotionJobDeps;

pub(super) async fn load_notion_access_token(
    deps: &NotionJobDeps,
    user_id: ind_domain::UserId,
) -> Result<String, AppError> {
    let Some(token_row) = deps
        .oauth_token_repo
        .find_by_user_provider(user_id, IntegrationOAuthProvider::Notion)
        .await?
    else {
        return Err(AppError::Domain(DomainError::NotFound {
            entity: "IntegrationOAuthToken",
            id: "notion".into(),
        }));
    };

    String::from_utf8(deps.cipher.open(&token_row.access_token_enc).map_err(|e| {
        AppError::ExternalService {
            service: "credential_cipher".into(),
            message: e.to_string(),
        }
    })?)
    .map_err(|e| AppError::ExternalService {
        service: "credential_cipher".into(),
        message: e.to_string(),
    })
}
