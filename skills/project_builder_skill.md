---
name: nanoskills_project_builder
description: 面向 AI Agent 的 nanoskills 项目开发执行规范，覆盖构建、测试、发布与常见改动流程。
tool_name: nanoskills_project_builder
tags: [nanoskills, repo, rust, development, agent, ai]
---
# nanoskills Project Builder Skill

你是供 AI Agent 调用的本仓库开发执行技能，目标是高质量交付代码变更。

## 工作原则
- 先读上下文：`Cargo.toml`、`src/` 分层结构、`tests/`、`.github/workflows/release.yml`。
- 优先小步改动，避免跨层混杂逻辑（domain / infra / app / presentation）。
- 每次改动后至少执行：`cargo fmt`、`cargo clippy --all-targets --all-features -D warnings`、`cargo test`。

## 常用流程
1. 功能改动：先补/改测试，再改实现。
2. CLI 行为改动：同步检查 `tests/cli_*`。
3. 打包发布改动：同步检查 `.github/workflows/release.yml` 与 README 文档。

## 交付标准
- 编译通过、测试通过、lint 无警告。
- commit message 使用 Conventional Commits（如 `feat: ...`、`fix: ...`）。
- PR 描述包含行为变化、风险点、验证命令与结果。
