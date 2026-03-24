# nanoskills

> **为 AI Agent 打造的极速本地技能库 CLI。**  
> 零配置扫盘，秒级建索引，原生输出工具 JSON。

[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Latest Release](https://img.shields.io/github/v/release/gtiders/nanoskills?label=Release)](https://github.com/gtiders/nanoskills/releases/latest)
[![English README](https://img.shields.io/badge/README-English-blue.svg)](./README.md)

## 安装

推荐多数用户直接下载二进制：

- 最新 Release： https://github.com/gtiders/nanoskills/releases/latest

从源码安装：

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

## 核心命令

- `nanoskills init` 创建全局配置和共享 skills 目录。
- `nanoskills init --local` 创建项目本地配置（`./.agent-skills.yaml`）。
- `nanoskills sync` 扫描路径并重建本地索引缓存。
- `nanoskills search <query> [--limit N]` 模糊搜索技能。
- `nanoskills search <query> --json` 导出可用于工具调用的 JSON。
- `nanoskills pick` 进入交互式 TUI 浏览。

## Agent 工具配置示例

先导出 JSON：

```bash
nanoskills search image --json > .nanoskills.tools.json
```

<details>
<summary>OpenCode 示例</summary>

```yaml
# 示例配置，字段名请按 OpenCode 当前版本调整
external_tools:
  source:
    type: command
    command: "nanoskills search image --json"
```

</details>

<details>
<summary>Codex 示例</summary>

```yaml
# 示例接线方式
tools:
  command_source:
    cmd: ["nanoskills", "search", "image", "--json"]
```

</details>

<details>
<summary>Claude 示例</summary>

```python
import json, subprocess

tools = json.loads(subprocess.check_output(
    ["nanoskills", "search", "image", "--json"],
    text=True,
))
# 将 tools 传入 Claude 的工具定义
```

</details>

<details>
<summary>OpenClaw 示例</summary>

```yaml
# 示例配置，请按 OpenClaw runtime 实际 schema 调整
tool_registry:
  provider: command
  command: "nanoskills search image --json"
```

</details>

## 配置模型

在项目目录中运行时，读取顺序为：

1. 全局配置（`~/.config/nanoskills/.agent-skills.yaml`）
2. 本地配置（`./.agent-skills.yaml`，覆盖全局）

这样可以全局共享技能，同时按项目覆盖扫描路径和限制参数。

## 工作原理

1. 基于 `ignore` 规则并行扫描文件。
2. 从脚本注释/头部解析 YAML skill 元数据。
3. 在 `~/.cache/nanoskills/` 生成索引缓存。
4. 通过 CLI、TUI、JSON 三种输出方式提供能力。

## 开发与贡献

```bash
cargo fmt
cargo clippy --all-targets --all-features -D warnings
cargo test
```

## FAQ

<details>
<summary>README 里带箭头的可展开部分怎么写？</summary>

使用 HTML 的 `details/summary`：

```markdown
<details>
<summary>点击展开</summary>

这里放折叠内容。

</details>
```

GitHub 会自动渲染为可点击的小箭头。

</details>

## License

MIT
