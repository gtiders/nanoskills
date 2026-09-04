# sks

一个极简的、基于注册表的脚本检索与执行 CLI。

## 概览

`sks` 只读取一个全局配置文件：`~/.config/sks/sks.yaml`。  
它不会扫描目录，也不会解析脚本头部元数据。

每个脚本注册项包含三个必填字段，并可添加描述与搜索标签：

- `name`
- `path`
- `command`
- `comment`（可选描述）
- `tags`（可选搜索词）

`path` 相对于“定义它的 YAML 文件”解析。只有全局配置允许声明 `imports`。

## 配置格式

全局配置：

```yaml
mcp:
  search_limit: 5

imports:
  - lang/python.yaml

scripts:
  - name: hello_world
    path: scripts/hello.py
    command: python {{path}}
    comment: 向用户问好
    tags: [hello, text]
```

被导入配置：

```yaml
scripts:
  - name: build_tool
    path: tools/build.py
    command: python {{path}}
```

规则：

- 只允许相对路径
- 所有平台的注册表路径都必须使用 Unix 风格 `/` 分隔符
- 配置始终位于 `~/.config/sks`，不会使用 AppData
- imported 文件不能再声明 `imports`
- `name` 必须符合 `[A-Za-z_][A-Za-z0-9_]*` 且全局唯一
- `command` 必须包含 `{{path}}`

## 命令

```bash
sks init
sks list
sks pick
sks run hello_world foo --bar baz
sks mcp
```

- `init` 创建 `~/.config/sks/sks.yaml`、空的 imported `scripts.yaml`，并向 `~/.agents/skills` 安装 `sks-script-discovery` 与 `sks-script-authoring` Agent Skills
- `list` 以 YAML 输出所有已注册脚本
- `pick` 打开交互式选择器，并显示表格化列表与语法高亮预览
- `run <name> [args...]` 替换 `command` 中的 `{{path}}`，并把剩余参数全部追加到命令尾部
- `mcp` 通过 stdio 启动本地 MCP server

## MCP

在 MCP 客户端中配置：

```json
{
  "command": "sks",
  "args": ["mcp"]
}
```

这个 MCP 是只读的。它只暴露一个由模型调用的 `search_scripts` 工具，以及注册表、脚本元数据和源码 resources。搜索复用 picker 的 skim 模糊匹配器，以自然语言 query 负责召回；可选 tags 只是排序加权信号，不再作为必需过滤条件。主配置中的 `mcp.search_limit` 可以把默认结果数量设置为 1–10，默认值是 5；单次工具调用仍可用 `limit` 临时覆盖。imported 配置不能声明 MCP 选项。每次请求都会重新读取注册表，因此修改 YAML 后无需重启 server，下一次搜索即可看到变化。

MCP 指令采用“先搜索、后新写”的触发策略：当任务可以通过脚本执行时，模型应在编写临时代码或 shell 命令前搜索一次，即使用户没有提到 sks、本地脚本或已有工具。计算、格式转换、文件与数据处理、内容生成、校验和构建流程都会触发；纯概念讨论则不触发。任何明确要求“使用脚本”的请求都必须先搜索，不因任务看起来简单或临时编写代码更快而跳过。工具同时声明为只读、幂等且不访问外部世界，以降低模型对试探性调用的风险判断。

找到脚本后，结果会提供 `sks run <name> [args...]` 和资源 URI；参数不清楚时，模型可以按需读取源码。没有匹配项是正常的成功结果，不会阻止模型换用其他方式继续工作。

`sks init` 安装两个互补的 Agent Skills：`sks-script-discovery` 让兼容代理在一次性编程前主动发现并复用已有脚本，`sks-script-authoring` 指导代理编写、注册、验证和测试新脚本。已有配置和 Skill 默认保留，只有使用 `--force` 时才覆盖。最终是否调用工具仍由 MCP 客户端和模型决定；服务器说明与 Skill 会显著增强触发倾向，但不能从协议层强制调用。

## Picker

`pick` 的结果列表显示脚本名称和描述：

- `NAME`
- `COMMENT`

右侧预览区会直接渲染完整脚本文件内容，并使用内嵌 `syntect` 做语法高亮。当前默认主题是 GitHub Dark，预览背景由 skim 控制。

## run 语义

`run` 的设计是刻意保持极简：

```bash
sks run example_script input.txt --mode fast
```

它的行为是：

1. 找到 `name: hello_world2`
2. 替换 `command` 中的 `{{path}}`
3. 把 `input.txt --mode fast` 原样追加到命令后面

也就是说，`run` 在 `<name>` 之后不再保留自己的选项解析层。

## 安装

从源码安装：

```bash
cargo install --path .
```
