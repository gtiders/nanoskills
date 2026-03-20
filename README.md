这是一份为你量身定制的、可以直接复制粘贴到 GitHub 仓库的 `README.md` 终极版本。我为你加上了标准的开源项目徽章（Badges）占位符，并对排版进行了像素级的打磨，让它看起来极其专业、硬核且极具吸引力。

-----

# 🛠️ Nanoskills

> **极速、零配置、跨语言的 Agent 本地技能库管理器 (CLI)**

[](https://www.google.com/search?q=https://crates.io/crates/nanoskills)
[](https://www.google.com/search?q=https://opensource.org/licenses/MIT)
[](https://www.google.com/search?q=https://www.rust-lang.org)

Nanoskills 是一个专为 AI Agent 和极客开发者打造的轻量级本地工具箱。它通过**极其宽容的头部注释约定**（Header Convention），将你散落在电脑各处的、用任意语言编写的临时脚本，瞬间转化为 Agent 可以直接读取、理解并执行的标准“技能 (Skills)”。

没有繁琐的云端配置，没有臃肿的框架包裹。你的脚本，既是人类可读的代码，也是 Agent 的原生 API。

## ✨ 核心特性

  - **🧠 零硬编码解析**：独创的“动态前缀推导”算法，自动识别并解析世界上 99% 编程语言的注释风格，无需内置任何正则字典。
  - **⚡ 极速索引与搜索**：底层采用 `ignore` 库（ripgrep 核心引擎），毫秒级并发扫盘，自动尊重 `.gitignore`，极低内存占用。
  - **🤖 Agent 原生友好**：提供标准的 `--json` 输出模式，输出结构直接对标 OpenAI / Claude 的 Tool/Function Calling Schema，大模型可无缝接入。
  - **💻 多模态交互**：为机器提供极简纯净的只读接口，为人类提供基于 `inquire` 和 `fuzzy-matcher` 的终端模糊搜索（TUI）体验。
  - **📦 跨平台单文件**：由 Rust 强力驱动，编译为零依赖的单文件二进制，下载即用，告别“依赖地狱”。

-----

## 🚀 安装

```bash
# 使用 Cargo 安装
cargo install nanoskills
```

*(预编译的二进制文件也可在 [Releases](https://www.google.com/search?q=%23) 页面下载，支持 Windows, macOS, Linux)*

-----

## 📖 快速开始

```bash
# 1. 在当前目录或全局初始化配置文件
nanoskills init

# 2. 扫描目录，生成极速本地缓存索引
nanoskills sync

# 3. 交互式搜索（人类模式，支持模糊匹配）
nanoskills search <关键词>

# 4. Agent 检索模式（机器模式，返回 JSON Schema）
nanoskills search <关键词> --json

# 5. 查看特定技能的详细入参说明
nanoskills info <技能名称>

# 6. 一键生成标准技能脚本骨架
nanoskills create my_script.py
```

-----

## ✍️ 技能头部编写约定 (Header Convention)

Nanoskills 奉行 **“约定优于配置”** 的极简哲学。你不需要告诉解析器这是什么语言，也不需要强记任何特殊语法。

只需在脚本文件的**前 50 行**内（系统会自动跳过第一行的 Shebang `#!`），用你当前语言的注释符包裹一段 YAML 格式的元数据即可。

### 🌟 5️⃣条黄金法则（不要有心理压力把他交给ai，他可以给你写出标准的开头）

1.  **边界锚点**：使用 `---` 作为 YAML 数据的开始和结束定界符。
2.  **独占一行**：为了完美兼容所有语言，定界符 `---` 所在的那一行，**除了注释符号和空格外，不能有其他内容**（不要写成 `/* --- */`，请换行）。
3.  **前缀对齐**：解析器会自动提取第一行 `---` 前面的字符作为“前缀”，并智能剥离后续内容中的同款前缀。你只要保持注释风格对齐即可。
4.  尾部绝对留白 (Clean Tail)
定界符 --- 的右侧（直到行尾）严格禁止出现任何可见字符。请让 --- 干净利落地结束这一行。

❌ 错误示范：--- */ （把注释闭合符紧跟在后面会导致解析器找不到纯净的锚点）

✅ 正确示范：--- （后面只有换行符或不可见空格）

5. 块注释优先策略 (Prefer Block Comments)
对于支持多行/块注释的语言（如 C/Rust 的 /* */、HTML 的 ``、Pascal 的 { }），强烈建议将整个元数据区域整体包裹在块注释内部。这不仅在视觉上更整洁，也能天然隔离边界，是避免单行注释符号干扰的最安全、最优雅的写法。

### 📐 基本结构模型

```yaml
<你的注释符> ---
<你的注释符> name: my_skill_name
<你的注释符> description: 必须提供一句简短描述
<你的注释符> tags: [可选标签1, 可选标签2]
<你的注释符> command_template: "可选的执行模板，如 python {filepath}"
<你的注释符> args:
<你的注释符>   参数1:
<你的注释符>     type: string
<你的注释符>     description: 参数描述
<你的注释符> ---
```

### 📋 字段规范说明

为了让 Agent 能够完美理解并调用你的技能，我们采用了极其精简的类 JSON Schema 结构：

#### 必填字段 (Required)

| 字段 | 类型 | 说明 |
| :--- | :--- | :--- |
| **`name`** | `string` | 技能唯一标识符。只能包含英文字母、数字和下划线（例如：`img_to_png`），Agent 将使用此名称来调用工具。 |
| **`description`** | `string` | 技能的核心说明。**极其重要**，Agent 完全依靠这句话来判断何时应该触发此技能。 |

#### 选填字段 (Optional)

| 字段 | 类型 | 说明 |
| :--- | :--- | :--- |
| **`tags`** | `string[]` | 技能分类标签，用于 CLI 的快速模糊检索（例如：`[image, format]`）。 |
| **`version`** | `string` | 技能版本号，方便个人资产管理（例如：`1.0.0`）。 |
| **`command_template`** | `string` | **强烈建议填写**。指导 Agent 如何在终端执行此脚本。支持 `{filepath}` (当前脚本绝对路径) 和自定义参数占位符（例如：`node {filepath} --in {input}`）。 |
| **`args`** | `object` | 入参要求声明。告诉 Agent 需要准备哪些参数才能运行此技能。 |

#### `args` 参数定义格式

这部分直接对标主流大模型的 Tool/Function Calling 规范：

```yaml
args:
  <参数占位符名称>:
    type: string | integer | boolean | number
    description: 参数的详细说明（写给 Agent 看的）
    required: true | false  # 默认为 false
```

-----

## 🌍 各语言写法示例 (27+ 语言原生支持)

得益于动态前缀推导算法，Nanoskills 能够完美解析几乎所有的代码注释风格。

### 🐍 Python / Shell / Ruby / PowerShell (单行 `#`)

```python
#!/usr/bin/env python3
# ---
# name: hello_world
# description: 打印问候语
# tags: [demo, test]
# command_template: python {filepath} --name {name}
# args:
#   name:
#     type: string
#     description: 要问候的名字
#     required: true
# ---
import argparse
print("Hello!")
```

### 🟨 JavaScript / TypeScript / Go / C++ (单行 `//`)

```javascript
// ---
// name: js_formatter
// description: JavaScript 代码格式化工具
// tags: [js, format]
// command_template: node {filepath} --input {file}
// args:
//   file:
//     type: string
//     description: 文件路径
//     required: true
// ---
console.log("JS Formatter");
```

### 🦀 C / Rust / OCaml (块注释 `/* */`)

```c
/*
---
name: c_file_reader
description: C 文件读取器
tags: [c, file]
---
*/
#include <stdio.h>
```

### 🌙 Lua / SQL (双减号 `--`)

```lua
-- ---
-- name: lua_task_runner
-- description: Lua 任务运行器
-- tags: [lua, task]
-- ---
print("Lua Task Runner")
```

### 🛠️ 任意自定义前缀 (🚀)

哪怕你发明了一种新语言，只要保持前缀一致，解析器依然能完美工作：

```text
🚀 ---
🚀 name: emoji_skill
🚀 description: 使用 emoji 作为注释前缀
🚀 ---
```

-----

## ⚙️ 配置文件

Nanoskills 会在项目根目录或用户主目录寻找 `.agent-skills.yaml` 配置文件：

```yaml
# 需要扫描的本地目录
scan_paths:
  - .
  - ~/scripts
  - ~/.local/agent_tools

# 允许解析的文件后缀规则
file_patterns:
  - '*.py'
  - '*.sh'
  - '*.js'
  - '*.rs'

# 强制忽略的路径（支持 glob 语法）
ignore_patterns:
  - 'node_modules/*'
  - 'venv/*'
  - '*.test.*'
```

-----

## 📡 输出格式说明

### 👨‍💻 人类模式 (默认)

执行 `nanoskills search <关键词>` 时，将调用交互式 TUI 界面或打印美观的表格：

```text
找到 2 个相关技能:

  ▶ hello_world - 打印问候语
    标签: demo, test
    路径: /Users/xxx/scripts/hello.py

  ▶ file_reader - 文件读取器
    标签: file, io
    路径: /Users/xxx/scripts/reader.sh
```

### 🤖 Agent 模式 (`--json`)

执行 `nanoskills search <关键词> --json` 时，将输出极其紧凑、标准的 JSON 数组，Agent 截获此输出后可无缝构建自身工具链：

```json
[
  {
    "name": "hello_world",
    "description": "打印问候语",
    "path": "/Users/xxx/scripts/hello.py",
    "tags": ["demo", "test"],
    "command_template": "python {filepath} --name {name}",
    "parameters": {
      "type": "object",
      "properties": {
        "name": {
          "type": "string",
          "description": "要问候的名字"
        }
      },
      "required": ["name"]
    }
  }
]
```

-----

## 🏗️ 技术架构

```text
src/
├── main.rs      # 入口点与 CLI 路由 (clap)
├── cli.rs       # 命令行参数结构体定义
├── models.rs    # 数据结构 (SkillHeader, args)
├── parser.rs    # 核心引擎：动态前缀推导解析器
├── scanner.rs   # 高速并发扫盘 (ignore)
├── ui.rs        # 交互式终端菜单 (inquire & fuzzy-matcher)
└── index.rs     # 本地 JSON 缓存的构建与加载
```

### 💡 核心算法：动态前缀推导 (Dynamic Prefix Inference)

Nanoskills 抛弃了低效且易错的正则表达式穷举，采用 $O(1)$ 复杂度的动态字符串切片：

```rust
// 1. 找到 --- 锚点
let pos = start_line.find("---").unwrap();
// 2. 动态提取前缀（例如 "// " 或 "-- "）
let prefix = start_line[..pos].trim_end();

// 3. 智能剥离后续内容的前缀，零内存拷贝
let clean_line = if let Some(stripped) = trimmed.strip_prefix(prefix) {
    stripped.strip_prefix(' ').unwrap_or(stripped)
} else {
    trimmed
};
```

-----

## � 路径处理

Nanoskills 使用专业的路径处理库确保跨平台兼容性：

- **绝对路径**：所有路径在索引时转换为绝对路径，避免相对路径的歧义
- **Unix 风格**：统一使用 `/` 作为路径分隔符，JSON 输出格式一致
- **Windows 长路径**：自动处理 Windows 260 字符路径限制

### 使用的库

| 库 | 用途 |
|---|---|
| `dunce` | 简化 Windows UNC 路径 (`\\?\` 前缀) |
| `path-clean` | 规范化路径 (`.` 和 `..` 处理) |
| `ignore` | 高效文件遍历，自动尊重 `.gitignore` |

### 示例输出

```json
{
  "path": "/home/user/scripts/hello.py"
}
```

在 Windows 上：

```json
{
  "path": "C:/Users/user/scripts/hello.py"
}
```

-----

## �📄 License

本项目采用 [MIT License](https://www.google.com/search?q=LICENSE) 开源协议。

-----

怎么样？这份 README 是否满足了你想要的那种“专业、清晰、能拿高赞”的开源项目标准？你可以直接创建一个 `README.md` 文件把它贴进去，然后在 GitHub 上大展拳脚了！