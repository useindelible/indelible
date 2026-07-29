use std::borrow::Cow;

use chrono::Duration;
use ind_domain::{DISPLAY_NAME_MAX_LENGTH, DomainError, Theme, TokenScope, UserId};
use validator::ValidateEmail;
use validator::{ValidationError, ValidationErrors, ValidationErrorsKind};

use crate::FieldError;

const MAX_LOCALE_LENGTH: usize = 35;
const MAX_TIMEZONE_LENGTH: usize = 100;

pub fn validation_errors_to_field_errors(errors: &ValidationErrors) -> Vec<FieldError> {
    let mut field_errors = Vec::new();
    collect_validation_errors("", errors, &mut field_errors);
    field_errors
}

pub fn trimmed_email(value: &str) -> Result<(), ValidationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.validate_email() {
        return Err(error_with_message("email", "must be a valid email address"));
    }

    Ok(())
}

pub fn trimmed_non_blank(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(error_with_message("required", "must not be empty"));
    }

    Ok(())
}

pub fn trimmed_max_display_name_length(value: &str) -> Result<(), ValidationError> {
    if value.trim().chars().count() > DISPLAY_NAME_MAX_LENGTH {
        return Err(error_with_message(
            "length",
            format!("must be at most {DISPLAY_NAME_MAX_LENGTH} characters"),
        ));
    }

    Ok(())
}

pub fn optional_trimmed_non_blank(value: &str) -> Result<(), ValidationError> {
    trimmed_non_blank(value)
}

pub fn optional_trimmed_max_display_name_length(value: &str) -> Result<(), ValidationError> {
    trimmed_max_display_name_length(value)
}

pub fn optional_locale(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() || value.len() > MAX_LOCALE_LENGTH {
        return Err(error_with_message(
            "length",
            format!("must be between 1 and {MAX_LOCALE_LENGTH} characters"),
        ));
    }

    Ok(())
}

pub fn optional_theme(value: &str) -> Result<(), ValidationError> {
    if value.parse::<Theme>().is_err() {
        return Err(error_with_message(
            "choice",
            format!("must be one of: {}", Theme::NAMES.join(", ")),
        ));
    }

    Ok(())
}

pub fn optional_timezone(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() || value.len() > MAX_TIMEZONE_LENGTH {
        return Err(error_with_message(
            "length",
            format!("must be between 1 and {MAX_TIMEZONE_LENGTH} characters"),
        ));
    }

    Ok(())
}

pub fn optional_avatar_reference(value: &str) -> Result<(), ValidationError> {
    if value.starts_with("https://")
        || value.starts_with("http://")
        || is_internal_avatar_key(value)
    {
        return Ok(());
    }

    Err(error_with_message(
        "url",
        "must be a valid http or https URL, or an internal avatar key",
    ))
}

pub fn is_internal_avatar_key(value: &str) -> bool {
    let mut segments = value.split('/');
    let Some(user_id) = segments.next() else {
        return false;
    };
    let Some(kind) = segments.next() else {
        return false;
    };
    let Some(filename) = segments.next() else {
        return false;
    };
    if segments.next().is_some() || kind != "avatars" || user_id.parse::<UserId>().is_err() {
        return false;
    }

    let Some((stem, ext)) = filename.rsplit_once('.') else {
        return false;
    };

    !stem.is_empty() && !ext.is_empty()
}

pub fn avatar_key_belongs_to_user(user_id: &UserId, value: &str) -> bool {
    is_internal_avatar_key(value) && value.starts_with(&format!("{user_id}/avatars/"))
}

pub fn allowed_scopes(value: &[String]) -> Result<(), ValidationError> {
    for scope in value {
        if scope.parse::<TokenScope>().is_err() {
            return Err(error_with_message(
                "choice",
                format!("invalid scope: {scope}"),
            ));
        }
    }

    Ok(())
}

pub fn positive_duration(value: &Duration) -> Result<(), ValidationError> {
    if *value <= Duration::zero() {
        return Err(error_with_message("range", "must be greater than zero"));
    }

    Ok(())
}

pub fn password_length(value: &str) -> Result<(), ValidationError> {
    ind_domain::validate_password(value).map_err(|err| match err {
        DomainError::Validation { message, .. } => error_with_message("length", message),
        other => error_with_message("password", other.to_string()),
    })
}

fn collect_validation_errors(
    prefix: &str,
    errors: &ValidationErrors,
    field_errors: &mut Vec<FieldError>,
) {
    let mut entries: Vec<_> = errors.errors().iter().collect();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));

    for (field, kind) in entries {
        let field_path = if prefix.is_empty() {
            field.to_string()
        } else {
            format!("{prefix}.{field}")
        };

        match kind {
            ValidationErrorsKind::Field(errors) => {
                for error in errors {
                    field_errors.push(FieldError {
                        field: field_path.clone(),
                        message: validation_error_message(error),
                    });
                }
            }
            ValidationErrorsKind::Struct(errors) => {
                collect_validation_errors(&field_path, errors, field_errors);
            }
            ValidationErrorsKind::List(errors) => {
                let mut entries: Vec<_> = errors.iter().collect();
                entries.sort_by_key(|(index, _)| *index);

                for (index, errors) in entries {
                    collect_validation_errors(
                        &format!("{field_path}[{index}]"),
                        errors,
                        field_errors,
                    );
                }
            }
        }
    }
}

fn validation_error_message(error: &ValidationError) -> String {
    if let Some(message) = &error.message {
        return message.to_string();
    }

    "is invalid".to_string()
}

fn error_with_message(
    code: &'static str,
    message: impl Into<Cow<'static, str>>,
) -> ValidationError {
    let mut error = ValidationError::new(code);
    error.message = Some(message.into());
    error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_length_uses_domain_policy_message() {
        let domain_message = match ind_domain::validate_password("short").unwrap_err() {
            DomainError::Validation { message, .. } => message,
            other => panic!("expected domain validation error, got {other:?}"),
        };

        let error = password_length("short").unwrap_err();
        assert_eq!(error.message.as_deref(), Some(domain_message.as_str()));
    }
}
