use std::borrow::Cow;

use ind_domain::{DISPLAY_NAME_MAX_LENGTH, UserId};
use validator::ValidationError;

const MAX_LOCALE_LENGTH: usize = 35;
const MAX_TIMEZONE_LENGTH: usize = 100;
const MAX_API_TOKEN_NAME_LENGTH: usize = 100;

pub fn optional_trimmed_non_blank(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(error_with_message("required", "must not be empty"))
    } else {
        Ok(())
    }
}

pub fn optional_trimmed_max_display_name_length(value: &str) -> Result<(), ValidationError> {
    if value.trim().chars().count() > DISPLAY_NAME_MAX_LENGTH {
        return Err(error_with_message(
            "length",
            format!("must be at most {DISPLAY_NAME_MAX_LENGTH} characters"),
        ));
    }

    Ok(())
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

pub fn trimmed_non_blank(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(error_with_message("required", "must not be empty"))
    } else {
        Ok(())
    }
}

pub fn trimmed_max_api_token_name_length(value: &str) -> Result<(), ValidationError> {
    if value.trim().len() > MAX_API_TOKEN_NAME_LENGTH {
        return Err(error_with_message(
            "length",
            format!("must be at most {MAX_API_TOKEN_NAME_LENGTH} characters"),
        ));
    }

    Ok(())
}

pub fn non_empty_token_scopes<T>(value: &[T]) -> Result<(), ValidationError> {
    if value.is_empty() {
        Err(error_with_message(
            "length",
            "must include at least one scope",
        ))
    } else {
        Ok(())
    }
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
    fn text_and_collection_boundaries_are_enforced() {
        assert!(optional_trimmed_non_blank(" value ").is_ok());
        assert!(optional_trimmed_non_blank(" \n ").is_err());
        assert!(
            optional_trimmed_max_display_name_length(&"é".repeat(DISPLAY_NAME_MAX_LENGTH)).is_ok()
        );
        assert!(
            optional_trimmed_max_display_name_length(&"é".repeat(DISPLAY_NAME_MAX_LENGTH + 1))
                .is_err()
        );
        assert!(optional_locale("en-US").is_ok());
        assert!(optional_locale("").is_err());
        assert!(optional_timezone("Africa/Lagos").is_ok());
        assert!(optional_timezone(&"x".repeat(MAX_TIMEZONE_LENGTH + 1)).is_err());
        assert!(trimmed_max_api_token_name_length(&"x".repeat(MAX_API_TOKEN_NAME_LENGTH)).is_ok());
        assert!(
            trimmed_max_api_token_name_length(&"x".repeat(MAX_API_TOKEN_NAME_LENGTH + 1)).is_err()
        );
        assert!(non_empty_token_scopes(&["read"]).is_ok());
        assert!(non_empty_token_scopes::<&str>(&[]).is_err());
    }

    #[test]
    fn avatar_references_are_structurally_and_user_scoped() {
        let user = UserId::new();
        let key = format!("{user}/avatars/profile.webp");
        for valid in [
            "https://cdn.example/avatar",
            "http://cdn.example/avatar",
            &key,
        ] {
            assert!(optional_avatar_reference(valid).is_ok());
        }
        for invalid in [
            "ftp://example/avatar",
            "avatars/file.png",
            "bad/avatars/file.png/extra",
        ] {
            assert!(optional_avatar_reference(invalid).is_err());
        }
        assert!(avatar_key_belongs_to_user(&user, &key));
        assert!(!avatar_key_belongs_to_user(&UserId::new(), &key));
    }
}
