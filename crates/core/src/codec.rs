use std::path::Path;

use serde::Serialize;

use crate::{
    errors::{ConfigParseError, ConfigValidationError, GitlaneError},
    fs::{read_text_file, write_text_file},
};

pub(crate) fn load_config_from_path<T>(
    config_path: &Path,
    parse: impl FnOnce(&str, &Path) -> Result<T, GitlaneError>,
) -> Result<T, GitlaneError> {
    let content = read_text_file(config_path)?;
    parse(&content, config_path)
}

pub(crate) fn parse_config<T, Repr, E>(
    content: &str,
    config_path: &Path,
    parse: impl FnOnce(&str) -> Result<Repr, E>,
) -> Result<T, GitlaneError>
where
    T: TryFrom<Repr, Error = ConfigValidationError>,
    E: Into<ConfigParseError>,
{
    let repr = parse(content).map_err(|source| GitlaneError::parse_config(config_path, source))?;

    repr.try_into()
        .map_err(|source| GitlaneError::invalid_config(config_path, source))
}

pub(crate) fn to_toml_string<T: Serialize>(
    value: &T,
    config_path: &Path,
) -> Result<String, GitlaneError> {
    toml::to_string(value).map_err(|source| GitlaneError::serialize_config(config_path, source))
}

pub(crate) fn save_toml_config<T: Serialize>(
    config_path: &Path,
    value: &T,
) -> Result<(), GitlaneError> {
    let content = to_toml_string(value, config_path)?;
    write_text_file(config_path, &content)?;
    Ok(())
}

pub(crate) fn to_yaml_string<T: Serialize>(
    value: &T,
    config_path: &Path,
) -> Result<String, GitlaneError> {
    serde_yaml::to_string(value)
        .map_err(|source| GitlaneError::serialize_config(config_path, source))
}

pub(crate) fn save_yaml_config<T: Serialize>(
    config_path: &Path,
    value: &T,
) -> Result<(), GitlaneError> {
    let content = to_yaml_string(value, config_path)?;
    write_text_file(config_path, &content)?;
    Ok(())
}
