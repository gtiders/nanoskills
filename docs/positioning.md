# Positioning

## What nanoskills is

`nanoskills` is a **local skill indexing and retrieval CLI** for agent workflows.

It is inspired by the Claude Skill idea, but focuses on a practical gap: teams already have many useful scripts and snippets, yet these capabilities are hard to discover, reuse, and feed into agent tool-calling.

Core workflow:

1. scan local files (`sync`)
2. parse YAML skill headers
3. build local index cache
4. query via CLI/TUI or export tools JSON (`search --json`)

## What nanoskills is not

- not a workflow orchestrator
- not a remote execution platform
- not a replacement for your runtime framework

It standardizes discovery and tool metadata so both humans and agents can reuse the same skill source.

## Core value

- For humans: quickly find previously written scripts/skills instead of rewriting.
- For agents: consume normalized tool metadata from `search --json`.
- For teams: share a stable `skills/` directory layout across repos and runtimes.
