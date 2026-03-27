---
name: nanoskills_usage_guide
description: 在需要管理或检索本地技能库时，直接使用 nanoskills CLI 完成配置检查、索引构建与 JSON 检索输出。
tool_name: nanoskills_usage_guide
tags: [nanoskills, cli, skill, search, sync]
---
# nanoskills Operational Skill

## 触发条件
- 用户提到：`nanoskills`、技能检索、技能索引、`.agent-skills.yaml`、技能 JSON 导出。
- 任务涉及：初始化配置、查看当前生效配置、重建索引、按关键词查找技能。

## 执行规则
1. 优先输出可直接执行的命令，不写教学说明。
2. 自动化流程默认使用非交互命令；不要使用 `nanoskills pick`。
3. 机器集成场景优先使用 `--json` 输出。
4. 配置相关问题一律先看 `nanoskills config` 的生效结果。
5. 若需要新增项目级 `scan_paths` 且本地配置不存在，可请求用户授权在当前项目创建 `./.agent-skills.yaml`（`nanoskills init --local`）。

## 标准命令集
- 初始化全局配置：`nanoskills init`
- 初始化当前目录配置：`nanoskills init --local`
- 查看配置快照（默认/当前目录/生效）：`nanoskills config`
- 构建索引：`nanoskills sync`
- 严格模式构建索引：`nanoskills sync --strict`
- 搜索技能：`nanoskills search <query>`
- 搜索技能（JSON）：`nanoskills search <query> --json`
- 列出已索引技能（JSON）：`nanoskills list --json`

## 决策顺序
1. 未初始化或配置不确定：执行 `nanoskills config`（必要时补 `nanoskills init` 或 `nanoskills init --local`）。
2. 搜索前先确保索引可用：执行 `nanoskills sync`。
3. 需要稳定检索给上游系统：执行 `nanoskills search <query> --json`。
4. 需要定位头部错误：执行 `nanoskills sync --strict`。

## 失败回退
- 报错“未找到索引”：先执行 `nanoskills sync`。
- 报错 YAML/解析失败：执行 `nanoskills sync --strict`，按报错文件逐个修复后重试。
- 搜索为空：检查 `nanoskills config` 中生效 `scan_paths`，确认后重建索引再搜。
- 需要扩展搜索路径但无本地配置：先请求用户授权创建项目配置，再写入新的 `scan_paths` 并执行 `nanoskills sync`。

## 输出格式
- 固定为三行：
  - `命令:`
  - `结果:`
  - `下一步:`
- 简短、可判定、可复制执行。
