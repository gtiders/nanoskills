# nanoskills

> **面向 AI Agent 的本地技能注册表 CLI。**  
> 把分散的脚本/提示词整理成可搜索、可导出的工具能力。

[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Latest Release](https://img.shields.io/github/v/release/gtiders/nanoskills?label=Release)](https://github.com/gtiders/nanoskills/releases/latest)
[![English README](https://img.shields.io/badge/README-English-blue.svg)](./README.md)

![Demo](assets/demo.gif)

## 定位与痛点

`nanoskills` 解决的是 Agent 工具链里最常见的一类问题：技能分散、难检索、难复用。

常见痛点：

- 技能文件散落，定位慢
- 工具元数据不统一，难直接给 LLM 调用
- Codex/Claude/OpenCode/OpenClaw 接入方式各异
- 团队共享技能包时目录结构不稳定

`nanoskills` 把流程统一成：扫描 -> 建索引 -> 检索 -> 导出 JSON。

## 安装

推荐：

- 最新 Release： https://github.com/gtiders/nanoskills/releases/latest

源码安装：

```bash
cargo install --path .
```

## 快速开始

```bash
nanoskills init
nanoskills sync
nanoskills search image
nanoskills search image --json
nanoskills pick
```

全局初始化会创建：

```text
~/.config/nanoskills/.agent-skills.yaml
~/.config/nanoskills/skills/
```

## 内置 Skills 目录结构

本仓库采用 **一技能一目录**，便于复制到其他运行时：

```text
skills/
  nanoskills_project_builder/
    nanoskills_project_builder.md
  nanoskills_usage_guide/
    nanoskills_usage_guide.md
```

解压 release 后，可直接把这些子目录复制到目标工具的 `skills/` 目录。

## 复制到其他运行时

示例（把目标路径替换为你的工具实际 skills 目录）：

```bash
cp -R ./skills/* <TOOL_SKILLS_DIR>/
```

此外，首次执行全局 `nanoskills init` 时，也会把 bundled/当前目录的 `skills/` 自动拷贝到 `~/.config/nanoskills/skills/`。

## 运行时接线示例

先导出 tools JSON：

```bash
nanoskills search image --json > .nanoskills.tools.json
```

<details>
<summary>OpenCode</summary>

```yaml
# 示例：按 OpenCode 当前 schema 调整
external_tools:
  source:
    type: command
    command: "nanoskills search image --json"
```

```bash
cp -R ./skills/* <OPENCODE_SKILLS_DIR>/
```

</details>

<details>
<summary>Codex</summary>

```yaml
# 示例：按 Codex runtime schema 调整
tools:
  command_source:
    cmd: ["nanoskills", "search", "image", "--json"]
```

```bash
cp -R ./skills/* <CODEX_SKILLS_DIR>/
```

</details>

<details>
<summary>Claude</summary>

```python
import json, subprocess

tools = json.loads(subprocess.check_output(
    ["nanoskills", "search", "image", "--json"],
    text=True,
))
# 把 tools 传给 Claude 工具定义
```

```bash
cp -R ./skills/* <CLAUDE_SKILLS_DIR>/
```

</details>

<details>
<summary>OpenClaw</summary>

```yaml
# 示例：按 OpenClaw 当前 schema 调整
tool_registry:
  provider: command
  command: "nanoskills search image --json"
```

```bash
cp -R ./skills/* <OPENCLAW_SKILLS_DIR>/
```

</details>

## 核心命令

- `nanoskills init` 创建全局配置和共享 skills 目录。
- `nanoskills init --local` 创建项目本地配置（`./.agent-skills.yaml`）。
- `nanoskills sync` 扫描路径并重建索引缓存。
- `nanoskills search <query> [--limit N]` 模糊搜索技能。
- `nanoskills search <query> --json` 导出可用于工具调用的 JSON。
- `nanoskills pick` 交互式 TUI 浏览。

## 配置模型

项目内运行时读取顺序：

1. 全局配置（`~/.config/nanoskills/.agent-skills.yaml`）
2. 本地配置（`./.agent-skills.yaml`，覆盖全局）

## 开发

```bash
cargo fmt
cargo clippy --all-targets --all-features -D warnings
cargo test
```

## FAQ

<details>
<summary>README 里带箭头的可展开部分怎么写？</summary>

```markdown
<details>
<summary>点击展开</summary>

这里放折叠内容。

</details>
```

GitHub 会自动渲染成可点击的小箭头。

</details>

## License

MIT
