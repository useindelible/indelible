use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::FieldError;
use crate::extract::Validate;
use ind_application::outputs::mila::{MilaPromptPresetGroupOutput, MilaPromptPresetOutput};
use ind_application::ports::{CreateMilaPromptPresetRequest, UpdateMilaPromptPresetRequest};

use super::{VALID_PROMPT_ACTIONS, format_prompt_action, parse_prompt_action, validate_required};

#[derive(Debug, Serialize, ToSchema)]
pub struct MilaPromptPresetResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub action: String,
    pub name: String,
    pub system_prompt: String,
    pub is_default: bool,
    pub is_built_in: bool,
}

pub fn project_mila_prompt_preset(output: MilaPromptPresetOutput) -> MilaPromptPresetResponse {
    MilaPromptPresetResponse {
        id: output.id.map(|id| id.to_string()),
        action: format_prompt_action(output.action).into(),
        name: output.name,
        system_prompt: output.system_prompt,
        is_default: output.is_default,
        is_built_in: output.is_built_in,
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MilaPromptPresetGroupResponse {
    pub action: String,
    pub presets: Vec<MilaPromptPresetResponse>,
}

pub fn project_mila_prompt_preset_group(
    output: MilaPromptPresetGroupOutput,
) -> MilaPromptPresetGroupResponse {
    MilaPromptPresetGroupResponse {
        action: format_prompt_action(output.action).into(),
        presets: output
            .presets
            .into_iter()
            .map(project_mila_prompt_preset)
            .collect(),
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MilaPromptPresetsResponse {
    pub groups: Vec<MilaPromptPresetGroupResponse>,
}

pub fn project_mila_prompt_presets(
    outputs: Vec<MilaPromptPresetGroupOutput>,
) -> MilaPromptPresetsResponse {
    MilaPromptPresetsResponse {
        groups: outputs
            .into_iter()
            .map(project_mila_prompt_preset_group)
            .collect(),
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMilaPromptPresetBody {
    pub action: String,
    pub name: String,
    pub system_prompt: String,
    #[serde(default)]
    pub is_default: bool,
}

impl Validate for CreateMilaPromptPresetBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        if parse_prompt_action(&self.action).is_none() {
            errors.push(FieldError {
                field: "action".into(),
                message: format!("must be one of: {}", VALID_PROMPT_ACTIONS.join(", ")),
            });
        }
        validate_required("name", &self.name, &mut errors);
        validate_required("system_prompt", &self.system_prompt, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl CreateMilaPromptPresetBody {
    pub fn into_state_request(self) -> Result<CreateMilaPromptPresetRequest, Vec<FieldError>> {
        let action = parse_prompt_action(&self.action).ok_or_else(|| {
            vec![FieldError {
                field: "action".into(),
                message: format!("must be one of: {}", VALID_PROMPT_ACTIONS.join(", ")),
            }]
        })?;

        Ok(CreateMilaPromptPresetRequest {
            action,
            name: self.name.trim().to_string(),
            system_prompt: self.system_prompt.trim().to_string(),
            is_default: self.is_default,
        })
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMilaPromptPresetBody {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub is_default: Option<bool>,
}

impl Validate for UpdateMilaPromptPresetBody {
    fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();

        if self.name.is_none() && self.system_prompt.is_none() && self.is_default.is_none() {
            errors.push(FieldError {
                field: "body".into(),
                message: "must include at least one field to update".into(),
            });
        }

        if self
            .name
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            errors.push(FieldError {
                field: "name".into(),
                message: "must not be empty".into(),
            });
        }

        if self
            .system_prompt
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            errors.push(FieldError {
                field: "system_prompt".into(),
                message: "must not be empty".into(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl UpdateMilaPromptPresetBody {
    pub fn into_state_request(self) -> UpdateMilaPromptPresetRequest {
        UpdateMilaPromptPresetRequest {
            name: self.name.map(|value| value.trim().to_string()),
            system_prompt: self.system_prompt.map(|value| value.trim().to_string()),
            is_default: self.is_default,
        }
    }
}
