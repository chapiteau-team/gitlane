use std::{ops::Range, str::FromStr};

use serde::de::DeserializeOwned;
use serde_json::{Map as JsonMap, Value as JsonValue};
use serde_yaml::{Mapping as YamlMapping, Value as YamlValue};
use thiserror::Error;
use toml_edit::{Array as TomlArray, DocumentMut, value as toml_value};

pub(crate) const TOML_FENCE: &str = "+++";
pub(crate) const YAML_FENCE: &str = "---";
pub(crate) const JSON_OBJECT_START: char = '{';
pub(crate) const JSON_OBJECT_END: char = '}';

/// Parser-specific errors for supported front matter formats.
#[derive(Debug, Error)]
pub enum FrontmatterParseError {
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    TomlDocument(#[from] toml_edit::TomlError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

impl FrontmatterParseError {
    pub(crate) fn format_name(&self) -> &'static str {
        match self {
            Self::Toml(_) => "TOML",
            Self::TomlDocument(_) => "TOML",
            Self::Json(_) => "JSON",
            Self::Yaml(_) => "YAML",
        }
    }
}

/// Serializer-specific errors for supported front matter formats.
#[derive(Debug, Error)]
pub enum FrontmatterSerializeError {
    #[error(transparent)]
    TimeFormat(#[from] time::error::Format),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

impl FrontmatterSerializeError {
    pub(crate) fn format_name(&self) -> &'static str {
        match self {
            Self::TimeFormat(_) => "front matter",
            Self::Json(_) => "JSON",
            Self::Yaml(_) => "YAML",
        }
    }
}

/// Validation error for parsed front matter content.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub(crate) struct FrontmatterValidationError {
    message: String,
}

impl FrontmatterValidationError {
    /// Creates a new validation error with a user-facing message.
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum FrontmatterError {
    #[error(transparent)]
    Validation(#[from] FrontmatterValidationError),
    #[error(transparent)]
    Parse(#[from] FrontmatterParseError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontmatterFormat {
    Toml,
    Json,
    Yaml,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedFrontmatter {
    pub(crate) document: FrontmatterDocument,
    pub(crate) body: String,
}

#[derive(Debug, Clone)]
pub(crate) enum FrontmatterDocument {
    Toml(DocumentMut),
    Json(JsonMap<String, JsonValue>),
    Yaml(YamlMapping),
}

impl FrontmatterDocument {
    pub(crate) fn format(&self) -> FrontmatterFormat {
        match self {
            Self::Toml(_) => FrontmatterFormat::Toml,
            Self::Json(_) => FrontmatterFormat::Json,
            Self::Yaml(_) => FrontmatterFormat::Yaml,
        }
    }

    pub(crate) fn deserialize<T>(&self) -> Result<T, FrontmatterParseError>
    where
        T: DeserializeOwned,
    {
        match self {
            Self::Toml(document) => toml::from_str(&document.to_string()).map_err(Into::into),
            Self::Json(object) => {
                serde_json::from_value(JsonValue::Object(object.clone())).map_err(Into::into)
            }
            Self::Yaml(mapping) => {
                serde_yaml::from_value(YamlValue::Mapping(mapping.clone())).map_err(Into::into)
            }
        }
    }

    pub(crate) fn set_string(&mut self, key: &str, value: &str) {
        match self {
            Self::Toml(document) => {
                document[key] = toml_value(value);
            }
            Self::Json(object) => {
                object.insert(key.to_owned(), JsonValue::String(value.to_owned()));
            }
            Self::Yaml(mapping) => {
                mapping.insert(
                    YamlValue::String(key.to_owned()),
                    YamlValue::String(value.to_owned()),
                );
            }
        }
    }

    pub(crate) fn set_string_list(&mut self, key: &str, values: &[String]) {
        match self {
            Self::Toml(document) => {
                let mut array = TomlArray::new();
                for value in values {
                    array.push(value.as_str());
                }
                document[key] = toml_value(array);
            }
            Self::Json(object) => {
                object.insert(
                    key.to_owned(),
                    JsonValue::Array(
                        values
                            .iter()
                            .map(|value| JsonValue::String(value.clone()))
                            .collect(),
                    ),
                );
            }
            Self::Yaml(mapping) => {
                mapping.insert(
                    YamlValue::String(key.to_owned()),
                    YamlValue::Sequence(
                        values
                            .iter()
                            .map(|value| YamlValue::String(value.clone()))
                            .collect(),
                    ),
                );
            }
        }
    }

    pub(crate) fn render(&self, body: &str) -> Result<String, FrontmatterSerializeError> {
        match self {
            Self::Toml(document) => {
                let mut frontmatter = document.to_string();
                ensure_trailing_newline(&mut frontmatter);

                Ok(format!("{TOML_FENCE}\n{frontmatter}{TOML_FENCE}\n{body}"))
            }
            Self::Json(object) => Ok(format!("{}{}", serde_json::to_string_pretty(object)?, body)),
            Self::Yaml(mapping) => {
                let mut frontmatter = serde_yaml::to_string(mapping)?;

                if let Some(stripped) = frontmatter.strip_prefix("---\n") {
                    frontmatter = stripped.to_owned();
                }

                ensure_trailing_newline(&mut frontmatter);

                Ok(format!("{YAML_FENCE}\n{frontmatter}{YAML_FENCE}\n{body}"))
            }
        }
    }
}

pub(crate) fn parse(content: &str) -> Result<ParsedFrontmatter, FrontmatterError> {
    if content.starts_with(TOML_FENCE) {
        return parse_toml(content);
    }

    if content.starts_with(YAML_FENCE) {
        return parse_yaml(content);
    }

    if content.starts_with(JSON_OBJECT_START) {
        return parse_json(content);
    }

    Err(FrontmatterValidationError::new(
        format!(
            "document must start with TOML (`{TOML_FENCE}`), YAML (`{YAML_FENCE}`), or JSON (`{JSON_OBJECT_START}`) front matter"
        ),
    )
    .into())
}

fn parse_toml(content: &str) -> Result<ParsedFrontmatter, FrontmatterError> {
    let (frontmatter, body) = parse_fenced_frontmatter(content, TOML_FENCE)?;
    let document = DocumentMut::from_str(frontmatter).map_err(FrontmatterParseError::from)?;

    Ok(ParsedFrontmatter {
        document: FrontmatterDocument::Toml(document),
        body: body.to_owned(),
    })
}

fn parse_yaml(content: &str) -> Result<ParsedFrontmatter, FrontmatterError> {
    let (frontmatter, body) = parse_fenced_frontmatter(content, YAML_FENCE)?;
    let mapping = serde_yaml::from_str(frontmatter).map_err(FrontmatterParseError::from)?;

    Ok(ParsedFrontmatter {
        document: FrontmatterDocument::Yaml(mapping),
        body: body.to_owned(),
    })
}

fn parse_json(content: &str) -> Result<ParsedFrontmatter, FrontmatterError> {
    let mut stream = serde_json::Deserializer::from_str(content).into_iter::<JsonValue>();
    let value = stream
        .next()
        .transpose()
        .map_err(FrontmatterParseError::from)?
        .ok_or_else(|| FrontmatterValidationError::new("document is missing JSON front matter"))?;
    let offset = stream.byte_offset();

    let object = match value {
        JsonValue::Object(object) => object,
        _ => {
            return Err(FrontmatterValidationError::new(
                "JSON front matter must be a top-level object",
            )
            .into());
        }
    };

    Ok(ParsedFrontmatter {
        document: FrontmatterDocument::Json(object),
        body: if content[offset..].trim().is_empty() {
            String::new()
        } else {
            content[offset..].to_owned()
        },
    })
}

fn parse_fenced_frontmatter<'a>(
    content: &'a str,
    delimiter: &str,
) -> Result<(&'a str, &'a str), FrontmatterValidationError> {
    let Some((first_line, frontmatter_start)) = next_line_range(content, 0) else {
        return Err(FrontmatterValidationError::new("document is empty"));
    };

    if trim_line_ending(&content[first_line.clone()]) != delimiter {
        return Err(FrontmatterValidationError::new(format!(
            "document must start with `{delimiter}` front matter"
        )));
    }

    let mut offset = frontmatter_start;
    while let Some((line_range, next_offset)) = next_line_range(content, offset) {
        if trim_line_ending(&content[line_range.clone()]) == delimiter {
            return Ok((
                &content[frontmatter_start..line_range.start],
                &content[next_offset..],
            ));
        }

        offset = next_offset;
    }

    Err(FrontmatterValidationError::new(format!(
        "document is missing closing `{delimiter}` front matter fence"
    )))
}

fn next_line_range(content: &str, start: usize) -> Option<(Range<usize>, usize)> {
    if start >= content.len() {
        return None;
    }

    let remainder = &content[start..];
    if let Some(newline_offset) = remainder.find('\n') {
        let end = start + newline_offset + 1;
        return Some((start..end, end));
    }

    Some((start..content.len(), content.len()))
}

fn trim_line_ending(line: &str) -> &str {
    let line = line.strip_suffix('\n').unwrap_or(line);
    line.strip_suffix('\r').unwrap_or(line)
}

fn ensure_trailing_newline(content: &mut String) {
    if !content.ends_with('\n') {
        content.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct ExampleMetadata {
        title: String,
        labels: Vec<String>,
    }

    #[test]
    fn parses_and_updates_toml_frontmatter() {
        let parsed = parse(concat!(
            "+++\n",
            "title = \"Old\"\n",
            "labels = [\"a\"]\n",
            "extra = \"keep\"\n",
            "+++\n",
            "Body\n"
        ))
        .expect("front matter should parse");

        let mut document = parsed.document;
        document.set_string("title", "New");
        document.set_string_list("labels", &["a".to_owned(), "b".to_owned()]);

        let metadata: ExampleMetadata = document
            .deserialize()
            .expect("typed metadata should deserialize");
        let rendered = document
            .render(&parsed.body)
            .expect("front matter should render");

        assert_eq!(metadata.title, "New");
        assert_eq!(metadata.labels, vec!["a".to_string(), "b".to_string()]);
        assert!(rendered.contains("extra = \"keep\""));
        assert!(rendered.ends_with("Body\n"));
    }

    #[test]
    fn parses_json_frontmatter_with_no_body() {
        let parsed = parse("{\n  \"title\": \"Example\",\n  \"labels\": []\n}\n")
            .expect("json front matter should parse");

        assert_eq!(parsed.document.format(), FrontmatterFormat::Json);
        assert_eq!(parsed.body, "");
    }

    #[test]
    fn rejects_missing_frontmatter_fence_with_invalid_frontmatter_error() {
        let err = parse("title = \"Missing fence\"\n").expect_err("missing fence should fail");

        assert!(matches!(err, FrontmatterError::Validation(_)));
    }

    #[test]
    fn rejects_invalid_toml_frontmatter_with_parse_error() {
        let err = parse(concat!(
            "+++\n",
            "title = \"Example\"\n",
            "labels = [\n",
            "+++\n"
        ))
        .expect_err("invalid TOML should fail");

        assert!(matches!(err, FrontmatterError::Parse(_)));
    }
}
