use std::path::Path;

use crate::{
    config::{load_config_from_path, parse_config, save_toml_config, to_toml_string},
    errors::GitlaneError,
};

use super::{LabelsConfig, codec::LabelsConfigRepr};

pub fn load_from_path(config_path: &Path) -> Result<LabelsConfig, GitlaneError> {
    load_config_from_path(config_path, parse_str)
}

pub fn parse_str(content: &str, config_path: &Path) -> Result<LabelsConfig, GitlaneError> {
    parse_config(content, config_path, |content| {
        ::toml::from_str::<LabelsConfigRepr>(content)
    })
}

pub fn to_string(config: &LabelsConfig, config_path: &Path) -> Result<String, GitlaneError> {
    to_toml_string(&LabelsConfigRepr::from(config), config_path)
}

pub fn save_to_path(config_path: &Path, config: &LabelsConfig) -> Result<(), GitlaneError> {
    save_toml_config(config_path, &LabelsConfigRepr::from(config))
}
