mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_init_creates_default_config_in_current_workspace() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    env.command(&workspace)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    let config_path = workspace.join(".agent-skills.yaml");
    assert!(
        config_path.exists(),
        "init should create a local config file"
    );

    let yaml = fs::read_to_string(config_path).expect("failed to read generated config");
    let config: serde_yaml::Value =
        serde_yaml::from_str(&yaml).expect("generated config should be valid YAML");

    // 默认配置文件必须可解析，并包含最基础的发布默认值。
    assert_eq!(config["scan_paths"][0].as_str(), Some("."));
    assert_eq!(config["max_file_size"].as_str(), Some("1MB"));
    assert_eq!(config["search_limit"].as_i64(), Some(5));
}

#[test]
fn cli_init_rejects_existing_config_without_force() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init-exists");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    let config_path = workspace.join(".agent-skills.yaml");
    fs::write(&config_path, "search_limit: 99\n").expect("failed to seed config");

    env.command(&workspace)
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "A configuration file already exists",
        ));

    // 不带 --force 时必须保留用户现有配置，不允许悄悄覆盖。
    let current = fs::read_to_string(config_path).expect("failed to read existing config");
    assert_eq!(current, "search_limit: 99\n");
}

#[test]
fn cli_init_force_overwrites_existing_config() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-init-force");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    let config_path = workspace.join(".agent-skills.yaml");
    fs::write(&config_path, "search_limit: 99\n").expect("failed to seed config");

    env.command(&workspace)
        .args(["init", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    let yaml = fs::read_to_string(config_path).expect("failed to read overwritten config");
    let config: serde_yaml::Value =
        serde_yaml::from_str(&yaml).expect("overwritten config should be valid YAML");

    assert_eq!(config["scan_paths"][0].as_str(), Some("."));
    assert_eq!(config["max_file_size"].as_str(), Some("1MB"));
    assert_eq!(config["search_limit"].as_i64(), Some(5));
}
