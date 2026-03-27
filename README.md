# nanoskills

A local-first, fast skill-retrieval CLI for deep AI agent integration.

## Comprehensive Overview

`nanoskills` solves two core problems for agent-oriented skill systems.

### 1) Fast Skill Retrieval for Agent Workflows

It builds a local searchable index from configured paths and keeps retrieval deterministic:

- scan skill files from global/local merged config
- parse normalized metadata (`name`, `description`, `tool_name`, `args`, `tags`)
- build a local cache index for low-latency lookup
- return stable JSON for tool-call integration

This makes `skill find` fast and repeatable in real agent loops.

### 2) Skill System Beyond Markdown

A skill is not limited to `.md`. Any script/file can be a skill as long as it contains a valid YAML header in the file's comment style.

- keep existing script logic unchanged
- add structured header for indexing and tool export
- support mixed repositories (Python/Shell/JS/Rust/Lua/Markdown, etc.)
- keep one-skill-per-folder layout for portability across runtimes

### Markdown-only vs nanoskills

| Dimension | Markdown-only skill system | nanoskills |
| --- | --- | --- |
| Skill carrier | Mostly `.md` | Markdown + scripts/files with YAML header |
| Existing script reuse | Usually requires rewrite | Keep original script logic, add header only |
| Retrieval path | Often file browsing/manual grep | Indexed local search (`sync` -> `search`) |
| Agent integration | Ad-hoc text extraction | Stable `search --json` tool output |
| Config visibility | Implicit | Explicit via `nanoskills config` snapshots |
| Cross-runtime portability | Varies by format/layout | One-skill-per-folder + normalized metadata |

## What It Does

`nanoskills` provides one deterministic pipeline:

1. scan configured paths
2. parse skill headers from scripts/markdown
3. build a local index cache
4. retrieve by fuzzy search
5. export machine-readable JSON tool definitions

Key characteristics:

- fast local retrieval for frequent skill find workflows
- stable JSON output for deep agent/runtime integration
- deterministic scan -> index -> search flow

It focuses on indexing and retrieval. It is not a remote execution or workflow orchestration system.

## Install

From release:
- https://github.com/gtiders/nanoskills/releases/latest

From source:

```bash
cargo install --path .
```

## Quick Start

```bash
nanoskills init
nanoskills config
nanoskills sync --strict
nanoskills search image --json
```

## Core Commands

- `nanoskills init`:
  Create global config at `~/.config/nanoskills/.agent-skills.yaml`.
- `nanoskills init --local`:
  Create local config at `./.agent-skills.yaml`.
- `nanoskills config`:
  Print three snapshots:
  - default config
  - current-directory local config
  - effective merged config
- `nanoskills sync`:
  Scan + rebuild local index cache.
- `nanoskills sync --strict`:
  Fail on malformed/invalid headers.
- `nanoskills search <query> [--limit N]`:
  Fuzzy search indexed skills.
- `nanoskills search <query> --json`:
  Export tool-call-ready JSON.
- `nanoskills list [--json] [--detailed]`:
  List indexed skills.
- `nanoskills pick`:
  Interactive TUI (human only, not for automation).

## Configuration Model

Config files:

- global: `~/.config/nanoskills/.agent-skills.yaml`
- local: `./.agent-skills.yaml`

Effective config is computed as:

1. read global (if exists)
2. read local (if exists)
3. merge global + local
   - list fields (`scan_paths`, `ignore_patterns`): append local unique items
   - scalar fields (`max_file_size`, `search_limit`, `language`): local overrides global
4. inject global shared skills path `~/.config/nanoskills/skills` into `scan_paths` front (if missing)

Use `nanoskills config` to inspect the exact runtime result.

## Minimal Config Example

```yaml
scan_paths:
  - skills
  - ./automation
ignore_patterns:
  - target
  - .git
max_file_size: 1MB
search_limit: 10
language: en
```

## Skill Header Requirements

Minimum header fields:

- `name`
- `description`

Recommended fields:

- `tool_name` (stable tool id)
- `tags`
- `args`

Python example:

```python
# ---
# name: disk_check
# description: Check disk usage
# tool_name: disk_check
# tags: [ops, monitoring]
# args:
#   path:
#     type: string
#     description: target path
#     required: false
# ---
print("ok")
```

## Integration Pattern

For AI runtime integration, prefer JSON search output:

```bash
nanoskills search <intent> --json
```

Suggested policy:

1. try `nanoskills search <intent> --json`
2. choose best-matching tool from returned JSON
3. if no match, fall back to normal tool/reasoning flow

## Built-in Skills

- [nanoskills_usage_guide](./skills/nanoskills_usager/SKILL.md)
- [nanoskills_builder](./skills/nanoskills_builder/SKILL.md)

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features -D warnings
cargo test
```

## License

MIT
