# sks

基于注册表的本地脚本启动器。`sks` 读取 YAML 注册项，提供脚本列表、检索、执行和 MCP 接口。

## 核心功能

- 使用 Python 风格 ASCII 名称精确执行脚本：`[A-Za-z_][A-Za-z0-9_]*`。
- 支持 YAML 注册表、配置导入、描述和标签。
- 交互式 Picker，支持源码预览和语法高亮。
- 通过 MCP 提供脚本搜索、元数据和源码只读资源。
- 执行前将脚本复制到当前目录的 `.sks/<文件名>`。
- 从 GitHub 最新 Release 自更新，按编译目标选择资源并校验文件摘要。
- 提供脚本使用和脚本创建的内置 Skill 指引。

## 环境依赖

- 从源码构建需要 Rust 1.85 或更高版本。
- 每个注册脚本所需的运行时，例如 Python，由脚本自行决定。
- 仅使用 MCP 集成时需要 MCP 客户端。

## 安装部署

从源码构建并安装：

```bash
git clone https://github.com/gtiders/skillscripts.git
cd skillscripts
cargo install --path .
```

初始化全局配置：

```bash
sks init
```

配置目录为 `~/.config/sks`。`init` 会创建 `sks.yaml`、空的 `scripts.yaml`，并将 `sks-script-use` 和 `sks-script-create` Agent Skill 安装到 `~/.agents/skills`。

## 使用方法

```bash
sks list
sks pick
sks run <name> [args...]
sks skill use
sks skill create
sks update
sks update --check
sks update --force
```

`run` 按 `name` 精确匹配。名称后的所有参数都会追加到注册命令。执行前，脚本源文件会复制到当前目录的 `.sks/<文件名>`；同名文件直接覆盖。即使命令执行失败，复制也已经完成。

`list` 以 YAML 输出完整注册表。`pick` 提供脚本名称、描述和源码预览。

`update` 请求 GitHub 最新 Release，按二进制编译时的 Rust target（包括 GNU 或 musl）选择资源，校验 `checksums.txt` 后替换当前可执行文件。`--check` 只检查不安装；`--force` 在版本比较无法确认时仍执行安装。

### MCP

让 MCP 客户端通过 stdio 启动服务：

```json
{
  "command": "sks",
  "args": ["mcp"]
}
```

服务提供 `search_scripts` 和以下只读资源：

```text
sks://registry
sks://scripts/<name>
sks://scripts/<name>/source
```

搜索结果包含 `sks run <name> [args...]`。`mcp.search_limit` 设置默认结果数量，范围为 1–10。

## 配置说明

全局配置文件：`~/.config/sks/sks.yaml`

```yaml
mcp:
  search_limit: 5

imports:
  - scripts.yaml
  - imports/tools.yaml

scripts: []
```

脚本注册示例：

```yaml
scripts:
  - name: ase_to_xyz
    path: tools/ase2xyz.py
    command: python {{path}}
    comment: 将 ASE 可读取的结构文件转换为 extended XYZ
    tags: [ase, structure, extxyz, conversion]
```

规则：

- `name` 必填、大小写敏感，且全局唯一。
- 名称必须匹配 `[A-Za-z_][A-Za-z0-9_]*`。空字符串、Unicode 字符、数字开头、点号、连字符、斜杠和空格均非法。
- `path` 必须是相对 Unix 风格路径，相对于定义它的 YAML 文件解析。
- 只有全局配置可以声明 `imports`；被导入文件不能继续导入，也不能声明 `mcp`。
- `command` 必须包含 `{{path}}`，运行时替换为解析后的脚本路径。
- `comment` 和 `tags` 可选。标签用于搜索排序加权。

## 常见问题

### `Global config not found`

先运行 `sks init`，再向 `~/.config/sks/sks.yaml` 或导入的 YAML 文件添加注册项。

### `invalid script name`

将名称改为 ASCII Python 风格标识符，例如 `convert_csv` 或 `_internal`。

### `unknown script name`

运行 `sks list` 查看已注册名称，并使用完全一致的名称。匹配区分大小写，不进行模糊猜测。

### `command` 校验失败

在命令中加入 `{{path}}`，例如 `python {{path}}`。

### `sks update` 找不到资源

Release 必须提供与当前二进制编译目标对应的压缩包以及匹配的 `checksums.txt`。检查网络连接和 GitHub 最新 Release 的资源列表。
