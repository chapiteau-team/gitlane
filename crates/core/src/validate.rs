use crate::errors::ConfigValidationError;

pub(crate) fn validate_non_blank(
    value: &str,
    message: impl Into<String>,
) -> Result<(), ConfigValidationError> {
    if value.trim().is_empty() {
        return Err(ConfigValidationError::new(message));
    }

    Ok(())
}

pub(crate) fn validate_id(
    id: &str,
    empty_message: impl Into<String>,
    whitespace_message: impl Into<String>,
) -> Result<(), ConfigValidationError> {
    if id.is_empty() {
        return Err(ConfigValidationError::new(empty_message));
    }

    if id.trim() != id {
        return Err(ConfigValidationError::new(whitespace_message));
    }

    Ok(())
}
