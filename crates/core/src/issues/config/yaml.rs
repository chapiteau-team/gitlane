use std::path::Path;

use crate::{
    config::{load_config_from_path, parse_config},
    errors::GitlaneError,
};

use super::{IssuesConfig, codec::IssuesConfigRepr};

pub fn load_from_path(config_path: &Path) -> Result<IssuesConfig, GitlaneError> {
    load_config_from_path(config_path, parse_str)
}

pub fn parse_str(content: &str, config_path: &Path) -> Result<IssuesConfig, GitlaneError> {
    parse_config(content, config_path, |content| {
        serde_yaml::from_str::<IssuesConfigRepr>(content)
    })
}
