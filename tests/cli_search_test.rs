mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_search_json_outputs_lightweight_skill_array() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("echo.py"),
        r#"# ---
# name: echo_skill
# description: Echo user input
# tool_name: echo_skill
# args:
#   message:
#     type: string
#     description: Message to echo
#     required: true
# ---
print("echo")
"#,
    )
    .expect("failed to write skill file");

    fs::write(
        workspace.join("image.py"),
        r#"# ---
# name: image_skill
# description: Generate an image
# ---
print("image")
"#,
    )
    .expect("failed to write second skill file");

    env.command(&workspace).arg("sync").assert().success();

    let mut search = env.command(&workspace);

    let assert = search
        .args(["search", "echo"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("search should emit a valid JSON array");
    let skills = json.as_array().expect("top-level JSON should be an array");

    assert!(!skills.is_empty(), "search should return at least one skill");

    assert_eq!(skills[0]["name"], "echo_skill");
    assert_eq!(skills[0]["description"], "Echo user input");
    let expected_path = workspace.join("echo.py").to_string_lossy().replace('\\', "/");
    assert_eq!(
        skills[0]["path"],
        serde_json::Value::String(expected_path)
    );
    assert!(skills[0]["tags"].is_array());
    assert!(skills[0].get("parameters").is_none());
}

#[test]
fn cli_search_json_respects_limit_and_keeps_result_order_stable() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-search-limit");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("alpha.py"),
        r#"# ---
# name: alpha_skill
# description: shared description
# tool_name: alpha_skill
# ---
print("alpha")
"#,
    )
    .expect("failed to write alpha skill");

    fs::write(
        workspace.join("beta.py"),
        r#"# ---
# name: beta_skill
# description: shared description
# tool_name: beta_skill
# ---
print("beta")
"#,
    )
    .expect("failed to write beta skill");

    env.command(&workspace).arg("sync").assert().success();

    let assert = env
        .command(&workspace)
        .args(["search", "shared", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("search should emit valid JSON");
    let skills = json.as_array().expect("top-level JSON should be an array");

    // `--limit 1` 必须真的裁剪结果数量，不能只影响展示文案。
    assert_eq!(skills.len(), 1);
    // 同分场景下必须按名称排序，确保结果集在不同机器上保持稳定。
    assert_eq!(skills[0]["name"], "alpha_skill");
}
