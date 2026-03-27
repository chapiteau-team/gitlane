use std::path::Path;

use crate::{
    config::{
        ConfigKind, default_config_path, load_config_from_path, parse_config, save_toml_config,
        to_toml_string,
    },
    errors::GitlaneError,
};

use super::{ProjectConfig, codec::ProjectConfigRepr};

pub fn load(project_dir: &Path) -> Result<ProjectConfig, GitlaneError> {
    load_from_path(&default_config_path(project_dir, ConfigKind::Project))
}

pub fn load_from_path(config_path: &Path) -> Result<ProjectConfig, GitlaneError> {
    load_config_from_path(config_path, parse_str)
}

pub fn parse_str(content: &str, config_path: &Path) -> Result<ProjectConfig, GitlaneError> {
    parse_config(content, config_path, |content| {
        ::toml::from_str::<ProjectConfigRepr>(content)
    })
}

pub fn to_string(config: &ProjectConfig, config_path: &Path) -> Result<String, GitlaneError> {
    to_toml_string(&ProjectConfigRepr::from(config), config_path)
}

pub fn save_to_path(config_path: &Path, config: &ProjectConfig) -> Result<(), GitlaneError> {
    save_toml_config(config_path, &ProjectConfigRepr::from(config))
}
