mod common;

use common::TestEnv;
use predicates::prelude::*;
use std::fs;

#[test]
fn cli_sync_builds_index_and_skips_invalid_files() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace");
    let global_skills_dir = env.global_config_dir().join("skills");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&global_skills_dir).expect("failed to create global skills dir");

    fs::write(
        workspace.join("hello.py"),
        r#"# ---
# name: hello_skill
# description: Echo hello
# tags: [shell]
# ---
print("hello")
"#,
    )
    .expect("failed to write skill file");

    fs::write(
        workspace.join("README.md"),
        "This file has no nanoskills header.\n",
    )
    .expect("failed to write plain text file");

    fs::write(workspace.join("binary.bin"), b"abc\0def").expect("failed to write binary file");

    let mut cmd = env.command(&workspace);

    // 黑盒调用 sync，只断言行为结果，不依赖实现细节。
    cmd.arg("sync")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let index_path = env.cache_dir().join("nanoskills").join("index.json");
    assert!(index_path.exists(), "sync should create cache index");

    let index_text = fs::read_to_string(&index_path).expect("failed to read generated index");
    let index_json: serde_json::Value =
        serde_json::from_str(&index_text).expect("generated index should be valid JSON");

    let skills = index_json["skills"]
        .as_array()
        .expect("index.skills should be an array");

    // 只有合法 header 的文本技能应进入索引；无头文件和二进制文件都必须被跳过。
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0]["name"], "hello_skill");
    assert!(
        skills[0]["path"]
            .as_str()
            .expect("skill path should be a string")
            .ends_with("hello.py")
    );
}

#[test]
fn cli_sync_warns_when_scan_path_does_not_exist() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-missing-path");
    let global_skills_dir = env.global_config_dir().join("skills");
    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&global_skills_dir).expect("failed to create global skills dir");

    fs::write(
        workspace.join(".agent-skills.yaml"),
        r#"
scan_paths:
  - ./missing-skills
"#,
    )
    .expect("failed to write local config");

    env.command(&workspace)
        .arg("sync")
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Skipped scan path that does not exist",
        ))
        .stderr(predicate::str::contains("missing-skills"));
}

#[test]
fn cli_sync_strict_reports_parse_errors_and_excludes_invalid_skills() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-strict");
    fs::create_dir_all(&workspace).expect("failed to create workspace");

    fs::write(
        workspace.join("good.py"),
        r#"# ---
# name: good_skill
# description: Valid skill
# ---
print("good")
"#,
    )
    .expect("failed to write valid skill");

    fs::write(
        workspace.join("broken.py"),
        r#"# ---
# name: broken_skill
# description: [unterminated
# ---
print("broken")
"#,
    )
    .expect("failed to write broken skill");

    let assert = env
        .command(&workspace)
        .args(["sync", "--strict"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse errors"))
        .stdout(predicate::str::contains("broken.py"));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("stdout should be valid UTF-8");
    assert!(
        stdout.contains("indexed 1 skills"),
        "strict sync should only keep valid skills: {stdout}"
    );

    let index_path = env.cache_dir().join("nanoskills").join("index.json");
    let index_text = fs::read_to_string(&index_path).expect("failed to read generated index");
    let index_json: serde_json::Value =
        serde_json::from_str(&index_text).expect("generated index should be valid JSON");
    let skills = index_json["skills"]
        .as_array()
        .expect("index.skills should be an array");

    // strict 模式下，解析失败的技能不能进入最终索引。
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0]["name"], "good_skill");
}
