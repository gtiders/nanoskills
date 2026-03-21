# 🛠️ Nanoskills

> **极速、零配置、跨语言的 Agent 本地技能库 CLI**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Nanoskills 是一个专为 AI Agent 和开发者打造的轻量级本地工具箱。通过**宽容的头部注释约定**，将任意语言的脚本瞬间转化为 Agent 可直接读取、理解并执行的标准"技能"。

## ✨ 核心特性

- **🧠 零硬编码解析**：动态前缀推导算法，自动识别 99% 编程语言的注释风格
- **⚡ 极速索引**：基于 `ignore` 库（ripgrep 核心），毫秒级并发扫描
- **🤖 Agent 原生友好**：`--json` 输出直接兼容 OpenAI Tools Schema
- **💻 多模态交互**：终端模糊搜索 TUI + 机器只读接口
- **📦 零依赖**：Rust 编译为单文件二进制，下载即用

## 🚀 安装

```bash
# 从源码编译
git clone https://github.com/gtiders/nanoskills.git
cd nanoskills
cargo build --release

# 二进制文件位于
./target/release/nanoskills
```

## 📖 快速开始

```bash
# 1. 初始化配置文件
nanoskills init

# 2. 扫描并索引技能
nanoskills sync

# 3. 列出所有技能
nanoskills list

# 4. 搜索技能（模糊匹配）
nanoskills search "图片"

# 5. Agent 模式（OpenAI Tools 格式）
nanoskills search "图片" --json

# 6. 交互式选择（TUI）
nanoskills pick
```

## ✍️ 技能头部编写约定

在脚本文件的**前 50 行**内，用注释符包裹 YAML 元数据：

### 基本结构

```yaml
<注释符> ---
<注释符> name: skill_name
<注释符> description: 技能描述
<注释符> tags: [tag1, tag2]
<注释符> command_template: "python {filepath} --input {file}"
<注释符> args:
<注释符>   file:
<注释符>     type: string
<注释符>     description: 输入文件路径
<注释符>     required: true
<注释符> ---
```

### 字段说明

| 字段 | 必填 | 说明 |
|------|------|------|
| `name` | ✅ | 技能唯一标识符 |
| `description` | ✅ | 技能描述（Agent 依赖此判断何时调用） |
| `tags` | ❌ | 分类标签，用于搜索 |
| `command_template` | ❌ | 执行模板，支持 `{filepath}` 占位符 |
| `args` | ❌ | 参数定义，兼容 JSON Schema |

## 🌍 多语言示例

### Python / Shell / Ruby (# 注释)

```python
#!/usr/bin/env python3
# ---
# name: image_converter
# description: 图片格式转换工具
# tags: [image, convert]
# command_template: python {filepath} --input {file} --format {format}
# args:
#   file:
#     type: string
#     description: 输入文件路径
#     required: true
#   format:
#     type: string
#     description: 目标格式 (png/jpg/webp)
#     required: true
# ---
```

### JavaScript / TypeScript / Go (// 注释)

```javascript
// ---
// name: js_formatter
// description: JavaScript 代码格式化
// tags: [js, format]
// command_template: node {filepath} --input {file}
// args:
//   file:
//     type: string
//     description: 文件路径
//     required: true
// ---
```

### C / Rust / OCaml (块注释)

```c
/*
---
name: c_compiler
description: C 代码编译器
tags: [c, compile]
---
*/
```

### Lua / SQL (-- 注释)

```lua
-- ---
-- name: lua_runner
-- description: Lua 脚本执行器
-- tags: [lua, script]
-- ---
```

## 📡 输出格式

### 人类模式（默认）

```bash
$ nanoskills search "image"
```

```
🔍 找到 2 个技能 (显示前 5 个):

┌───┬──────────────────┬──────────────────┬──────────────────┬─────────────────┐
│ # ┆ 📝 名称          ┆ 📖 描述          ┆ 🏷️ 标签          ┆ 📁 路径         │
╞═══╪══════════════════╪══════════════════╪══════════════════╪═════════════════╡
│ 1 ┆ image_converter  ┆ 图片格式转换工具 ┆ image, convert   ┆ /path/to/skill  │
│ 2 ┆ image_resizer    ┆ 图片尺寸调整     ┆ image, resize    ┆ /path/to/skill  │
└───┴──────────────────┴──────────────────┴──────────────────┴─────────────────┘
```

### Agent 模式 (--json)

```bash
$ nanoskills search "image" --json
```

```json
[
  {
    "type": "function",
    "function": {
      "name": "image_converter",
      "description": "图片格式转换工具",
      "parameters": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "description": "输入文件路径"
          },
          "format": {
            "type": "string",
            "description": "目标格式 (png/jpg/webp)"
          }
        },
        "required": ["file", "format"]
      }
    }
  }
]
```

**直接兼容 OpenAI Function Calling！**

## ⚙️ 配置文件

`.agent-skills.yaml`:

```yaml
# 扫描路径
scan_paths:
  - .
  - ~/scripts

# 忽略模式
ignore_patterns:
  - node_modules/*
  - venv/*

# 最大文件大小（支持 MB/KB/GB）
max_file_size: 1MB

# 搜索结果限制
search_limit: 5
```

## 🏗️ 命令详解

| 命令 | 说明 |
|------|------|
| `nanoskills init` | 初始化配置文件 |
| `nanoskills init --force` | 强制覆盖已存在的配置 |
| `nanoskills sync` | 扫描并索引技能 |
| `nanoskills sync --strict` | 严格模式，显示解析错误 |
| `nanoskills list` | 列出所有技能 |
| `nanoskills list --detailed` | 显示详细参数定义 |
| `nanoskills list --json` | JSON 格式输出 |
| `nanoskills search <query>` | 模糊搜索技能 |
| `nanoskills search <query> --json` | OpenAI Tools 格式输出 |
| `nanoskills search <query> --limit 10` | 限制输出数量 |
| `nanoskills pick` | 交互式 TUI 选择 |

## 🔧 技术架构

```
src/
├── main.rs      # 入口点
├── cli.rs       # CLI 命令定义
├── models.rs    # 数据结构
├── parser.rs    # 动态前缀推导解析器
├── scanner.rs   # 高速并发扫描
├── cmd_sync.rs  # 索引构建与搜索
├── config.rs    # 配置解析
└── ui.rs        # TUI 交互界面
```

### 核心算法：动态前缀推导

```rust
// 1. 定位 --- 锚点
let pos = start_line.find("---")?;

// 2. 动态提取前缀
let prefix = start_line[..pos].trim_end();

// 3. 智能剥离后续内容
let clean_line = trimmed.strip_prefix(prefix)
    .unwrap_or(trimmed);
```

## 📄 License

[MIT License](LICENSE)
