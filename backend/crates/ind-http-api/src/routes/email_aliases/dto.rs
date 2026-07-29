use chrono::{DateTime, Utc};
use ind_domain::{
    EmailAlias, EmailAliasId, EmailAliasStatus, EmailDestination, validate_local_part,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{ApiError, FieldError};
use crate::extract::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AliasDestinationDto {
    Feed,
    Library,
}

impl From<EmailDestination> for AliasDestinationDto {
    fn from(value: EmailDestination) -> Self {
        match value {
            EmailDestination::Feed => AliasDestinationDto::Feed,
            EmailDestination::Library => AliasDestinationDto::Library,
        }
    }
}

impl From<AliasDestinationDto> for EmailDestination {
    fn from(value: AliasDestinationDto) -> Self {
        match value {
            AliasDestinationDto::Feed => EmailDestination::Feed,
            AliasDestinationDto::Library => EmailDestination::Library,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AliasStatusDto {
    Active,
    Retired,
}

impl From<EmailAliasStatus> for AliasStatusDto {
    fn from(value: EmailAliasStatus) -> Self {
        match value {
            EmailAliasStatus::Active => AliasStatusDto::Active,
            EmailAliasStatus::Retired => AliasStatusDto::Retired,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EmailAliasResponse {
    pub id: String,
    pub object: &'static str,
    pub destination: AliasDestinationDto,
    pub local_part: String,
    pub status: AliasStatusDto,
    pub is_default: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub retire_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub retired_at: Option<DateTime<Utc>>,
}

impl EmailAliasResponse {
    pub fn from_domain(
        alias: EmailAlias,
        feed_domain: Option<&str>,
        library_domain: Option<&str>,
    ) -> Self {
        let domain = match alias.destination {
            EmailDestination::Feed => feed_domain,
            EmailDestination::Library => library_domain,
        };
        let address = domain.map(|d| format!("{}@{d}", alias.local_part));

        Self {
            id: alias.id.to_string(),
            object: "email_alias",
            destination: alias.destination.into(),
            local_part: alias.local_part,
            status: alias.status.into(),
            is_default: alias.is_default,
            address,
            created_at: alias.created_at,
            retire_at: alias.retire_at,
            retired_at: alias.retired_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListEmailAliasesResponse {
    pub data: Vec<EmailAliasResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEmailAliasRequest {
    pub destination: AliasDestinationDto,
    pub local_part: String,
    #[serde(default)]
    pub is_default: bool,
}

impl Validate for CreateEmailAliasRequest {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        if let Err(err) = validate_local_part(&self.local_part) {
            return Err(vec![FieldError {
                field: "local_part".into(),
                message: err.to_string(),
            }]);
        }
        Ok(())
    }
}

pub(crate) fn parse_alias_id(s: &str) -> Result<EmailAliasId, ApiError> {
    s.parse().map_err(|_| ApiError::NotFound {
        entity: "EmailAlias",
        id: s.to_string(),
    })
}
