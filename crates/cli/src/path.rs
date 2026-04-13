//! Helpers for resolving the active Gitlane project directory.

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow, bail};
use gitlane::{
    config::{ConfigKind, config_candidate_paths, config_file_names},
    paths::GITLANE_DIR,
};

/// Resolves the nearest `.gitlane` directory from `start_path` or its parents.
pub fn resolve_project(start_path: &Path) -> anyhow::Result<PathBuf> {
    let start = start_path
        .canonicalize()
        .with_context(|| format!("failed to resolve `{}`", start_path.display()))?;
    let start_display = start.display().to_string();

    let mut cursor = if start.is_file() {
        start
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| anyhow!("`{}` has no parent directory", start.display()))?
    } else {
        start
    };

    loop {
        if is_gitlane_dir(&cursor) {
            return validate_project(cursor);
        }

        let candidate = cursor.join(GITLANE_DIR);
        if candidate.is_dir() {
            return validate_project(candidate);
        }

        let Some(parent) = cursor.parent() else {
            break;
        };

        cursor = parent.to_path_buf();
    }

    bail!(
        "unable to find `{}` from `{}` or any parent directory",
        GITLANE_DIR,
        start_display
    )
}

fn is_gitlane_dir(path: &Path) -> bool {
    path.file_name() == Some(OsStr::new(GITLANE_DIR)) && path.is_dir()
}

fn validate_project(project: PathBuf) -> anyhow::Result<PathBuf> {
    let mut project_configs = Vec::new();

    for project_config in config_candidate_paths(&project, ConfigKind::Project) {
        if project_config.exists() {
            if !project_config.is_file() {
                bail!(
                    "found project directory `{}` but expected file `{}`",
                    project.display(),
                    project_config.display()
                );
            }

            project_configs.push(project_config);
        }
    }

    match project_configs.len() {
        0 => {
            let supported_project_configs = config_file_names(ConfigKind::Project)
                .map(|file_name| format!("`{file_name}`"))
                .join(", ");
            bail!(
                "found project directory `{}` but missing a supported project config file ({})",
                project.display(),
                supported_project_configs
            );
        }
        1 => {}
        _ => {
            let formatted_paths = project_configs
                .iter()
                .map(|path| format!("`{}`", path.display()))
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "found project directory `{}` but multiple supported project config files exist: {}",
                project.display(),
                formatted_paths
            );
        }
    }

    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use tempfile::TempDir;

    use gitlane::config::{ConfigFileExtension, ConfigKind, config_path};

    fn create_project_with_config(base: &Path, extension: ConfigFileExtension) -> PathBuf {
        let gitlane_dir = base.join(GITLANE_DIR);

        fs::create_dir_all(&gitlane_dir).expect(".gitlane directory should be created");
        fs::write(
            config_path(&gitlane_dir, ConfigKind::Project, extension),
            "",
        )
        .expect("project config should be created");

        gitlane_dir
            .canonicalize()
            .expect(".gitlane path should canonicalize")
    }

    #[test]
    fn resolves_project_when_start_path_contains_gitlane_dir() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let gitlane_dir = create_project_with_config(temp_dir.path(), ConfigFileExtension::Toml);
        let project_dir = gitlane_dir.parent().expect(".gitlane should have a parent");

        let resolved = resolve_project(project_dir).expect("project should resolve");

        assert_eq!(resolved, gitlane_dir);
    }

    #[test]
    fn resolves_project_when_start_path_is_gitlane_dir() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let gitlane_dir = create_project_with_config(temp_dir.path(), ConfigFileExtension::Toml);

        let resolved = resolve_project(&gitlane_dir).expect("project should resolve");

        assert_eq!(resolved, gitlane_dir);
    }

    #[test]
    fn resolves_project_from_nested_directory() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let gitlane_dir = create_project_with_config(temp_dir.path(), ConfigFileExtension::Toml);
        let nested = gitlane_dir
            .parent()
            .expect(".gitlane should have a parent")
            .join("src")
            .join("feature");
        fs::create_dir_all(&nested).expect("nested directory should be created");

        let resolved = resolve_project(&nested).expect("project should resolve");

        assert_eq!(resolved, gitlane_dir);
    }

    #[test]
    fn resolves_project_from_file_path() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let gitlane_dir = create_project_with_config(temp_dir.path(), ConfigFileExtension::Toml);
        let nested_dir = gitlane_dir
            .parent()
            .expect(".gitlane should have a parent")
            .join("src");
        fs::create_dir_all(&nested_dir).expect("nested directory should be created");
        let file_path = nested_dir.join("input.txt");
        fs::write(&file_path, "data").expect("input file should be created");

        let resolved = resolve_project(&file_path).expect("project should resolve");

        assert_eq!(resolved, gitlane_dir);
    }

    #[test]
    fn errors_when_gitlane_dir_is_missing() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");

        let err = resolve_project(temp_dir.path()).expect_err("resolution should fail");

        assert!(err.to_string().contains("unable to find `.gitlane`"));
    }

    #[test]
    fn resolves_project_when_project_yaml_exists() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let gitlane_dir = create_project_with_config(temp_dir.path(), ConfigFileExtension::Yaml);

        let resolved = resolve_project(temp_dir.path()).expect("project should resolve");

        assert_eq!(resolved, gitlane_dir);
    }

    #[test]
    fn resolves_project_when_project_yml_exists() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let gitlane_dir = create_project_with_config(temp_dir.path(), ConfigFileExtension::Yml);

        let resolved = resolve_project(temp_dir.path()).expect("project should resolve");

        assert_eq!(resolved, gitlane_dir);
    }

    #[test]
    fn resolves_project_when_project_json_exists() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let gitlane_dir = create_project_with_config(temp_dir.path(), ConfigFileExtension::Json);

        let resolved = resolve_project(temp_dir.path()).expect("project should resolve");

        assert_eq!(resolved, gitlane_dir);
    }

    #[test]
    fn errors_when_supported_project_config_is_missing() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let project_dir = temp_dir.path();
        let gitlane_dir = project_dir.join(GITLANE_DIR);
        fs::create_dir_all(&gitlane_dir).expect(".gitlane directory should be created");

        let err = resolve_project(project_dir).expect_err("resolution should fail");
        let err_text = err.to_string();

        assert!(err_text.contains("missing"));
        assert!(err_text.contains("project.toml"));
        assert!(err_text.contains("project.json"));
        assert!(err_text.contains("project.yaml"));
        assert!(err_text.contains("project.yml"));
    }

    #[test]
    fn errors_when_multiple_supported_project_configs_exist() {
        let temp_dir = TempDir::new().expect("temp test directory should be created");
        let gitlane_dir = temp_dir.path().join(GITLANE_DIR);
        fs::create_dir_all(&gitlane_dir).expect(".gitlane directory should be created");
        fs::write(
            config_path(&gitlane_dir, ConfigKind::Project, ConfigFileExtension::Toml),
            "",
        )
        .expect("project.toml should be created");
        fs::write(
            config_path(&gitlane_dir, ConfigKind::Project, ConfigFileExtension::Yaml),
            "",
        )
        .expect("project.yaml should be created");

        let err = resolve_project(temp_dir.path()).expect_err("resolution should fail");
        let err_text = err.to_string();

        assert!(err_text.contains("multiple supported project config files"));
        assert!(err_text.contains("project.toml"));
        assert!(err_text.contains("project.yaml"));
    }
}
