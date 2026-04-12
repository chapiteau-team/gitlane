use std::{collections::HashSet, path::Path};

use thiserror::Error;
use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

use crate::{
    errors::GitlaneError,
    frontmatter::{self, FrontmatterDocument, FrontmatterSerializeError},
    fs::{ensure_file, read_text_file, write_text_file},
    issues::{config::PriorityId, labels::LabelId},
    validate::{ValidationError, validate_non_blank},
};

pub(crate) mod repr;

pub use crate::frontmatter::FrontmatterFormat;

/// Validation error for parsed issue content.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(transparent)]
pub struct IssueValidationError(#[from] ValidationError);

impl IssueValidationError {
    /// Creates a new validation error with a user-facing message.
    pub fn new(message: impl Into<String>) -> Self {
        Self(ValidationError::new(message))
    }
}

/// Typed issue document loaded from `issue.md`.
#[derive(Debug, Clone)]
pub struct Issue {
    metadata: IssueMetadata,
    body: String,
    front_matter: FrontmatterDocument,
}

/// Typed mutable issue metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueMetadata {
    title: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    reporter: String,
    assignees: Vec<String>,
    priority: PriorityId,
    labels: Vec<LabelId>,
}

impl Issue {
    /// Loads a typed issue document from disk.
    pub fn load(issue_path: &Path) -> Result<Self, GitlaneError> {
        ensure_file(issue_path)?;
        let content = read_text_file(issue_path)?;
        Self::parse(&content, issue_path)
    }

    /// Parses a typed issue document from `content`.
    pub fn parse(content: &str, issue_path: &Path) -> Result<Self, GitlaneError> {
        let parsed = frontmatter::parse(content)
            .map_err(|source| GitlaneError::from_frontmatter(issue_path, source))?;
        let metadata_repr: repr::IssueMetadataRepr = parsed
            .document
            .deserialize()
            .map_err(|source| GitlaneError::parse_frontmatter(issue_path, source))?;
        let metadata = metadata_repr
            .try_into()
            .map_err(|source| GitlaneError::invalid_issue(issue_path, source))?;

        Ok(Self {
            metadata,
            body: parsed.body,
            front_matter: parsed.document,
        })
    }

    /// Returns immutable typed issue metadata.
    pub fn metadata(&self) -> &IssueMetadata {
        &self.metadata
    }

    /// Returns the markdown body exactly as stored after front matter.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Returns the parsed front matter format.
    pub fn front_matter_format(&self) -> FrontmatterFormat {
        self.front_matter.format()
    }

    /// Replaces the issue title.
    pub fn set_title(&mut self, title: String) -> Result<(), IssueValidationError> {
        validate_title(&title)?;
        self.metadata.title = title;
        Ok(())
    }

    /// Replaces the ordered assignee list.
    pub fn set_assignees(&mut self, assignees: Vec<String>) -> Result<(), IssueValidationError> {
        validate_person_refs(&assignees, "assignees", "person reference")?;
        self.metadata.assignees = assignees;
        Ok(())
    }

    /// Replaces the issue priority id.
    pub fn set_priority(&mut self, priority: PriorityId) -> Result<(), IssueValidationError> {
        validate_issue_non_blank(
            &priority,
            "issue metadata `priority` must be a non-empty priority id",
        )?;
        self.metadata.priority = priority;
        Ok(())
    }

    /// Replaces the ordered label id list.
    pub fn set_labels(&mut self, labels: Vec<LabelId>) -> Result<(), IssueValidationError> {
        validate_identifiers(&labels, "labels", "label id")?;
        self.metadata.labels = labels;
        Ok(())
    }

    /// Saves metadata changes back to `issue_path` and refreshes `updated_at`.
    pub fn save(&mut self, issue_path: &Path) -> Result<(), GitlaneError> {
        ensure_file(issue_path)?;

        self.metadata.touch_updated_at();
        let metadata_repr = repr::IssueMetadataRepr::from_metadata(&self.metadata)
            .map_err(|source| GitlaneError::serialize_frontmatter(issue_path, source))?;
        apply_issue_metadata(&mut self.front_matter, &metadata_repr);

        let content = self
            .front_matter
            .render(&self.body)
            .map_err(|source| GitlaneError::serialize_frontmatter(issue_path, source))?;
        write_text_file(issue_path, &content)?;
        Ok(())
    }
}

impl IssueMetadata {
    pub(super) fn new(
        title: String,
        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
        reporter: String,
        assignees: Vec<String>,
        priority: PriorityId,
        labels: Vec<LabelId>,
    ) -> Result<Self, IssueValidationError> {
        validate_title(&title)?;
        validate_utc_timestamp(created_at, "created_at")?;
        validate_utc_timestamp(updated_at, "updated_at")?;
        validate_issue_non_blank(
            &reporter,
            "issue metadata `reporter` must be a non-empty person reference",
        )?;
        validate_person_refs(&assignees, "assignees", "person reference")?;
        validate_issue_non_blank(
            &priority,
            "issue metadata `priority` must be a non-empty priority id",
        )?;
        validate_identifiers(&labels, "labels", "label id")?;

        if updated_at < created_at {
            return Err(IssueValidationError::new(
                "issue metadata `updated_at` must be greater than or equal to `created_at`",
            ));
        }

        Ok(Self {
            title,
            created_at,
            updated_at,
            reporter,
            assignees,
            priority,
            labels,
        })
    }

    /// Returns the non-empty issue title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the immutable creation timestamp.
    pub fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }

    /// Returns the last updated timestamp.
    pub fn updated_at(&self) -> OffsetDateTime {
        self.updated_at
    }

    /// Returns the immutable reporter reference.
    pub fn reporter(&self) -> &str {
        &self.reporter
    }

    /// Returns the ordered assignee list.
    pub fn assignees(&self) -> &[String] {
        &self.assignees
    }

    /// Returns the priority id.
    pub fn priority(&self) -> &str {
        &self.priority
    }

    /// Returns the ordered label id list.
    pub fn labels(&self) -> &[LabelId] {
        &self.labels
    }

    fn touch_updated_at(&mut self) {
        let now = OffsetDateTime::now_utc();
        self.updated_at = std::cmp::max(now, std::cmp::max(self.updated_at, self.created_at));
    }
}

pub(super) fn parse_utc_timestamp(
    raw_value: String,
    field_name: &str,
) -> Result<OffsetDateTime, IssueValidationError> {
    validate_issue_non_blank(
        &raw_value,
        format!("issue metadata `{field_name}` must be an RFC3339 UTC timestamp with `Z` suffix"),
    )?;

    if !raw_value.ends_with('Z') {
        return Err(IssueValidationError::new(format!(
            "issue metadata `{field_name}` must be an RFC3339 UTC timestamp with `Z` suffix"
        )));
    }

    let timestamp = OffsetDateTime::parse(&raw_value, &Rfc3339).map_err(|_| {
        IssueValidationError::new(format!(
            "issue metadata `{field_name}` must be an RFC3339 UTC timestamp with `Z` suffix"
        ))
    })?;

    validate_utc_timestamp(timestamp, field_name)?;
    Ok(timestamp.to_offset(UtcOffset::UTC))
}

pub(super) fn format_utc_timestamp(
    timestamp: OffsetDateTime,
) -> Result<String, FrontmatterSerializeError> {
    Ok(timestamp.to_offset(UtcOffset::UTC).format(&Rfc3339)?)
}

fn apply_issue_metadata(document: &mut FrontmatterDocument, metadata: &repr::IssueMetadataRepr) {
    document.set_string(
        "title",
        metadata.title.as_deref().expect("title should be present"),
    );
    document.set_string(
        "created_at",
        metadata
            .created_at
            .as_deref()
            .expect("created_at should be present"),
    );
    document.set_string(
        "updated_at",
        metadata
            .updated_at
            .as_deref()
            .expect("updated_at should be present"),
    );
    document.set_string(
        "reporter",
        metadata
            .reporter
            .as_deref()
            .expect("reporter should be present"),
    );
    document.set_string_list(
        "assignees",
        metadata
            .assignees
            .as_deref()
            .expect("assignees should be present"),
    );
    document.set_string(
        "priority",
        metadata
            .priority
            .as_deref()
            .expect("priority should be present"),
    );
    document.set_string_list(
        "labels",
        metadata
            .labels
            .as_deref()
            .expect("labels should be present"),
    );
}

fn validate_title(title: &str) -> Result<(), IssueValidationError> {
    validate_issue_non_blank(title, "issue metadata `title` must be a non-empty string")
}

fn validate_utc_timestamp(
    timestamp: OffsetDateTime,
    field_name: &str,
) -> Result<(), IssueValidationError> {
    if timestamp.offset() != UtcOffset::UTC {
        return Err(IssueValidationError::new(format!(
            "issue metadata `{field_name}` must be an RFC3339 UTC timestamp with `Z` suffix"
        )));
    }

    Ok(())
}

fn validate_person_refs(
    values: &[String],
    field_name: &str,
    value_name: &str,
) -> Result<(), IssueValidationError> {
    validate_unique_values(values, field_name, value_name)
}

fn validate_identifiers(
    values: &[String],
    field_name: &str,
    value_name: &str,
) -> Result<(), IssueValidationError> {
    validate_unique_values(values, field_name, value_name)
}

fn validate_unique_values(
    values: &[String],
    field_name: &str,
    value_name: &str,
) -> Result<(), IssueValidationError> {
    let mut seen = HashSet::with_capacity(values.len());

    for (index, value) in values.iter().enumerate() {
        validate_issue_non_blank(
            value,
            format!("issue metadata `{field_name}[{index}]` must be a non-empty {value_name}"),
        )?;

        if !seen.insert(value) {
            return Err(IssueValidationError::new(format!(
                "duplicate value `{value}` in issue metadata `{field_name}`"
            )));
        }
    }

    Ok(())
}

fn validate_issue_non_blank(
    value: &str,
    message: impl Into<String>,
) -> Result<(), IssueValidationError> {
    validate_non_blank(value, message)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{fs, path::Path};

    use tempfile::TempDir;

    use crate::{config::ConfigFileExtension, errors::GitlaneError, issues::templates};

    const TOML_ISSUE: &str = concat!(
        "+++\n",
        "title = \"Document parser\"\n",
        "created_at = \"2026-02-27T10:08:15Z\"\n",
        "updated_at = \"2026-02-27T10:08:15Z\"\n",
        "reporter = \"@alice\"\n",
        "assignees = [\"@bob\"]\n",
        "priority = \"p2\"\n",
        "labels = [\"type_docs\"]\n",
        "extra = \"keep\"\n",
        "+++\n",
        "\n",
        "Body line 1\n",
        "\n",
        "Body line 2\n",
    );

    const YAML_ISSUE: &str = concat!(
        "---\n",
        "title: Document parser\n",
        "created_at: 2026-02-27T10:08:15Z\n",
        "updated_at: 2026-02-27T10:08:15Z\n",
        "reporter: '@alice'\n",
        "assignees:\n",
        "  - '@bob'\n",
        "priority: p2\n",
        "labels:\n",
        "  - type_docs\n",
        "extra: keep\n",
        "---\n",
        "\n",
        "Body line 1\n",
        "\n",
        "Body line 2\n",
    );

    const JSON_ISSUE: &str = concat!(
        "{\n",
        "  \"title\": \"Document parser\",\n",
        "  \"created_at\": \"2026-02-27T10:08:15Z\",\n",
        "  \"updated_at\": \"2026-02-27T10:08:15Z\",\n",
        "  \"reporter\": \"@alice\",\n",
        "  \"assignees\": [\"@bob\"],\n",
        "  \"priority\": \"p2\",\n",
        "  \"labels\": [\"type_docs\"],\n",
        "  \"extra\": \"keep\"\n",
        "}\n",
        "\n",
        "Body line 1\n",
        "\n",
        "Body line 2\n",
    );

    fn issue_path(temp_dir: &TempDir, filename: &str) -> std::path::PathBuf {
        temp_dir.path().join(filename)
    }

    #[test]
    fn loads_and_saves_toml_issue() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let issue_path = issue_path(&temp_dir, "issue.md");
        fs::write(&issue_path, TOML_ISSUE).expect("issue should be written");

        let mut issue = Issue::load(&issue_path).expect("issue should load");
        let original_body = issue.body().to_owned();
        let original_updated_at = issue.metadata().updated_at();

        issue
            .set_title("Updated parser docs".to_owned())
            .expect("title should update");
        issue.save(&issue_path).expect("issue should save");

        let saved_content =
            fs::read_to_string(&issue_path).expect("saved issue should be readable");
        assert!(saved_content.contains("extra = \"keep\""));

        let saved_issue = Issue::load(&issue_path).expect("saved issue should reload");
        assert_eq!(saved_issue.metadata().title(), "Updated parser docs");
        assert_eq!(saved_issue.metadata().reporter(), "@alice");
        assert_eq!(
            saved_issue.metadata().created_at(),
            issue.metadata().created_at()
        );
        assert!(saved_issue.metadata().updated_at() >= original_updated_at);
        assert_eq!(saved_issue.body(), original_body);
        assert_eq!(saved_issue.front_matter_format(), FrontmatterFormat::Toml);
    }

    #[test]
    fn loads_and_saves_yaml_issue() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let issue_path = issue_path(&temp_dir, "issue.md");
        fs::write(&issue_path, YAML_ISSUE).expect("issue should be written");

        let mut issue = Issue::load(&issue_path).expect("issue should load");
        let original_body = issue.body().to_owned();

        issue
            .set_assignees(vec!["@bob".to_owned(), "@carol".to_owned()])
            .expect("assignees should update");
        issue.save(&issue_path).expect("issue should save");

        let saved_content =
            fs::read_to_string(&issue_path).expect("saved issue should be readable");
        assert!(saved_content.contains("extra: keep"));

        let saved_issue = Issue::load(&issue_path).expect("saved issue should reload");
        assert_eq!(
            saved_issue.metadata().assignees(),
            &["@bob".to_string(), "@carol".to_string()]
        );
        assert_eq!(saved_issue.body(), original_body);
        assert_eq!(saved_issue.front_matter_format(), FrontmatterFormat::Yaml);
    }

    #[test]
    fn loads_and_saves_json_issue() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let issue_path = issue_path(&temp_dir, "issue.md");
        fs::write(&issue_path, JSON_ISSUE).expect("issue should be written");

        let mut issue = Issue::load(&issue_path).expect("issue should load");
        let original_body = issue.body().to_owned();

        issue
            .set_labels(vec!["type_docs".to_owned(), "needs_decision".to_owned()])
            .expect("labels should update");
        issue.save(&issue_path).expect("issue should save");

        let saved_content =
            fs::read_to_string(&issue_path).expect("saved issue should be readable");
        assert!(saved_content.contains("\"extra\": \"keep\""));

        let saved_issue = Issue::load(&issue_path).expect("saved issue should reload");
        assert_eq!(
            saved_issue.metadata().labels(),
            &["type_docs".to_string(), "needs_decision".to_string()]
        );
        assert_eq!(saved_issue.body(), original_body);
        assert_eq!(saved_issue.front_matter_format(), FrontmatterFormat::Json);
    }

    #[test]
    fn parses_json_front_matter_with_body_immediately_after_object() {
        let issue = Issue::parse(
            concat!(
                "{",
                "\"title\":\"Document parser\",",
                "\"created_at\":\"2026-02-27T10:08:15Z\",",
                "\"updated_at\":\"2026-02-27T10:08:15Z\",",
                "\"reporter\":\"@alice\",",
                "\"assignees\":[],",
                "\"priority\":\"p2\",",
                "\"labels\":[]",
                "}",
                "Body"
            ),
            Path::new("issue.md"),
        )
        .expect("json issue should parse");

        assert_eq!(issue.body(), "Body");
        assert_eq!(issue.front_matter_format(), FrontmatterFormat::Json);
    }

    #[test]
    fn maps_front_matter_validation_errors_to_invalid_frontmatter() {
        let err = Issue::parse("title = \"Missing fence\"\n", Path::new("issue.md"))
            .expect_err("missing fence should fail");

        assert!(matches!(err, GitlaneError::InvalidFrontmatter { .. }));
        assert!(err.to_string().contains("document must start with TOML"));
    }

    #[test]
    fn maps_front_matter_parse_errors_to_parse_frontmatter() {
        let err = Issue::parse(
            concat!(
                "+++\n",
                "title = \"Document parser\"\n",
                "labels = [\n",
                "+++\n"
            ),
            Path::new("issue.md"),
        )
        .expect_err("invalid TOML front matter should fail");

        assert!(matches!(err, GitlaneError::ParseFrontmatter { .. }));
    }

    #[test]
    fn rejects_duplicate_assignees() {
        let err = Issue::parse(
            concat!(
                "+++\n",
                "title = \"Document parser\"\n",
                "created_at = \"2026-02-27T10:08:15Z\"\n",
                "updated_at = \"2026-02-27T10:08:15Z\"\n",
                "reporter = \"@alice\"\n",
                "assignees = [\"@bob\", \"@bob\"]\n",
                "priority = \"p2\"\n",
                "labels = []\n",
                "+++\n"
            ),
            Path::new("issue.md"),
        )
        .expect_err("duplicate assignees should fail");

        assert!(matches!(err, GitlaneError::InvalidIssue { .. }));
        assert!(err.to_string().contains("duplicate value `@bob`"));
    }

    #[test]
    fn rejects_updated_at_before_created_at() {
        let err = Issue::parse(
            concat!(
                "+++\n",
                "title = \"Document parser\"\n",
                "created_at = \"2026-02-27T10:08:15Z\"\n",
                "updated_at = \"2026-02-27T09:08:15Z\"\n",
                "reporter = \"@alice\"\n",
                "assignees = []\n",
                "priority = \"p2\"\n",
                "labels = []\n",
                "+++\n"
            ),
            Path::new("issue.md"),
        )
        .expect_err("backwards timestamps should fail");

        assert!(matches!(err, GitlaneError::InvalidIssue { .. }));
        assert!(err.to_string().contains("updated_at"));
    }

    #[test]
    fn rejects_blank_title() {
        let err = Issue::parse(
            concat!(
                "+++\n",
                "title = \"   \"\n",
                "created_at = \"2026-02-27T10:08:15Z\"\n",
                "updated_at = \"2026-02-27T10:08:15Z\"\n",
                "reporter = \"@alice\"\n",
                "assignees = []\n",
                "priority = \"p2\"\n",
                "labels = []\n",
                "+++\n"
            ),
            Path::new("issue.md"),
        )
        .expect_err("blank title should fail");

        assert!(matches!(err, GitlaneError::InvalidIssue { .. }));
        assert!(err.to_string().contains("title"));
    }

    #[test]
    fn front_matter_parser_accepts_default_templates() {
        for format in [
            ConfigFileExtension::Toml,
            ConfigFileExtension::Yaml,
            ConfigFileExtension::Json,
        ] {
            let template = templates::default(format);
            let parsed =
                frontmatter::parse(&template).expect("default template front matter should parse");

            assert_eq!(parsed.body, "");
        }
    }
}
