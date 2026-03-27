use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    errors::GitlaneError,
    fs::{read_text_file, write_text_file},
    paths::PROJECT_CONFIG_FILE,
};

use super::ProjectConfig;

#[derive(Debug, Deserialize, Serialize)]
struct RawProjectConfig {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    homepage: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    people: Vec<String>,
}

impl From<&ProjectConfig> for RawProjectConfig {
    fn from(config: &ProjectConfig) -> Self {
        Self {
            name: config.name().to_owned(),
            description: config.description().map(ToOwned::to_owned),
            homepage: config.homepage().map(ToOwned::to_owned),
            people: config.people().to_vec(),
        }
    }
}

pub fn load(project_dir: &Path) -> Result<ProjectConfig, GitlaneError> {
    load_from_path(&project_dir.join(PROJECT_CONFIG_FILE))
}

pub fn load_from_path(config_path: &Path) -> Result<ProjectConfig, GitlaneError> {
    let content = read_text_file(config_path)?;
    parse_str(&content, config_path)
}

pub fn parse_str(content: &str, config_path: &Path) -> Result<ProjectConfig, GitlaneError> {
    let raw: RawProjectConfig =
        ::toml::from_str(content).map_err(|source| GitlaneError::ParseToml {
            path: config_path.to_path_buf(),
            source,
        })?;

    ProjectConfig::new(raw.name, raw.description, raw.homepage, raw.people)
        .map_err(|source| GitlaneError::invalid_config(config_path, source))
}

pub fn to_string(config: &ProjectConfig, config_path: &Path) -> Result<String, GitlaneError> {
    ::toml::to_string(&RawProjectConfig::from(config)).map_err(|source| {
        GitlaneError::SerializeToml {
            path: config_path.to_path_buf(),
            source,
        }
    })
}

pub fn save_to_path(config_path: &Path, config: &ProjectConfig) -> Result<(), GitlaneError> {
    let content = to_string(config, config_path)?;
    write_text_file(config_path, &content)?;
    Ok(())
}
