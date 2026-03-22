mod common;

use common::TestEnv;
use std::fs;

#[test]
fn cli_sync_reads_global_and_local_config_together() {
    let env = TestEnv::new();
    let workspace = env.root().join("workspace-config-resolution");
    let local_skills_dir = workspace.join("local-skills");
    let global_skills_dir = env.global_config_dir().join("skills");

    fs::create_dir_all(&workspace).expect("failed to create workspace");
    fs::create_dir_all(&local_skills_dir).expect("failed to create local skills dir");
    fs::create_dir_all(&global_skills_dir).expect("failed to create global skills dir");

    fs::write(
        env.global_config_file(),
        r#"
scan_paths:
  - skills
"#,
    )
    .expect("failed to write global config");

    fs::write(
        workspace.join(".agent-skills.yaml"),
        r#"
scan_paths:
  - ./local-skills
"#,
    )
    .expect("failed to write local config");

    fs::write(
        global_skills_dir.join("global.py"),
        r#"# ---
# name: global_skill
# description: From global config
# ---
print("global")
"#,
    )
    .expect("failed to write global skill");

    fs::write(
        local_skills_dir.join("local.py"),
        r#"# ---
# name: local_skill
# description: From local config
# ---
print("local")
"#,
    )
    .expect("failed to write local skill");

    env.command(&workspace).arg("sync").assert().success();

    let index_path = env.cache_dir().join("nanoskills").join("index.json");
    let index_text = fs::read_to_string(index_path).expect("failed to read index");
    let index_json: serde_json::Value =
        serde_json::from_str(&index_text).expect("index should be valid JSON");
    let skills = index_json["skills"]
        .as_array()
        .expect("index.skills should be an array");

    // 解析逻辑必须同时读取全局和当前目录配置，两边的技能都应进入索引。
    assert_eq!(skills.len(), 2);
    assert!(skills.iter().any(|skill| skill["name"] == "global_skill"));
    assert!(skills.iter().any(|skill| skill["name"] == "local_skill"));
}
