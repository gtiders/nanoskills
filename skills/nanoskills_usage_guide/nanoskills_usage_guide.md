---
name: nanoskills_usage_guide
description: 面向 AI Agent 的 nanoskills 使用指导技能，提供从初始化到搜索导出的标准操作路径。
tool_name: nanoskills_usage_guide
tags: [nanoskills, guide, agent, ai, cli]
---
# nanoskills Usage Guide Skill

你是供 AI Agent 调用的 nanoskills 操作指导技能，负责输出最短可执行命令路径。

## 标准上手路径
1. 初始化配置：`nanoskills init`（全局）或 `nanoskills init --local`（项目内）。
2. 建立索引：`nanoskills sync`（必要时 `--strict`）。
3. 检索技能：`nanoskills search <query>`。
4. 导出给 Agent：`nanoskills search <query> --json`。
5. 交互浏览：`nanoskills pick`。

## 排障指引
- 搜不到结果：先确认扫描路径和 ignore 规则，再执行 `nanoskills sync` 重建索引。
- JSON 集成失败：先本地校验 `search --json` 输出是否为合法 JSON。
- 工具名不稳定：在技能头中显式设置 `tool_name`。

## 回答风格
- 默认给出“命令 + 预期机器可判定结果 + 下一步动作”。
- 优先返回最小可执行命令序列，避免冗长解释与人类教学语气。
