mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_sync_non_strict_deduplicates_duplicate_tool_names() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-tool-name-nonstrict");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("alpha.py"),
        r#"# ---
# name: alpha_skill
# description: First skill
# tool_name: duplicate_tool
# ---
print("alpha")
"#,
    )
    .expect("failed to write alpha skill");

    fs::write(
        workspace.join("beta.py"),
        r#"# ---
# name: beta_skill
# description: Second skill
# tool_name: duplicate_tool
# ---
print("beta")
"#,
    )
    .expect("failed to write beta skill");

    env.command(&workspace)
        .arg("sync")
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Duplicate tool_name 'duplicate_tool'",
        ));

    let index_path = env.cache_dir().join("skillscripts").join("index.json");
    let index_text = fs::read_to_string(index_path).expect("failed to read index");
    let index_json: serde_json::Value =
        serde_json::from_str(&index_text).expect("index should be valid JSON");
    let skills = index_json["skills"]
        .as_array()
        .expect("index.skills should be an array");

    assert_eq!(skills.len(), 2);
    // non-strict 下应自动加 hash 去重，而不是丢弃冲突技能。
    assert_ne!(skills[0]["tool_name"], skills[1]["tool_name"]);
    assert!(
        skills[0]["tool_name"]
            .as_str()
            .expect("tool_name should be a string")
            .starts_with("duplicate_tool_")
    );
    assert!(
        skills[1]["tool_name"]
            .as_str()
            .expect("tool_name should be a string")
            .starts_with("duplicate_tool_")
    );
}

#[test]
fn cli_sync_strict_rejects_duplicate_tool_names_from_index() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-tool-name-strict");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("alpha.py"),
        r#"# ---
# name: alpha_skill
# description: First skill
# tool_name: duplicate_tool
# ---
print("alpha")
"#,
    )
    .expect("failed to write alpha skill");

    fs::write(
        workspace.join("beta.py"),
        r#"# ---
# name: beta_skill
# description: Second skill
# tool_name: duplicate_tool
# ---
print("beta")
"#,
    )
    .expect("failed to write beta skill");

    let assert = env
        .command(&workspace)
        .args(["sync", "--strict"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Duplicate tool_name 'duplicate_tool'.",
        ));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    assert!(
        stdout.contains("indexed 0 skills"),
        "strict sync should exclude duplicate tool names from index: {stdout}"
    );

    let index_path = env.cache_dir().join("skillscripts").join("index.json");
    let index_text = fs::read_to_string(index_path).expect("failed to read index");
    let index_json: serde_json::Value =
        serde_json::from_str(&index_text).expect("index should be valid JSON");
    let skills = index_json["skills"]
        .as_array()
        .expect("index.skills should be an array");

    assert!(
        skills.is_empty(),
        "strict mode should drop duplicate tool names"
    );
}
