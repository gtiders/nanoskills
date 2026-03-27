# nanoskills

面向 AI Agent 深度集成场景的本地高速 Skill 检索 CLI。

## 全面介绍

`nanoskills` 重点解决两类核心问题。

### 1）面向 Agent 工作流的高速 Skill 检索

它会从配置路径构建本地可搜索索引，让检索链路稳定且可复现：

- 基于全局/本地合并配置扫描技能文件
- 解析并标准化元数据（`name`、`description`、`tool_name`、`args`、`tags`）
- 构建本地缓存索引，实现低延迟检索
- 输出稳定 JSON，直接对接工具调用

这让 `skill find` 在真实 Agent 循环里更快、更稳定。

### 2）Skill 系统不止支持 Markdown

Skill 不局限于 `.md`。任何脚本/文件，只要按该语言注释风格写入合法 YAML 头部，都可以成为 Skill。

- 保留现有脚本逻辑，不强制重写
- 仅补充结构化头部用于索引与导出
- 支持混合仓库（Python/Shell/JS/Rust/Lua/Markdown 等）
- 通过一技能一目录提升跨运行时迁移能力

### Markdown-only 与 nanoskills 对比

| 维度 | 仅 Markdown 的 Skill 系统 | nanoskills |
| --- | --- | --- |
| Skill 载体 | 主要是 `.md` | Markdown + 带 YAML 头部的脚本/文件 |
| 既有脚本复用 | 常需要重写 | 保留原逻辑，仅补头部 |
| 检索路径 | 常依赖人工浏览/grep | 索引化本地检索（`sync` -> `search`） |
| Agent 集成 | 文本提取方式不稳定 | `search --json` 稳定输出 |
| 配置可见性 | 生效配置不直观 | `nanoskills config` 直接查看三段快照 |
| 跨运行时迁移 | 依赖各自格式与目录 | 一技能一目录 + 标准元数据 |

## 能力说明

`nanoskills` 提供一条确定性流程：

1. 扫描配置路径
2. 解析脚本/Markdown 中的 skill 头部
3. 构建本地索引缓存
4. 模糊检索 skill
5. 导出机器可读 JSON 工具定义

关键特点：

- 本地检索速度快，适合高频 skill find
- JSON 输出稳定，适合 Agent/Runtime 深度集成
- scan -> index -> search 流程稳定可复现

项目聚焦索引与检索，不负责远程执行或工作流编排。

## 安装

Release 安装：
- https://github.com/gtiders/nanoskills/releases/latest

源码安装：

```bash
cargo install --path .
```

## 快速开始

```bash
nanoskills init
nanoskills config
nanoskills sync --strict
nanoskills search image --json
```

## 核心命令

- `nanoskills init`:
  创建全局配置 `~/.config/nanoskills/.agent-skills.yaml`。
- `nanoskills init --local`:
  创建项目本地配置 `./.agent-skills.yaml`。
- `nanoskills config`:
  打印三段配置快照：
  - 默认配置
  - 当前目录本地配置
  - 最终生效配置（合并后）
- `nanoskills sync`:
  扫描并重建本地索引。
- `nanoskills sync --strict`:
  严格模式，遇到非法头部直接报错。
- `nanoskills search <query> [--limit N]`:
  模糊检索技能。
- `nanoskills search <query> --json`:
  导出可直接用于工具调用的 JSON。
- `nanoskills list [--json] [--detailed]`:
  列出已索引技能。
- `nanoskills pick`:
  交互式 TUI（仅人类交互，不适合自动化）。

## 配置模型

配置文件位置：

- 全局：`~/.config/nanoskills/.agent-skills.yaml`
- 本地：`./.agent-skills.yaml`

最终生效配置计算顺序：

1. 读取全局配置（若存在）
2. 读取本地配置（若存在）
3. 合并全局 + 本地：
   - 列表字段（`scan_paths`、`ignore_patterns`）：本地去重追加
   - 标量字段（`max_file_size`、`search_limit`、`language`）：本地覆盖全局
4. 若缺失则在 `scan_paths` 前置注入 `~/.config/nanoskills/skills`

使用 `nanoskills config` 查看真实生效结果。

## 最小配置示例

```yaml
scan_paths:
  - skills
  - ./automation
ignore_patterns:
  - target
  - .git
max_file_size: 1MB
search_limit: 10
language: en
```

## Skill 头部要求

最小字段：

- `name`
- `description`

推荐字段：

- `tool_name`（稳定工具标识）
- `tags`
- `args`

Python 示例：

```python
# ---
# name: disk_check
# description: Check disk usage
# tool_name: disk_check
# tags: [ops, monitoring]
# args:
#   path:
#     type: string
#     description: target path
#     required: false
# ---
print("ok")
```

## 集成模式

与 AI Runtime 集成时，优先使用：

```bash
nanoskills search <intent> --json
```

建议策略：

1. 先执行 `nanoskills search <intent> --json`
2. 从返回 JSON 中选择最匹配工具
3. 若无匹配，再回退默认工具/推理流程

## 内置 Skills

- [nanoskills_usage_guide](./skills/nanoskills_usager/SKILL.md)
- [nanoskills_builder](./skills/nanoskills_builder/SKILL.md)

## 开发

```bash
cargo fmt
cargo clippy --all-targets --all-features -D warnings
cargo test
```

## License

MIT
