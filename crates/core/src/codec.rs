use std::path::Path;

use serde::{Serialize, de::DeserializeOwned};

use crate::{
    config::ConfigFileExtension,
    errors::{ConfigValidationError, GitlaneError},
    fs::{read_text_file, write_text_file},
};

pub(crate) fn load<T, Repr>(config_path: &Path) -> Result<T, GitlaneError>
where
    Repr: DeserializeOwned,
    T: TryFrom<Repr, Error = ConfigValidationError>,
{
    let content = read_text_file(config_path)?;
    parse::<T, Repr>(&content, config_path)
}

pub(crate) fn parse<T, Repr>(content: &str, config_path: &Path) -> Result<T, GitlaneError>
where
    Repr: DeserializeOwned,
    T: TryFrom<Repr, Error = ConfigValidationError>,
{
    let repr: Repr = match ConfigFileExtension::from_path(config_path)? {
        ConfigFileExtension::Toml => toml::from_str(content)
            .map_err(|source| GitlaneError::parse_config(config_path, source))?,
        ConfigFileExtension::Json => serde_json::from_str(content)
            .map_err(|source| GitlaneError::parse_config(config_path, source))?,
        ConfigFileExtension::Yaml | ConfigFileExtension::Yml => serde_yaml::from_str(content)
            .map_err(|source| GitlaneError::parse_config(config_path, source))?,
    };

    repr.try_into()
        .map_err(|source| GitlaneError::invalid_config(config_path, source))
}

pub(crate) fn save<T, Repr>(config_path: &Path, value: &T) -> Result<(), GitlaneError>
where
    Repr: Serialize + for<'a> From<&'a T>,
{
    let repr = Repr::from(value);
    let content = match ConfigFileExtension::from_path(config_path)? {
        ConfigFileExtension::Toml => toml::to_string(&repr)
            .map_err(|source| GitlaneError::serialize_config(config_path, source))?,
        ConfigFileExtension::Json => serde_json::to_string_pretty(&repr)
            .map_err(|source| GitlaneError::serialize_config(config_path, source))?,
        ConfigFileExtension::Yaml | ConfigFileExtension::Yml => serde_yaml::to_string(&repr)
            .map_err(|source| GitlaneError::serialize_config(config_path, source))?,
    };

    write_text_file(config_path, &content)?;
    Ok(())
}
