use crate::config::ConfigFileExtension;

/// Returns the default built-in issue template content for `issue.md`.
pub fn default(format: ConfigFileExtension) -> &'static str {
    match format {
        ConfigFileExtension::Toml => {
            "+++\ntitle = \"New issue\"\nassignees = []\nlabels = []\n+++\n"
        }
        ConfigFileExtension::Json => {
            "{\n  \"title\": \"New issue\",\n  \"assignees\": [],\n  \"labels\": []\n}\n"
        }
        ConfigFileExtension::Yaml | ConfigFileExtension::Yml => {
            "---\ntitle: New issue\nassignees: []\nlabels: []\n---\n"
        }
    }
}
