mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_list_json_outputs_plain_machine_readable_skill_array() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("alpha.py"),
        r#"# ---
# name: alpha_skill
# description: First skill
# tags: [alpha, beta]
# ---
print("alpha")
"#,
    )
    .expect("failed to write first skill");

    fs::write(
        workspace.join("beta.py"),
        r#"# ---
# name: beta_skill
# description: Second skill
# ---
print("beta")
"#,
    )
    .expect("failed to write second skill");

    env.run_sync(&workspace);

    let assert = env
        .command(&workspace)
        .args(["list", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("list --json should emit a valid JSON array");
    let skills = json.as_array().expect("top-level JSON should be an array");

    // `list --json` 是给机器消费的输出，必须是无 ANSI 污染的纯 JSON 数组。
    assert_eq!(skills.len(), 2);
    assert_eq!(skills[0]["name"], "alpha_skill");
    assert_eq!(skills[1]["name"], "beta_skill");
    assert_eq!(skills[0]["description"], "First skill");
}
