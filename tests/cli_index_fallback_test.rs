mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_list_auto_builds_index_when_cache_is_missing() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-missing-index");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("hello.py"),
        r#"# ---
# name: hello_skill
# description: hello
# ---
print("hello")
"#,
    )
    .expect("failed to write skill file");

    env.command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello_skill"));

    let index_path = env.cache_dir().join("skillscripts").join("index.json");
    assert!(
        index_path.exists(),
        "list should auto-build index when cache is missing"
    );
}

#[test]
fn cli_list_rebuilds_stale_cache_before_rendering() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-stale-index");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("one.py"),
        r#"# ---
# name: one_skill
# description: one
# ---
print("one")
"#,
    )
    .expect("failed to write first skill");

    env.command(&workspace).arg("sync").assert().success();

    fs::write(
        workspace.join("two.py"),
        r#"# ---
# name: two_skill
# description: two
# ---
print("two")
"#,
    )
    .expect("failed to write second skill");

    let index_path = env.cache_dir().join("skillscripts").join("index.json");
    let index_text = fs::read_to_string(&index_path).expect("failed to read index");
    let mut index_json: serde_json::Value =
        serde_json::from_str(&index_text).expect("index should be valid json");
    index_json["last_sync_unix"] = serde_json::json!(1);
    index_json["last_sync"] = serde_json::json!("1Z");
    fs::write(
        &index_path,
        serde_json::to_string_pretty(&index_json).expect("failed to serialize stale index"),
    )
    .expect("failed to write stale index");

    env.command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("one_skill"))
        .stdout(predicate::str::contains("two_skill"));
}

#[test]
fn cli_search_recovers_from_corrupted_index_by_rebuilding() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-corrupted-index");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("echo.py"),
        r#"# ---
# name: echo_skill
# description: Echo user input
# ---
print("echo")
"#,
    )
    .expect("failed to write skill file");

    let cache_dir = env.cache_dir().join("skillscripts");
    fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
    fs::write(cache_dir.join("index.json"), "{ invalid json").expect("failed to write bad index");

    env.command(&workspace)
        .args(["search", "echo"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["))
        .stdout(predicate::str::contains("\"name\": \"echo_skill\""));
}

#[test]
fn cli_default_command_recovers_from_corrupted_index_before_picker() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-default-command");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    let cache_dir = env.cache_dir().join("skillscripts");
    fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
    fs::write(cache_dir.join("index.json"), "not json").expect("failed to write bad index");

    // 无子命令时本应进入 pick，但损坏索引必须先被拦截，避免进入 TUI。
    env.command(&workspace)
        .assert()
        .success()
        .stderr(predicate::str::contains("No skill selected"));
}
