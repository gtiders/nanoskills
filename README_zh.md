# nanoskills

> **为 AI Agent 打造的极速本地技能库 CLI。**  
> **零配置**扫盘，**秒级**建索引，**原生输出**可直接喂给大模型的工具 JSON。

[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![English README](https://img.shields.io/badge/README-English-blue.svg)](./README.md)

![Demo](assets/demo.gif)

## nanoskills 是什么？

`nanoskills` 是一个用 Rust 编写的本地技能库索引与检索工具。

它做三件事，而且都做得很快：

- 扫描你的本地脚本、提示词、自动化工具
- 把它们整理成可搜索的技能索引
- 以 **Agent 可消费** 的 JSON 输出给 OpenAI / Claude 等模型

一句话：

**把你散落在本地磁盘里的能力，整理成 AI Agent 能直接调用的技能库。**

## 核心特性

### ⚡ Nanospeed
**1.5 秒扫描 150,000+ 文件。**

这是 `nanoskills` 最硬的底层能力。

- 基于 `ignore` 的高速并发扫描
- 默认尊重 `.gitignore` / ignore 规则
- **几乎零配置**
- 内置大文件限制与二进制嗅探
- 本地暴力扫盘，但依然克制、稳定、可控

这是“暴力美学”，也是工程化。

### 🤖 Agent Ready
原生输出 **OpenAI / Claude 友好** 的工具 JSON。

- `search --json` 直接返回机器可读结构
- 自动生成稳定、合法、可去重的 tool ID
- 参数结构映射为 JSON Schema 风格
- 能直接接入 function calling / tool calling

你不需要再手写一层胶水。

### 🎨 Immersive TUI
一个真正好用的终端界面。

- 实时模糊搜索
- Master-Detail 浏览体验
- 语法高亮预览
- 键盘优先
- 沉浸、顺手、没有多余噪音

## 演示

![Demo](assets/demo.gif)

## 快速开始

### 1. 安装

```bash
cargo install --path .
```

### 2. 初始化配置

```bash
nanoskills init
```

默认会创建全局配置：

```text
~/.config/nanoskills/.agent-skills.yaml
```

如果你想在当前目录创建项目级配置：

```bash
nanoskills init --local
```

### 3. 构建本地索引

```bash
nanoskills sync
```

### 4. 搜索技能

```bash
nanoskills search image
```

### 5. 导出 Agent JSON

```bash
nanoskills search image --json
```

### 6. 打开 TUI 浏览

```bash
nanoskills pick
```

## 核心命令

### 构建索引

```bash
nanoskills sync
nanoskills sync --strict
```

示例输出：

```text
⚡ 索引构建完成，耗时 1487 ms。共扫描 150243 个文件，索引 312 个技能。
```

### 命令行搜索

```bash
nanoskills search resize
nanoskills search resize --limit 10
```

### 导出 JSON 给 Agent

```bash
nanoskills search resize --json
```

示例输出：

```json
[
  {
    "type": "function",
    "function": {
      "name": "image_resize_1a2b3c4d",
      "description": "Resize an image to a target width and height.",
      "parameters": {
        "type": "object",
        "properties": {
          "input": {
            "type": "string",
            "description": "Input image path"
          },
          "width": {
            "type": "integer",
            "description": "Target width"
          }
        },
        "required": ["input", "width"]
      }
    }
  }
]
```

### 使用 TUI 浏览和选择

```bash
nanoskills pick
```

## 配置作用域

`nanoskills` 同时支持**全局配置**和**项目本地配置**。

### 全局配置

```bash
nanoskills init
```

会创建：

```text
~/.config/nanoskills/.agent-skills.yaml
~/.config/nanoskills/skills/
```

默认的全局配置会扫描 `~/.config/nanoskills/skills` 这个共享技能目录。

### 本地配置

```bash
nanoskills init --local
```

会创建：

```text
./.agent-skills.yaml
```

默认的本地配置会扫描当前目录（`.`）。

### 读取顺序

`nanoskills` 在项目目录中运行时，会按这个顺序读取配置：

1. 先读取全局配置，作为基础层
2. 再读取当前目录的本地配置，作为覆盖层

也就是说，你可以把通用技能放在全局目录里，同时按项目覆盖 `scan_paths`、限制项和 ignore 规则。

## Agent 接入指南

你可以把 `nanoskills` 当成一个**本地工具注册表**。

### Python

```python
import json
import subprocess

tools_json = subprocess.check_output(
    ["nanoskills", "search", "image", "--json"],
    text=True,
)
tools = json.loads(tools_json)

# 把 tools 直接传给 OpenAI / Claude 的工具调用接口
print(tools)
```

### Bash

```bash
TOOLS="$(nanoskills search image --json)"
echo "$TOOLS"
# 再把这段 JSON 喂给你的 Agent Runtime
```

## 工作方式

`nanoskills` 会：

1. 扫描本地目录
2. 从脚本头部提取 YAML 元数据
3. 构建本地缓存索引
4. 通过三种方式暴露能力

- 适合人类查看的 CLI
- 沉浸式 TUI
- 适合 Agent 消费的 JSON

## 适用场景

- 个人 AI 自动化工具箱
- 本地 Prompt / Script 注册表
- 团队内部技能库
- Coding Agent 的工具发现层
- 本地优先的工具调用目录

## 技术栈

- Rust
- `ignore`
- `rayon`
- `ratatui`
- `crossterm`
- `syntect`
- `serde_json`
- `serde_yaml`

## 项目哲学

- **本地优先**
- **快到无感**
- **零仪式感**
- **Agent 原生**
- **终端体验也值得认真做**

## License

MIT
