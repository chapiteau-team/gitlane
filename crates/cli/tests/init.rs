//! End-to-end integration tests for the `gitlane init` command.
//!
//! These exercise the actual binary via `assert_cmd`, focusing on CLI-surface
//! behavior (clap parsing, exit codes, stdout, the `--project` global flag,
//! cwd-based name inference, the success summary). Layout/contents of the
//! generated project are covered by unit tests in `gitlane::init`.

use std::fs;

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

fn gitlane() -> Command {
    Command::cargo_bin("gitlane").expect("gitlane binary should be built by cargo")
}

#[test]
fn init_with_explicit_name_succeeds() {
    let temp = TempDir::new().expect("temp directory should be created");

    gitlane()
        .arg("--project")
        .arg(temp.path())
        .args(["init", "--name", "demo"])
        .assert()
        .success()
        .stdout(contains("Initialized gitlane project \"demo\""))
        .stdout(contains("(format: toml)"));

    assert!(temp.path().join(".gitlane/project.toml").is_file());
    // Sanity check that init actually scaffolded issue layout, not just printed.
    assert!(temp.path().join(".gitlane/issues/workflow.toml").is_file());
}

#[test]
fn init_infers_name_from_project_directory() {
    let temp = TempDir::new().expect("temp directory should be created");
    let project_root = temp.path().join("inferred-name");
    fs::create_dir_all(&project_root).expect("project root should be created");

    gitlane()
        .arg("--project")
        .arg(&project_root)
        .arg("init")
        .assert()
        .success()
        .stdout(contains("Initialized gitlane project \"inferred-name\""));

    let config = fs::read_to_string(project_root.join(".gitlane/project.toml"))
        .expect("project.toml should be readable");
    assert!(
        config.contains("name = \"inferred-name\""),
        "expected name in config: {config}"
    );
}

#[test]
fn init_creates_json_config_when_format_json() {
    let temp = TempDir::new().expect("temp directory should be created");

    gitlane()
        .arg("--project")
        .arg(temp.path())
        .args(["init", "--name", "demo", "--format", "json"])
        .assert()
        .success()
        .stdout(contains("(format: json)"));

    assert!(temp.path().join(".gitlane/project.json").is_file());
    assert!(!temp.path().join(".gitlane/project.toml").exists());
}

#[test]
fn init_creates_yaml_config_when_format_yaml() {
    let temp = TempDir::new().expect("temp directory should be created");

    gitlane()
        .arg("--project")
        .arg(temp.path())
        .args(["init", "--name", "demo", "--format", "yaml"])
        .assert()
        .success()
        .stdout(contains("(format: yaml)"));

    assert!(temp.path().join(".gitlane/project.yaml").is_file());
}

#[test]
fn init_creates_yml_config_when_format_yml() {
    let temp = TempDir::new().expect("temp directory should be created");

    gitlane()
        .arg("--project")
        .arg(temp.path())
        .args(["init", "--name", "demo", "--format", "yml"])
        .assert()
        .success()
        .stdout(contains("(format: yml)"));

    assert!(temp.path().join(".gitlane/project.yml").is_file());
}

#[test]
fn init_fails_when_project_already_initialized() {
    let temp = TempDir::new().expect("temp directory should be created");

    gitlane()
        .arg("--project")
        .arg(temp.path())
        .args(["init", "--name", "first"])
        .assert()
        .success();

    gitlane()
        .arg("--project")
        .arg(temp.path())
        .args(["init", "--name", "second"])
        .assert()
        .failure();
}

#[test]
fn init_writes_description_and_homepage_when_provided() {
    let temp = TempDir::new().expect("temp directory should be created");

    gitlane()
        .arg("--project")
        .arg(temp.path())
        .args([
            "init",
            "--name",
            "demo",
            "--description",
            "Issue tracker",
            "--homepage",
            "https://example.com",
        ])
        .assert()
        .success();

    let config = fs::read_to_string(temp.path().join(".gitlane/project.toml"))
        .expect("project.toml should be readable");
    assert!(
        config.contains("description = \"Issue tracker\""),
        "expected description in config: {config}"
    );
    assert!(
        config.contains("homepage = \"https://example.com\""),
        "expected homepage in config: {config}"
    );
}

#[test]
fn init_rejects_empty_name() {
    let temp = TempDir::new().expect("temp directory should be created");

    gitlane()
        .arg("--project")
        .arg(temp.path())
        .args(["init", "--name", "   "])
        .assert()
        .failure();

    assert!(!temp.path().join(".gitlane/project.toml").exists());
}
