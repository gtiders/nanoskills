mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_list_reports_missing_index_without_crashing() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-missing-index");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    env.command(&workspace)
        .arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains("No local index found"));
}

#[test]
fn cli_search_reports_corrupted_index_without_entering_runtime_flow() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-corrupted-index");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    let cache_dir = env.cache_dir().join("nanoskills");
    fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
    fs::write(cache_dir.join("index.json"), "{ invalid json").expect("failed to write bad index");

    env.command(&workspace)
        .args(["search", "echo"])
        .assert()
        .success()
        .stderr(predicate::str::contains("The local index is unreadable"))
        .stderr(predicate::str::contains(
            "Run 'nanoskills sync' to rebuild it",
        ));
}

#[test]
fn cli_default_command_reports_corrupted_index_before_launching_tui() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-default-command");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    let cache_dir = env.cache_dir().join("nanoskills");
    fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
    fs::write(cache_dir.join("index.json"), "not json").expect("failed to write bad index");

    // 无子命令时本应进入 pick，但损坏索引必须先被拦截，避免进入 TUI。
    env.command(&workspace)
        .assert()
        .success()
        .stderr(predicate::str::contains("The local index is unreadable"));
}
