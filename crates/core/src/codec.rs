use std::path::Path;

use serde::{Serialize, de::DeserializeOwned};
use toml_edit::{DocumentMut, Item, Table, value};

use crate::{
    config::{ConfigFileExtension, ConfigValidationError},
    errors::GitlaneError,
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
    Repr: Serialize + TomlFormat + for<'a> From<&'a T>,
{
    let repr = Repr::from(value);
    let content = match ConfigFileExtension::from_path(config_path)? {
        ConfigFileExtension::Toml => {
            let mut document = toml_edit::ser::to_document(&repr)
                .map_err(|source| GitlaneError::serialize_config(config_path, source))?;
            repr.format_toml_document(&mut document);
            document.to_string()
        }
        ConfigFileExtension::Json => serde_json::to_string_pretty(&repr)
            .map_err(|source| GitlaneError::serialize_config(config_path, source))?,
        ConfigFileExtension::Yaml | ConfigFileExtension::Yml => serde_yaml::to_string(&repr)
            .map_err(|source| GitlaneError::serialize_config(config_path, source))?,
    };

    write_text_file(config_path, &content)?;
    Ok(())
}

pub(crate) trait TomlFormat {
    fn format_toml_document(&self, _document: &mut DocumentMut) {}
}

pub(crate) fn table_mut<'a>(document: &'a mut DocumentMut, key: &str) -> Option<&'a mut Table> {
    ensure_table(document.as_table_mut().get_mut(key)?)
}

pub(crate) fn ensure_table(item: &mut Item) -> Option<&mut Table> {
    if item.as_table_mut().is_none() {
        let current = std::mem::take(item);
        *item = match current {
            Item::Value(toml_edit::Value::InlineTable(inline_table)) => {
                Item::Table(inline_table.into_table())
            }
            other => other,
        };
    }

    item.as_table_mut()
}

pub(crate) fn inline_child_tables(table: &mut Table) {
    for (_, item) in table.iter_mut() {
        let current = std::mem::take(item);
        *item = match current {
            Item::Table(table) => value(table.into_inline_table()),
            other => other,
        };
    }
}
