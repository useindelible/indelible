use chrono::{DateTime, Utc};
use ind_domain::{EmailDestination, EmailSender, EmailSenderId, EmailSenderRenderDefault};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::{ApiError, FieldError};
use crate::extract::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RenderDefaultDto {
    Reader,
    Original,
}

impl From<EmailSenderRenderDefault> for RenderDefaultDto {
    fn from(value: EmailSenderRenderDefault) -> Self {
        match value {
            EmailSenderRenderDefault::Reader => RenderDefaultDto::Reader,
            EmailSenderRenderDefault::Original => RenderDefaultDto::Original,
        }
    }
}

impl From<RenderDefaultDto> for EmailSenderRenderDefault {
    fn from(value: RenderDefaultDto) -> Self {
        match value {
            RenderDefaultDto::Reader => EmailSenderRenderDefault::Reader,
            RenderDefaultDto::Original => EmailSenderRenderDefault::Original,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DestinationDto {
    Feed,
    Library,
}

impl From<EmailDestination> for DestinationDto {
    fn from(value: EmailDestination) -> Self {
        match value {
            EmailDestination::Feed => DestinationDto::Feed,
            EmailDestination::Library => DestinationDto::Library,
        }
    }
}

impl From<DestinationDto> for EmailDestination {
    fn from(value: DestinationDto) -> Self {
        match value {
            DestinationDto::Feed => EmailDestination::Feed,
            DestinationDto::Library => EmailDestination::Library,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EmailSenderResponse {
    pub id: String,
    pub object: &'static str,
    pub canonical_addr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub render_default: RenderDefaultDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_default: Option<DestinationDto>,
    pub blocked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub blocked_at: Option<DateTime<Utc>>,
    #[schema(value_type = String, format = DateTime)]
    pub first_seen_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub last_seen_at: DateTime<Utc>,
    pub delivery_count: i32,
}

impl EmailSenderResponse {
    pub fn from_domain(sender: EmailSender) -> Self {
        Self {
            id: sender.id.to_string(),
            object: "email_sender",
            canonical_addr: sender.canonical_addr,
            list_id: sender.list_id,
            display_name: sender.display_name,
            render_default: sender.render_default.into(),
            routing_default: sender.routing_default.map(Into::into),
            blocked: sender.blocked_at.is_some(),
            blocked_at: sender.blocked_at,
            first_seen_at: sender.first_seen_at,
            last_seen_at: sender.last_seen_at,
            delivery_count: sender.delivery_count,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListEmailSendersResponse {
    pub data: Vec<EmailSenderResponse>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UnsubscribeEmailSenderResponse {
    pub object: &'static str,
    pub sender_id: String,
    pub job_id: String,
    #[schema(value_type = String, format = DateTime)]
    pub blocked_at: DateTime<Utc>,
}

impl UnsubscribeEmailSenderResponse {
    pub fn from_outcome(
        outcome: ind_application::ports::EmailSenderUnsubscribeOutcome,
    ) -> Result<Self, ApiError> {
        let blocked_at = outcome
            .sender
            .blocked_at
            .ok_or_else(|| ApiError::Internal {
                message: "unsubscribe outcome is missing blocked_at".to_string(),
            })?;
        Ok(Self {
            object: "email_unsubscribe_action",
            sender_id: outcome.sender.id.to_string(),
            job_id: outcome.job_id.to_string(),
            blocked_at,
        })
    }
}

#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListEmailSendersParams {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEmailSenderRequest {
    #[serde(default)]
    pub blocked: Option<bool>,
    #[serde(default)]
    pub render_default: Option<RenderDefaultDto>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_nullable_destination"
    )]
    #[schema(value_type = Option<DestinationDto>, nullable)]
    pub routing_default: Option<Option<DestinationDto>>,
}

impl Validate for UpdateEmailSenderRequest {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        if self.blocked.is_none() && self.render_default.is_none() && self.routing_default.is_none()
        {
            return Err(vec![FieldError {
                field: "_".into(),
                message: "at least one field must be provided".into(),
            }]);
        }
        Ok(())
    }
}

fn deserialize_optional_nullable_destination<'de, D>(
    deserializer: D,
) -> Result<Option<Option<DestinationDto>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<DestinationDto>::deserialize(deserializer).map(Some)
}

pub(crate) fn parse_sender_id(s: &str) -> Result<EmailSenderId, ApiError> {
    s.parse().map_err(|_| ApiError::NotFound {
        entity: "EmailSender",
        id: s.to_string(),
    })
}
