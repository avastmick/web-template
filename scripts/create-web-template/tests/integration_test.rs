use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_new_project_creation() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test-project";
    let project_path = temp_dir.path().join(project_name);

    // Run the CLI command
    let mut cmd = Command::cargo_bin("create-web-template").unwrap();
    cmd.args(&[
        "new",
        project_name,
        "--path",
        project_path.to_str().unwrap(),
        "--no-interactive",
    ])
    .assert()
    .success();

    // Check that the project was created
    assert!(project_path.exists());
    assert!(project_path.join("justfile").exists());
    assert!(project_path.join("server/Cargo.toml").exists());
    assert!(project_path.join("client/package.json").exists());
    assert!(project_path.join(".web-template.json").exists());

    // Check metadata file
    let metadata_content = fs::read_to_string(project_path.join(".web-template.json")).unwrap();
    assert!(metadata_content.contains("test-project"));
}

#[test]
fn test_config_show_command() {
    let mut cmd = Command::cargo_bin("create-web-template").unwrap();
    cmd.args(&["config", "show"]).assert().success();
}

#[test]
fn test_config_features_command() {
    let mut cmd = Command::cargo_bin("create-web-template").unwrap();
    cmd.args(&["config", "features"])
        .assert()
        .success()
        .stdout(predicates::str::contains("local_auth"))
        .stdout(predicates::str::contains("google_auth"))
        .stdout(predicates::str::contains("stripe_payment"));
}

#[test]
fn test_dry_run_mode() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test-dry-run";
    let project_path = temp_dir.path().join(project_name);

    let mut cmd = Command::cargo_bin("create-web-template").unwrap();
    cmd.args(&[
        "new",
        project_name,
        "--path",
        project_path.to_str().unwrap(),
        "--no-interactive",
        "--dry-run",
    ])
    .assert()
    .success();

    // In dry-run mode, no files should be created
    assert!(!project_path.exists());
}

#[test]
fn test_variable_substitution() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "my-awesome-project";
    let project_path = temp_dir.path().join(project_name);

    let mut cmd = Command::cargo_bin("create-web-template").unwrap();
    cmd.args(&[
        "new",
        project_name,
        "--path",
        project_path.to_str().unwrap(),
        "--no-interactive",
    ])
    .assert()
    .success();

    // Check that project name was substituted in files
    let cargo_content = fs::read_to_string(project_path.join("server/Cargo.toml")).unwrap();
    assert!(cargo_content.contains("my-awesome-project"));
    assert!(!cargo_content.contains("web-template"));

    let package_json_content =
        fs::read_to_string(project_path.join("client/package.json")).unwrap();
    assert!(package_json_content.contains("my-awesome-project"));
}
