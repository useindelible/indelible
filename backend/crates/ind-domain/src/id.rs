use std::fmt;
use std::str::FromStr;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum IdParseError {
    #[error("invalid prefix: expected `{expected}`, found `{found}`")]
    InvalidPrefix { expected: String, found: String },

    #[error("missing prefix: expected `{expected}`")]
    MissingPrefix { expected: String },

    #[error("invalid UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),
}

macro_rules! impl_id_common {
    ($name:ident) => {
        impl $name {
            #[allow(clippy::new_without_default)]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            pub fn into_uuid(self) -> Uuid {
                self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = Uuid;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

macro_rules! prefixed_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(Uuid);

        impl_id_common!($name);

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({}{})", stringify!($name), $prefix, self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}{}", $prefix, self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.strip_prefix($prefix) {
                    Some(rest) => {
                        let uuid = Uuid::parse_str(rest)?;
                        Ok(Self(uuid))
                    }
                    None => {
                        if s.len() < $prefix.len() {
                            Err(IdParseError::MissingPrefix {
                                expected: $prefix.to_owned(),
                            })
                        } else {
                            Err(IdParseError::InvalidPrefix {
                                expected: $prefix.to_owned(),
                                found: s.chars().take($prefix.len()).collect(),
                            })
                        }
                    }
                }
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_str(self)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct IdVisitor;

                impl Visitor<'_> for IdVisitor {
                    type Value = $name;

                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        write!(f, "a string with prefix `{}`", $prefix)
                    }

                    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                        v.parse().map_err(de::Error::custom)
                    }
                }

                deserializer.deserialize_str(IdVisitor)
            }
        }
    };
}

macro_rules! internal_id {
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl_id_common!($name);

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let uuid = Uuid::parse_str(s)?;
                Ok(Self(uuid))
            }
        }
    };
}

// -- Prefixed IDs (API-facing) --

prefixed_id!(UserId, "usr_");
prefixed_id!(ItemId, "itm_");
prefixed_id!(CollectionId, "col_");
prefixed_id!(TagId, "tag_");
prefixed_id!(FeedSourceId, "fso_");
prefixed_id!(FeedSourceEntryId, "fse_");
prefixed_id!(FeedSubscriptionId, "fed_");
prefixed_id!(HighlightId, "hlt_");
prefixed_id!(ApiTokenId, "tok_");
prefixed_id!(SubscriptionId, "sub_");
prefixed_id!(BillingAccountId, "bil_");
prefixed_id!(JobOutboxId, "job_");
prefixed_id!(WebhookEndpointId, "whk_");
prefixed_id!(EntityId, "ent_");
prefixed_id!(SmartListId, "sml_");
prefixed_id!(MilaSessionId, "thr_");
prefixed_id!(MilaMessageId, "msg_");
prefixed_id!(IntegrationConnectionId, "int_");
prefixed_id!(ImportJobId, "imp_");
prefixed_id!(TtsVoicePersonaId, "vper_");
prefixed_id!(TtsChunkRecordId, "tch_");
prefixed_id!(TtsAudioAssetId, "taa_");
prefixed_id!(TtsSessionId, "tss_");
prefixed_id!(EmailSenderId, "snd_");
prefixed_id!(EmailAliasId, "als_");
prefixed_id!(DocumentId, "doc_");
prefixed_id!(LibraryEntryId, "lib_");
prefixed_id!(FeedDeliveryId, "dlv_");
prefixed_id!(ReadingEventId, "rev_");
prefixed_id!(ClientId, "cli_");

// -- Internal IDs (no API prefix) --

internal_id!(OAuthIdentityId);
internal_id!(RefreshTokenId);
internal_id!(AuthorizationCodeId);
internal_id!(PlanId);
internal_id!(ArchiveAssetId);
internal_id!(ItemNoteId);
internal_id!(HighlightNoteId);
internal_id!(TagAliasId);
internal_id!(AiPromptPresetId);
internal_id!(AiRunId);
internal_id!(AiOutputId);
internal_id!(WebhookDispatchId);
internal_id!(WebhookDeliveryId);
internal_id!(DomainEventId);
internal_id!(DeadLetterJobId);
internal_id!(SearchDocumentId);
internal_id!(ContentVectorId);
internal_id!(RecentSearchId);
internal_id!(UsageCounterId);
internal_id!(BackgroundJobRecoveryId);
