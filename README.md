# skillscripts

A local-first, fast skill-retrieval CLI for AI agent integration.

## What It Does

`skillscripts` (alias `sks`) solves two core problems for agent-oriented skill systems.

### 1) Fast Skill Retrieval for Agent Workflows

It builds a local searchable index from configured paths and keeps retrieval deterministic:

- scan skill files from global/local merged config
- parse normalized metadata (`name`, `description`, `tags`)
- build a local index cache for low-latency lookup
- return stable JSON for tool-call integration

### 2) Skill System Beyond Markdown

A skill is not limited to `.md`. Any script/file can be a skill as long as it contains a valid YAML header in the file's comment style.

- keep existing script logic unchanged
- add a structured header for indexing and tool export
- support mixed repositories (Python/Shell/JS/Rust/Lua/Markdown, etc.)
- keep one-skill-per-folder layout for portability across runtimes

### Key Design

- **auto-indexing** — the index is built and refreshed automatically on first run or when stale (TTL-based or config changed). `sync` is still available for manual rebuilds.
- **JSON-first output** — `search` and `list` always emit machine-readable JSON.
- **fuzzy search** — fast in-memory scoring across name, description, and tags. Paths are never part of the search corpus.
- **interactive picker** — skim-based TUI for human browsing, with syntax-highlighted preview.

## Install

From release:
- https://github.com/gtiders/skillscripts/releases/latest

From source:

```bash
cargo install --path .
```

## Quick Start

```bash
skillscripts init
skillscripts config
skillscripts search image    # auto-builds index on first run
# or use the short alias
sks init
sks search image
```

## Core Commands

| Command | Description |
|---|---|
| `init` | Create global config at `~/.config/skillscripts/skillscripts.yaml`. Use `--local` for a project-level config. |
| `config` | Print default, local, and effective merged config. |
| `sync` | Scan and rebuild the local index cache. Use `--strict` to fail on malformed headers. |
| `search <query>` | Fuzzy search indexed skills. Always outputs JSON: `[{name, tags, description, path}, …]` |
| `list` | List all indexed skills as JSON. Use `skillscripts list --json` for compact output. |
| `pick` | Interactive TUI picker with preview. |

### Search and List

Both commands emit JSON only:

```json
[
  {
    "name": "image_resize",
    "tags": ["image", "python"],
    "description": "Resize an image using PIL",
    "path": "./skills/image_resize"
  }
]
```

`search` fuzzy-matches across `name`, `description`, and `tags`. `path` is never part of the search corpus.

### The Index Lifecycle

The index is managed automatically:

- **first run** — builds and caches the index automatically before the first query
- **cache stale** — if `cache_ttl_seconds` has elapsed since the last sync, the index is rebuilt silently before the query
- **config changed** — if `scan_paths`, `ignore_patterns`, or `max_file_size` differ from the last sync, the index is rebuilt silently
- **manual sync** — `skillscripts sync` always triggers a rebuild and prints progress

```
$ skillscripts search image
[cache stale, refreshing…]
[
  { "name": "image_resize", ... }
]
```

## Configuration

Config files:

- **global**: `~/.config/skillscripts/skillscripts.yaml`
- **local**: `./skillscripts.yaml` (project-level, merged with global)

Use `skillscripts config` to inspect the exact runtime result.

### Minimal Config

```yaml
scan_paths:
  - skills
  - ./automation
ignore_patterns:
  - target
  - .git
max_file_size: 1MB
search_limit: 10
cache_ttl_seconds: 1h
```

### Cache TTL

Accepts durations: `30s`, `5m`, `2h`, `1d`, or plain seconds. Default is `1h`. Set to `0` to disable TTL-based refresh (rebuild only on config change or explicit `sync`).

## Skill Header Requirements

A skill is any file containing a YAML block in its comment syntax.

**Minimum fields:**

- `name`
- `description`

**Recommended fields:**

- `tags` (list of string tags)
- `args` (parameter definitions)

### Python Example

```python
# ---
# name: disk_check
# description: Check disk usage
# tags: [ops, monitoring]
# args:
#   path:
#     type: string
#     description: target path
#     required: false
# ---
print("ok")
```

### Shell Example

```bash
#!/bin/bash
# ---
# name: git_log
# description: Show recent commits
# tags: [git, vcs]
# ---
git log --oneline -10
```

## Architecture

```
src/
  model/        # domain types: Skill, Index, Config, JsonView
  io/           # filesystem: scanner, parser, index_store, config_loader
  services/     # business logic: engine, index_service, search, sync
  cli/          # presentation: commands, picker (skim), output
```

## Dependencies

| Crate | Role |
|---|---|
| `anyhow` | Error handling |
| `clap` | CLI argument parsing |
| `comfy-table` | Human-readable table output for `list` |
| `dirs` | Platform-specific config directory |
| `dunce` | Path normalization |
| `fuzzy-matcher` | In-memory fuzzy scoring (name + description + tags) |
| `ignore` | Fast glob/gitignore pattern matching |
| `rayon` | Parallel file scanning |
| `serde` / `serde_yaml` / `serde_json` | Serialization |
| `skim` | Interactive TUI picker |
| `syntect` | Syntax highlighting in picker preview |
| `path-clean` | Path cleaning |

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features -D warnings
cargo test
```

## License

MIT
