# 产品定位与架构设计报告：Agent 轻量级本地技能库 (CLI)

> 创建时间：2026-03-19
> 用途：项目 README 草案 / 架构设计文档（RFC）

---

## 一、背景与痛点分析 (Problem Statement)

在当前的 AI Agent 生态中，处理本地任务存在两个极端的痛点：

1. **现有 Skill/Tool 框架"杀鸡用牛刀"**：如 GPT Actions、Claude Tools 等系统主要面向复杂的、重型的 API 交互（如鉴权、调用外部 SaaS 服务）。对于本地高频、微小但具体的任务（如"按正则重命名文件"、"转换图片格式"），要求开发者编写复杂的 OpenAPI Schema 或去云端配置是极其反人性的。

2. **"脚本坟墓"与 AI 的"重复造轮子"**：人类开发者经常编写一次性脚本解决临时问题，但往往用完即弃，缺乏整理；另一方面，当要求 Agent 处理简单任务时，Agent 往往需要经历"构思代码 -> 编写 -> 缺依赖报错 -> 调试"的漫长且易翻车的链路，浪费时间和 Token。

---

## 二、核心理念 (Core Philosophy)

本工具的核心哲学是 **"人机同构 (Human-Machine Isomorphism)"** 与 **"零摩擦封装 (Zero-Friction Wrapping)"**：

- **Script-as-a-Skill**：一段普通的 Python/Shell 脚本，只要在头部加入标准的元数据（Header Metadata）注释块，它就瞬间变成了一个 Agent 可以精准调用的"技能"。
- **双向适用**：对人类，它依然是可以通过终端直接运行的普通脚本；对 Agent，它是带有明确参数说明和执行模板的标准 API。

---

## 三、场景定位 (Positioning & Triggers)

系统支持两种核心触发模式，赋予 Agent "长期工具记忆"和"自我进化"能力：

### 1. 主动调用 (The On-Demand Assistant)

- **场景**：解决用户的即时需求，跨越"寻找并执行脚本"的摩擦力。
- **流程**：用户下发指令 -> Agent 通过 CLI 检索本地技能库 -> 获取匹配脚本的执行参数 -> Agent 直接通过系统终端拼装并执行脚本 -> 返回结果。
- **价值**：将 Agent 转化为本地私人工具箱的智能调度员。

### 2. 被动反思与沉淀 (The Auto-Archiver & Combiner) ✨

- **场景**：解决开发者"不想整理脚本"的痛点，实现 Agent 能力的有机生长。
- **流程 (沉淀)**：Agent 在帮用户写完一段解决临时问题的有效代码后，触发"被动反思"，自动在后台将代码加上标准 Header，通过 CLI 存入本地库并建立索引。
- **流程 (复用)**：面对复杂新任务，Agent 首选策略从"从零写代码"变为"检索本地库寻找可用模块组装"。
- **价值**：打造 Agent 的"外挂海马体"，让它越用越聪明，越用越懂你的本地环境。

---

## 四、核心数据结构：标准 Header 规范

利用特定格式的注释块（如 YAML Frontmatter）实现对脚本的无损封装：

```yaml
#!/usr/bin/env python3
"""
---
name: img_format_converter
description: 极速转换图片格式。支持 webp, png, jpg 等常见格式互转。
tags: [image, format, convert]
command_template: python {filepath} --input {in} --format {fmt}
args:
  in:
    type: string
    description: 原始图片文件的绝对路径
  fmt:
    type: string
    description: 目标格式，例如 png, jpg
---
"""
# 正常的脚本逻辑...
```

---

## 五、CLI 工具明细功能 (Features Blueprint)

CLI 专注于管理、索引和呈现，不负责实际的执行生命周期（执行交由 Agent 调取系统 Shell 完成）。

| 功能 | 说明 |
|------|------|
| `init` | 在当前目录或全局生成配置文件（.agent-skills.yaml），定义脚本扫描路径和匹配正则。 |
| `sync` / `index` | 递归遍历目录，提取所有符合正则的文件的 Header 信息，构建极速检索的本地索引（如 JSON 或 SQLite 格式）。 |
| `search` | Agent 的核心发现工具。支持按 tags、name 或全文 description 检索，强制输出 Agent 友好的纯 JSON 列表。 |
| `info <script_name>` | 精准输出特定脚本的绝对路径、command_template 和参数定义（args），指导 Agent 如何执行。 |
| `create` / `scaffold` | 一键生成带有空 Header 骨架的脚本文件，方便 Agent 或人类快速录入新技能。 |

---

## 六、Agent 侧配套 Skill 设计 (Agent Integration)

为了让 Agent 能使用这个 CLI，需要为其配置两个基础的系统级 Skill：

### 1. CLI 领航员 (Navigator Tool)

- **封装命令**：`search` & `info`
- **作用**：Agent 的"本地百度"。在执行操作类任务前，强制 Agent 先调用此工具查找是否有现成的脚本可用，获取执行规范。

### 2. 技能构造师 (Constructor Tool)

- **封装命令**：`create` & `sync`
- **作用**：Agent 的"档案管理员"。允许 Agent 将验证成功的临时代码片段规范化，写入本地文件系统并更新索引，完成技能的自我繁衍。

---

## 下一步行动建议 (Next Steps)

这份报告已经为你搭建了非常坚实的骨架。如果要真正开始编码实现，我们需要优先敲定协议的细节。

你希望我们接下来：
- [ ] 详细定义 YAML Header 中支持的所有字段类型（比如需不需要加入 `dependencies` 字段来处理依赖问题？）
- [ ] 先设计 CLI 的命令行接口交互规范（比如具体的输入输出 JSON 格式）

---

## 七、技术选型：为什么开源推广必须选 Rust？

> 更新时间：2026-03-19

### 7.1 核心理由

从简历装饰和推广的角度来看，Rust 具有 Python 无法比拟的优势：

| 优势维度 | Python 版 | Rust 版 |
|----------|-----------|---------|
| **分发方式** | 需要检查 Python 版本、创建虚拟环境、pip install | 单文件二进制，下载解压即用 |
| **启动性能** | 数百毫秒 | 毫秒级 |
| **内存占用** | 数十 MB | 几 MB |
| **简历吸引力** | "使用 Python 写了一个 CLI" | "使用 Rust 写了一个高性能、零依赖、跨平台的 Agent 基础设施 CLI" |
| **推广门槛** | 用户在 pip install 步骤流失 | 极致低门槛，Star 增长的生死线 |

### 7.2 核心卖点

- **零依赖分发**：Rust 编译成单文件二进制。用户下载，解压，丢进 `$PATH`，一秒钟开始体验。
- **性能即营销**：正如 `rg` (ripgrep)，极致性能让用户自发推荐。
- **对 Agent 友好**：极低启动耗时和极小内存占用，是面向 Agent 这一特定受众的核心卖点。

---

## 八、开源包装路线图 (Star Getter Roadmap)

### 阶段 1：Rust 极速复刻 (技术核心)

利用 Rust 强大的生态复刻现有功能，重点在于高性能和规范化：

- **CLI 解析**：使用 `clap` crate，自动生成专业的 `--help` 界面
- **文件遍历**：使用 `ignore` crate (ripgrep 底层实现)，快速递归遍历目录并自动尊重 `.gitignore`
- **Header 解析**：使用 `serde` + `serde_yaml` 解析注释块中的 YAML
- **跨平台编译**：配置 GitHub Actions，每次 Push 或 Release 时自动编译不同平台的二进制文件

### 阶段 2：产品化包装 (营销核心) ⭐

这是获得 Star 的关键：

- **起一个好名字**：朗朗上口且具有 Agent 辨识度。例如：`AgentCache`, `SkillForge`, `Loki`
- **写一个惊艳的 README.md**：
  - 一句话 Slogan：直击痛点，如 "The ultra-fast, zero-config Skill Manager for local AI Agents."
  - 演示 GIF/SVG：使用 `vhs` 录制 CLI 极速搜索、Agent 调用的过程
  - 详尽的"为什么"：清晰阐述为什么现在的 Tool 系统太重
- **提供 Agent 集成示例**：LangChain, AutoGen 或纯 OpenAI 接口调用 CLI 的代码示例
- **撰写配套的 Skill Prompt**：把两个配套 Skill（Construct & Navigate）写成优雅的 Prompt Schema

### 阶段 3：推广与社交 (渠道核心)

- **ProductHunt**：产品成熟时提交，争取当日 Top
- **HackerNews (Show HN)**：硬核开发者关注的最佳场所
- **Reddit & X (Twitter)**：在 r/LocalLLM, r/ChatGPTCoding, #AIAgent 等板块发帖
- **简历关键词**：Rust, Single Binary, Cross-platform, Performance Optimization, AIAgent Infrastructure, Middleware, File Parsing

---

## 九、技术债务与优化建议

- [ ] Python PoC 版本已实现核心逻辑，可作为参考
- [ ] 建议使用 Rust 重写 2.0 版本
- [ ] 优先实现 `clap` + `ignore` 核心骨架
- [ ] 配置 GitHub Actions 自动发布

---

## 十、跨语言元数据解析策略 (Frontmatter Parsing)

> 更新时间：2026-03-19

### 10.1 解决 Shebang (#!) 的影响

解析策略：**允许头部信息浮动（Top-N 扫描）**

- 解析器在读取文件时，不需要强求第一行就是头部
- 设定规则：只扫描文件的前 50 行（或前 2KB）
- **跳过 Shebang**：如果第一行以 `#!` 开头，解析器直接忽略这一行
- **寻找定界符**：从第二行开始往下找，寻找头部"定界符"（Delimiter）

### 10.2 解决不同语言注释差异 (语言无关解析法)

| 语言 | 注释符 |
|------|--------|
| Python/Shell/Ruby | `#` |
| JS/TS/C++/Go/Rust | `//` 或 `/* ... */` |
| HTML/Markdown | `<!-- -->` |

**核心思路：YAML Frontmatter + 智能前缀剥离 (Prefix Stripping)**

算法逻辑示例：
1. 解析器逐行扫描，找到第一行包含 `---` 的字符串
2. 提取前缀：记录下 `---` 前面的所有字符（比如 `#` 或 `//` 或 `*`）
3. 内容提取：继续往下读，把每一行开头相同的"前缀"自动切掉（Strip），直到遇到闭合的 `---`
4. 送入 YAML 解析：把切掉前缀后纯净的文本，交给 Rust 的 `serde_yaml` 进行反序列化

### 10.3 各种语言写法演示

**Python / Shell (使用 `#`)**

```python
#!/usr/bin/env python3
# ---
# name: format_json
# description: 格式化 JSON 字符串
# ---
import json
```

**JavaScript / Go (使用 `//`)**

```javascript
// ---
// name: format_json
// description: 格式化 JSON 字符串
// ---
const fs = require('fs');
```

**C++ / Rust (使用块注释 `/*`)**

```rust
/*
---
name: format_json
description: 格式化 JSON 字符串
---
*/
#include <iostream>
```

---

## 十一、头部字段规范设计 (Header Spec)

解决了怎么读的问题，接下来规定读什么。字段分为"必填（Required）"和"选填（Optional）"。

### 11.1 必填字段 (Agent 强依赖)

```yaml
---
name: image_resizer    # 技能唯一标识符（字母、数字、下划线）
description: 批量调整图片大小。  # 一句话描述，Agent 靠这个决定是否使用该技能。
---
```

### 11.2 选填字段

```yaml
---
tags: [image, utils]   # 用于搜索分类
version: 1.0.0         # 版本管理

# 执行模板。如果没有这一项，CLI 默认直接把参数按顺序追加在脚本后面。
# 有了这一项，Agent 就知道怎么精确拼装系统命令。
command_template: python {filepath} --width {w} --height {h} {input_dir}

# 参数定义（基于 JSON Schema 的简化版，对 LLM 非常友好）
args:
  w:
    type: integer
    description: 目标宽度像素值
    required: true
  h:
    type: integer
    description: 目标高度像素值
    required: false
  input_dir:
    type: string
    description: 需要处理的图片文件夹路径
    required: true
---
```

### 11.3 设计优势总结

- **极度宽容**：开发者不用去记特定的解析语法，只要在日常的注释里敲一对 `---` 塞点 YAML 进去就行
- **不破坏原生执行**：无论是在 IDE 里高亮，还是系统直接执行，都不会报错
- **Agent 解析精准**：提取出来的 YAML 天生适合转换成 JSON Schema，这是目前所有大模型调用 Tool 时最熟悉的格式

---

## 下一步行动建议 (续)

- [ ] 构思 Rust CLI 的具体命令字和输出 JSON 格式（比如 `search` 命令具体返回什么）
- [ ] 聊聊配套给 Agent 的那两个 Prompt 应该怎么写

---

## 十二、Rust 编译器对 Shebang 的特殊处理

> 更新时间：2026-03-19

### 12.1 为什么 Rust 的 Shebang 不会报错？

你非常敏锐！在 Rust 里，标准的注释是 `//`，而 `#` 是用来写宏或者属性的（比如 `#[derive(Debug)]`）。按理说，写个 `#!/usr/bin/env rust-script` 编译器应该直接罢工。

**答案是：Rust 编译器（rustc）在底层词法分析器（Lexer）里，专门为 Unix 文化开了一个"物理后门"。**

这并不是 Rust 语法的一部分，而是编译器的一个硬编码特例：

当 rustc 读取文件时，它会检查第一行是不是以 `#!` 开头：
- 如果是 `#![`（后面跟着个左方括号），它会认为这是 Rust 的全局属性（Inner Attribute），正常解析
- 如果是 `#!` 后面不是 `[`（比如跟的是 `/usr/bin...`），编译器会**强行把这一整行当成空白丢弃掉**，完全不进行语法检查！

> **彩蛋**：不仅是 Rust，Node.js (V8 引擎)、Go 语言的编译器，都在底层偷偷写了这个特例。大家为了兼容 Unix 系统的这个伟大发明，都选择了"妥协"。

---

## 十三、动态前缀推导算法 (Dynamic Prefix Inference)

### 13.1 为什么不写成千上万的正则规则？

如果按"写一堆规则（Python 用 `#`，JS 用 `//`，Lua 用 `--`，Lisp 用 `;`），然后每个文件挨个去撞"：
- **性能极差**：每次都要跑一堆正则
- **难以维护**：万一哪天出来个新语言用 `@@` 当注释怎么办？

### 13.2 核心思路：基于约定的动态特征提取

我们根本不关心这个文件是什么语言，也不需要内置任何语言的注释字典。CLI 就像一个**瞎子摸象**，只认 `---` 这个锚点。

### 13.3 算法解析

假设我们读取到了这样一段文本：

```lua
-- 这是我用 Lua 写的脚本
-- ---
-- name: hello_lua
-- description: 打印你好
-- ---
print("Hello")
```

CLI 的解析大脑是这样运作的（**无需任何预设规则**）：

1. **寻找锚点**：CLI 逐行扫描，忽略第一行的 Shebang。它在寻找包含 `---` 的第一行。

2. **动态提取前缀**（高光时刻！）：它找到了第二行 `-- ---`。此时，CLI 直接截取 `---` 前面的所有字符。
   - **提取结果**：前缀 = `--`（两个减号加一个空格）。

3. **闭眼剥离**：既然知道了前缀是 `--`，CLI 根本不管这是什么语言。它往下读第三行 `-- name: hello_lua`，直接用 Rust 原生的字符串切片功能，把开头的 `--` 咔嚓掉，剩下纯净的 `name: hello_lua`。

4. **遇到闭合**：直到它再次遇到包含 `---` 且前缀相同的行，停止读取。

### 13.4 为什么这个设计极其高明？

| 特性 | 说明 |
|------|------|
| **O(1) 复杂度** | 没有遍历规则，没有正则回溯。找到 `---` 的瞬间，规则就自动生成了。 |
| **永远向前兼容** | 哪怕明天有人发明了一门用 🚀 当注释符的语言，只要他写成 `🚀 ---`，我们的解析器依然能完美解析，一行代码都不用改。 |
| **完美应对块注释** | 如果是在 C++ 的 `/* ... */` 块注释里，`---` 前面可能什么都没有。那么推导出的前缀就是"空（Empty）"。接下来的行直接原样提取，逻辑依然完美闭环！ |

### 13.5 性能优势

有了这样优雅且性能爆炸的解析逻辑，你的 CLI 可以在**几百毫秒内扫完 Agent 本地挂载的几万个脚本库**，并精准提取出所有的"技能说明书"。

---

## 十四、多模态输出设计 (Multi-modal Output)

> 更新时间：2026-03-19

核心逻辑就是**"看人下菜碟"**。我们只需要在 CLI 里加一个全局开关（比如 `--json`），就能完美伺候好人类和机器这两位大爷：

### 14.1 给人看：ASCII 表格 (默认模式)

当你在终端直接敲 `agent-cli search 图片` 时，直接调用 Rust 的 `comfy-table` 库，打印一个对齐的表格：

```
+----------------+--------------------------+-----------+-----------------------------------+
| Name           | Description              | Tags      | Path                              |
+----------------+--------------------------+-----------+-----------------------------------+
| img_to_png     | 将任意图片转换为 PNG 格式| img, tool | /Users/xxx/scripts/img_to_png.py |
| resize_image   | 批量修改图片分辨率       | img       | /Users/xxx/scripts/resize.sh      |
+----------------+--------------------------+-----------+-----------------------------------+
```

人类一眼就能找到自己要的脚本路径。

### 14.2 给 Agent 看：JSON (--json 模式)

当 Agent 在后台调用 `agent-cli search 图片 --json` 时，终端直接吐出一段紧凑的 JSON 数组。

输出的 JSON 结构**直接贴合主流大模型（OpenAI / Claude / Gemini）的 Tool/Function Calling Schema 标准**，Agent 拿到数据后，连"转换和理解"的脑力都省了，直接当成自己的原生工具用：

```json
[
  {
    "name": "img_to_png",
    "description": "将任意图片转换为 PNG 格式",
    "path": "/Users/xxx/scripts/img_to_png.py",
    "command_template": "python {filepath} --in {input_path}",
    "parameters": {
      "type": "object",
      "properties": {
        "input_path": {
          "type": "string",
          "description": "需要转换的源图片绝对路径"
        }
      },
      "required": ["input_path"]
    }
  }
]
```

### 14.3 输出格式对比

| 模式 | 开关 | 用途 | 格式 |
|------|------|------|------|
| 人类模式 | (默认) | 终端直接查看 | ASCII 表格 |
| Agent 模式 | `--json` | 程序化调用 | JSON (Tool Schema 兼容) |

---

## 十五、极速索引机制设计 (Index Strategy)

> 更新日期：2026-03-19

### 15.1 索引的必要性

虽然 Rust 扫文件很快，但如果 Agent 每次思考都要去全盘正则扫描几千个文件，依然会有无谓的磁盘 I/O 损耗。为了追求极致的性能，需要一个极轻量级的本地缓存/索引机制。

### 15.2 索引文件设计

**方案 A：JSON 索引文件 (`.skills-index.json`)**

```json
{
  "version": "1.0.0",
  "last_sync": "2026-03-19T23:50:00Z",
  "skills": [
    {
      "name": "img_to_png",
      "description": "将任意图片转换为 PNG 格式",
      "path": "/Users/xxx/scripts/img_to_png.py",
      "tags": ["img", "tool"],
      "command_template": "python {filepath} --in {input_path}",
      "args": {
        "input_path": {
          "type": "string",
          "description": "需要转换的源图片绝对路径",
          "required": true
        }
      },
      "checksum": "a1b2c3d4..."
    }
  ]
}
```

**方案 B：SQLite 数据库**

- 适合技能库非常大的场景（>10,000 个脚本）
- 支持更复杂的查询（按日期、标签组合搜索）
- 索引文件更小（JSON 有冗余）

### 15.3 索引更新策略

| 策略 | 触发条件 | 优点 | 缺点 |
|------|----------|------|------|
| 手动 sync | 用户主动执行 `agent-cli sync` | 完全可控 | 需要主动触发 |
| 定时任务 | CRON 或定时检查文件 mtime | 保持新鲜 | 有延迟 |
| 混合模式 | 首次调用自动触发 + 增量更新 | 智能高效 | 实现复杂 |

### 15.4 增量更新思路

- 记录每个文件的 `mtime`（修改时间）和 `checksum`
- `sync` 时只解析有变化的文件
- 百万级文件也能做到秒级更新

---

## 十六、CLI 命令与输出格式总结

### 16.1 核心命令

```bash
# 初始化配置文件
agent-cli init

# 同步/构建索引
agent-cli sync

# 搜索技能（人类模式）
agent-cli search <关键词>

# 搜索技能（Agent 模式）
agent-cli search <关键词> --json

# 查看技能详情
agent-cli info <skill_name>

# 创建新技能
agent-cli create <skill_name>
```

### 16.2 输出格式确认

- [x] 人类模式：ASCII 表格
- [x] Agent 模式：JSON (Tool Schema 兼容)
- [ ] 索引机制：JSON 文件 或 SQLite（待定）

---

## 十七、缓存引擎：真正的极速之源

> 更新时间：2026-03-19

通过缓存，Agent 和你都不需要每次都去读几千个文件的硬盘 I/O，而是直接在内存里查字典。

### 17.1 agent-cli init (冷启动初始化)

- 用户指定一个或多个脚本目录
- CLI 扫一遍目录，提取所有合法的 Header
- 在用户的主目录（比如 `~/.config/agent-cli/index.json` 或 `.db`）生成一个全量缓存文件

### 17.2 agent-cli sync (触发式更新)

- 当用户新增了脚本，或者 Agent 自己写完脚本并 save 之后，主动调用一下 sync
- CLI 只比对文件的修改时间（mtime），进行增量更新，把新的 Header 刷入缓存
- 速度在毫秒级

---

## 十八、机器的接口：非交互式的 search

对于 Agent 来说，它只需要一个像数据库查询一样的 API。

- **命令**：`agent-cli search "处理图片" --json`
- **底层动作**：直接读取 index.json 缓存，进行关键字或正则匹配
- **输出**：直接把匹配到的列表以标准 JSON 吐出来
- **优势**：Agent 拿到后瞬间解析，没有任何交互阻碍。稳定且极速

---

## 十九、人类的浪漫：类似 fzf 的交互式 TUI

这是工具针对人类用户体验的杀手锏。Rust 社区有着极度繁荣的 TUI（终端用户界面）生态。

### 19.1 技术选型

- **skim**：纯 Rust 实现的 fzf 克隆版
- **inquire** / **dialoguer**：Rust 交互式提示库

### 19.2 交互场景

当你忘记了一个脚本的具体名字，在终端只敲下：

```bash
agent-cli  # 默认唤起交互模式，或者叫 agent-cli ui
```

屏幕上立刻弹出一个类似 fzf 的列表：

```
> img_ <-- 你在这里实时输入拼音或关键字
 2/150 matches

> img_to_png   | 将任意图片转换为 PNG 格式    | ~/.scripts/img.py
 resize_img    | 批量修改图片分辨率           | ~/.scripts/resize.sh
```

### 19.3 操作流程

- 敲几个字母（支持模糊匹配，比如打 `itp` 就能搜到 `img_to_png`）
- 上下箭头选择
- 按 **Enter**：直接唤起执行模板，让你填参数运行
- 按 **?** 或 **Tab**：在右侧展示这个脚本的完整描述和参数要求（预览模式）

---

## 二十、架构最终蓝图 (The Final Blueprint)

> 更新日期：2026-03-19

到这里，工具不仅在理念上完胜了现在笨重的 Agent Tool 框架，而且在工程实现上形成了一个极度优雅的闭环：

| 模块 | 核心功能 | 适用对象 | 核心技术点 (Rust) |
|------|----------|----------|-------------------|
| **解析器引擎** | 智能提取 `---` 包裹的 YAML 头部，动态剥离注释前缀 | 内部底层 | serde_yaml, 字符串切片 |
| **缓存与索引引擎** | init / sync 扫描全盘，生成本地高速索引 | 内部底层 | ignore (并发扫盘), JSON 序列化 |
| **API 模式 (机器)** | search --json 和 info，0 交互，纯结构化输出 | AI Agent | 只读高速响应，JSON Schema 输出 |
| **TUI 模式 (人类)** | agent-cli 唤起模糊匹配列表，回车运行 | 开发者 / 极客 | skim 或 inquire 实现终端交互 |
| **建造者模式** | create (生成骨架) / Agent 后台自动录入新代码并 sync | 人与机器皆可 | 文件 I/O 写入 |

---

## 二十一、设计阶段总结

### 21.1 已确认的模块

- [x] **立意**：解决"脚本坟墓"和 Agent 本地调用痛点
- [x] **核心机制**：约定优于配置的 Header（YAML Frontmatter）
- [x] **工程架构**：Rust、缓存、双模交互
- [x] **解析策略**：动态前缀推导算法（O(1) 复杂度）
- [x] **输出格式**：人类 ASCII 表格 + Agent JSON (Tool Schema)
- [x] **索引机制**：init / sync 增量更新
- [x] **交互模式**：非交互式 API + 交互式 TUI (fzf 风格)

### 21.2 下一步

- [ ] 编码实现阶段：直接开始建 Rust 项目 (`cargo new`)
- [ ] 或先探讨其他顾虑或想法

---

*文档状态：✅ 设计阶段已完成 | 等待编码实现*