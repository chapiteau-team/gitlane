use crate::errors::ConfigValidationError;

const WINDOWS_RESERVED_PATH_SEGMENTS: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

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

pub(crate) fn validate_path_segment(
    value: &str,
    empty_message: impl Into<String>,
    whitespace_message: impl Into<String>,
    invalid_message: impl Into<String>,
) -> Result<(), ConfigValidationError> {
    validate_id(value, empty_message, whitespace_message)?;

    if value == "."
        || value == ".."
        || value.ends_with('.')
        || value.ends_with(' ')
        || value.chars().any(is_invalid_path_segment_char)
        || is_windows_reserved_path_segment(value)
    {
        return Err(ConfigValidationError::new(invalid_message));
    }

    Ok(())
}

fn is_invalid_path_segment_char(ch: char) -> bool {
    ch.is_control() || matches!(ch, '/' | '\\' | '<' | '>' | ':' | '"' | '|' | '?' | '*')
}

fn is_windows_reserved_path_segment(value: &str) -> bool {
    let stem = value.split('.').next().unwrap_or(value);

    WINDOWS_RESERVED_PATH_SEGMENTS
        .iter()
        .any(|reserved| stem.eq_ignore_ascii_case(reserved))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY_MESSAGE: &str = "value must be non-empty";
    const WHITESPACE_MESSAGE: &str = "value must not have surrounding whitespace";
    const INVALID_MESSAGE: &str = "value must be a portable filesystem-safe path segment";

    fn validate(value: &str) -> Result<(), ConfigValidationError> {
        validate_path_segment(value, EMPTY_MESSAGE, WHITESPACE_MESSAGE, INVALID_MESSAGE)
    }

    #[test]
    fn accepts_portable_path_segment() {
        assert!(validate("in_progress").is_ok());
        assert!(validate("release-2026").is_ok());
    }

    #[test]
    fn rejects_segment_with_path_separator() {
        let err = validate("in/progress").expect_err("path separator should fail");

        assert_eq!(err.to_string(), INVALID_MESSAGE);
    }

    #[test]
    fn rejects_segment_with_windows_reserved_name() {
        let err = validate("CON").expect_err("reserved name should fail");

        assert_eq!(err.to_string(), INVALID_MESSAGE);
    }

    #[test]
    fn rejects_segment_with_reserved_stem_and_extension() {
        let err = validate("nul.txt").expect_err("reserved stem should fail");

        assert_eq!(err.to_string(), INVALID_MESSAGE);
    }

    #[test]
    fn rejects_segment_that_ends_with_dot() {
        let err = validate("review.").expect_err("trailing dot should fail");

        assert_eq!(err.to_string(), INVALID_MESSAGE);
    }

    #[test]
    fn rejects_segment_that_is_current_or_parent_directory() {
        assert_eq!(
            validate(".")
                .expect_err("current directory segment should fail")
                .to_string(),
            INVALID_MESSAGE
        );
        assert_eq!(
            validate("..")
                .expect_err("parent directory segment should fail")
                .to_string(),
            INVALID_MESSAGE
        );
    }
}
