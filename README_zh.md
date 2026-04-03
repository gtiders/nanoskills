# nanoskills

本地优先的 AI Agent 技能库索引与检索 CLI。

## 核心功能

`nanoskills` 解决 Agent 导向技能系统的两个核心问题。

### 1) Agent 工作流中的极速技能检索

从配置路径构建本地可搜索索引，保证检索结果稳定可复现：

- 扫描全局/本地合并配置下的技能文件
- 解析标准化元数据（`name`、`description`、`tags`）
- 构建本地索引缓存，实现低延迟查询
- 返回稳定的 JSON，用于工具调用集成

### 2) 不止于 Markdown 的技能系统

技能不限于 `.md` 文件。任何脚本或文件只要包含符合文件注释风格的 YAML 头，都可以成为技能。

- 保持原有脚本逻辑不变
- 仅需添加结构化头部即可完成索引和工具导出
- 支持混合仓库（Python/Shell/JS/Rust/Lua/Markdown 等）
- 一技能一目录，保持跨运行时可移植性

### 核心设计

- **自动索引** — 首次运行或缓存过期时自动构建/刷新索引（基于 TTL 或配置变更）。`sync` 仍可用于手动重建。
- **JSON 优先输出** — `search` 和 `list` 始终输出机器可读的 JSON。
- **模糊搜索** — 对 name、description、tags 进行快速内存评分。路径永不参与搜索。
- **交互式选择器** — 基于 skim 的 TUI 界面，带语法高亮预览。

## 安装

从 release 安装：
- https://github.com/gtiders/nanoskills/releases/latest

从源码安装：

```bash
cargo install --path .
```

## 快速上手

```bash
nanoskills init
nanoskills config
nanoskills search image    # 首次运行自动构建索引
```

## 核心命令

| 命令 | 说明 |
|---|---|
| `init` | 在 `~/.config/nanoskills/.agent-skills.yaml` 创建全局配置。`--local` 创建项目级配置。 |
| `config` | 打印默认配置、本地配置和最终合并配置。 |
| `sync` | 扫描并重建本地索引缓存。`--strict` 会在遇到格式错误的头部时报错退出。 |
| `search <query>` | 对索引技能进行模糊搜索。始终输出 JSON：`[{name, tags, description, path}, …]` |
| `list` | 列出所有已索引技能，输出 JSON。`nanoskills list --json` 输出紧凑格式。 |
| `pick` | 交互式 TUI 选择器，带预览。 |

### Search 和 List

两个命令均只输出 JSON：

```json
[
  {
    "name": "image_resize",
    "tags": ["image", "python"],
    "description": "使用 PIL 调整图片尺寸",
    "path": "./skills/image_resize"
  }
]
```

`search` 对 `name`、`description`、`tags` 进行模糊匹配。`path` 永不参与搜索。

### 索引生命周期

索引由程序自动管理：

- **首次运行** — 在首次查询前自动构建并缓存索引
- **缓存过期** — 若距上次同步已超过 `cache_ttl_seconds`，查询前静默重建索引
- **配置变更** — 若 `scan_paths`、`ignore_patterns` 或 `max_file_size` 与上次同步时不同，查询前静默重建索引
- **手动同步** — `nanoskills sync` 始终触发重建并打印进度

```
$ nanoskills search image
[cache stale, refreshing…]
[
  { "name": "image_resize", ... }
]
```

## 配置

配置文件：

- **全局**：`~/.config/nanoskills/.agent-skills.yaml`
- **本地**：`./.agent-skills.yaml`（项目级，与全局配置合并）

使用 `nanoskills config` 查看实际生效的运行时配置。

### 最简配置

```yaml
scan_paths:
  - skills
  - ./automation
ignore_patterns:
  - target
  - .git
max_file_size: 1MB
search_limit: 10
cache_ttl_seconds: 1h
```

### 缓存 TTL

支持时长格式：`30s`、`5m`、`2h`、`1d` 或纯秒数。默认为 `1h`。设为 `0` 可禁用基于 TTL 的刷新（仅在配置变更或显式 `sync` 时重建）。

## 技能头部规范

技能是任何包含 YAML 块（使用文件注释语法）的文件。

**必填字段：**

- `name`
- `description`

**推荐字段：**

- `tags`（字符串标签列表）
- `args`（参数定义）

### Python 示例

```python
# ---
# name: disk_check
# description: 检查磁盘使用情况
# tags: [ops, monitoring]
# args:
#   path:
#     type: string
#     description: 目标路径
#     required: false
# ---
print("ok")
```

### Shell 示例

```bash
#!/bin/bash
# ---
# name: git_log
# description: 显示最近提交
# tags: [git, vcs]
# ---
git log --oneline -10
```

## 项目结构

```
src/
  model/        # 领域类型：Skill, Index, Config, JsonView
  io/           # 文件系统操作：scanner, parser, index_store, config_loader
  services/     # 业务逻辑：engine, index_service, search, sync
  cli/          # 表现层：commands, picker (skim), output
```

## 依赖

| Crate | 用途 |
|---|---|
| `anyhow` | 错误处理 |
| `clap` | CLI 参数解析 |
| `comfy-table` | `list` 的人类可读表格输出 |
| `dirs` | 平台相关的配置目录 |
| `dunce` | 路径规范化 |
| `fuzzy-matcher` | name + description + tags 内存模糊评分 |
| `ignore` | 快速 glob/gitignore 模式匹配 |
| `rayon` | 并行文件扫描 |
| `serde` / `serde_yaml` / `serde_json` | 序列化 |
| `skim` | 交互式 TUI 选择器 |
| `syntect` | 选择器预览的语法高亮 |
| `path-clean` | 路径清理 |

## 开发

```bash
cargo fmt
cargo clippy --all-targets --all-features -D warnings
cargo test
```

## License

MIT
