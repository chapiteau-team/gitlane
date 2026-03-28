use std::path::Path;

use crate::{
    codec::{load_config_from_path, parse_config, save_toml_config, to_toml_string},
    errors::GitlaneError,
};

use super::{IssuesConfig, codec::IssuesConfigRepr};

pub fn load_from_path(config_path: &Path) -> Result<IssuesConfig, GitlaneError> {
    load_config_from_path(config_path, parse_str)
}

pub fn parse_str(content: &str, config_path: &Path) -> Result<IssuesConfig, GitlaneError> {
    parse_config(content, config_path, |content| {
        ::toml::from_str::<IssuesConfigRepr>(content)
    })
}

pub fn to_string(config: &IssuesConfig, config_path: &Path) -> Result<String, GitlaneError> {
    to_toml_string(&IssuesConfigRepr::from(config), config_path)
}

pub fn save_to_path(config_path: &Path, config: &IssuesConfig) -> Result<(), GitlaneError> {
    save_toml_config(config_path, &IssuesConfigRepr::from(config))
}
