mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_search_json_outputs_machine_readable_tools_array() {
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

    env.run_sync(&workspace);

    let mut search = env.command(&workspace);

    let assert = search
        .args(["search", "echo", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("search --json should emit a valid JSON array");
    let tools = json.as_array().expect("top-level JSON should be an array");

    assert!(!tools.is_empty(), "search should return at least one tool");

    // 断言输出结构符合 OpenAI tools 约定，而不是只检查一段字符串。
    assert_eq!(tools[0]["type"], "function");
    assert_eq!(tools[0]["function"]["name"], "echo_skill");
    assert_eq!(tools[0]["function"]["description"], "Echo user input");
    assert_eq!(tools[0]["function"]["parameters"]["type"], "object");
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

    env.run_sync(&workspace);

    let assert = env
        .command(&workspace)
        .args(["search", "shared", "--json", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("search --json should emit valid JSON");
    let tools = json.as_array().expect("top-level JSON should be an array");

    // `--limit 1` 必须真的裁剪结果数量，不能只影响展示文案。
    assert_eq!(tools.len(), 1);
    // 同分场景下必须按名称排序，确保结果集在不同机器上保持稳定。
    assert_eq!(tools[0]["function"]["name"], "alpha_skill");
}
