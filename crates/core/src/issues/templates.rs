//! Built-in issue template scaffolding helpers.

use crate::config::ConfigFileExtension;
use crate::frontmatter::{JSON_OBJECT_END, JSON_OBJECT_START, TOML_FENCE, YAML_FENCE};

/// Returns the default built-in issue template content for `issue.md`.
pub fn default(format: ConfigFileExtension) -> String {
    match format {
        ConfigFileExtension::Toml => format!(
            "{TOML_FENCE}\ntitle = \"New issue\"\nassignees = []\nlabels = []\n{TOML_FENCE}\n"
        ),
        ConfigFileExtension::Json => format!(
            "{JSON_OBJECT_START}\n  \"title\": \"New issue\",\n  \"assignees\": [],\n  \"labels\": []\n{JSON_OBJECT_END}\n"
        ),
        ConfigFileExtension::Yaml | ConfigFileExtension::Yml => {
            format!("{YAML_FENCE}\ntitle: New issue\nassignees: []\nlabels: []\n{YAML_FENCE}\n")
        }
    }
}
