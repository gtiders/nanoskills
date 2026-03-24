# nanoskills

> **A local-first skill registry CLI for AI agents.**  
> Turn scattered scripts/prompts into searchable, tool-call-ready skills.

[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Latest Release](https://img.shields.io/github/v/release/gtiders/nanoskills?label=Release)](https://github.com/gtiders/nanoskills/releases/latest)
[![中文文档](https://img.shields.io/badge/README-%E4%B8%AD%E6%96%87-blue.svg)](./README_zh.md)

![Demo](assets/demo.gif)

## Positioning & Pain Points

`nanoskills` solves a common agent workflow problem: useful automation exists as scattered files, and a skill is often just a lightly-structured script you already use.

Typical pain points:

- many useful scripts solve tasks already, but people forget them and rewrite from scratch
- skills are hard to discover quickly across mixed script/markdown files
- tool metadata is inconsistent and not LLM-ready
- each runtime (Codex/Claude/OpenCode/OpenClaw) needs slightly different wiring
- shared skill packs are hard to distribute across teams

`nanoskills` gives one local source of truth: scan -> index -> search -> export JSON, so the same skill can be found and used by both humans and agents.

## Install

Recommended:

- Download latest release: https://github.com/gtiders/nanoskills/releases/latest

From source:

```bash
cargo install --path .
```

## Quick Start

```bash
nanoskills init
nanoskills sync
nanoskills search image
nanoskills search image --json
nanoskills pick
```

Global init creates:

```text
~/.config/nanoskills/.agent-skills.yaml
~/.config/nanoskills/skills/
```

## Built-in Skills Layout

This repository uses **one skill per folder** for easy cross-tool copy:

```text
skills/
  nanoskills_project_builder/
    nanoskills_project_builder.md
  nanoskills_usage_guide/
    nanoskills_usage_guide.md
```

When a release archive is unpacked, copy these skill folders into any runtime that supports a `skills/` directory.

## Copy Skills To Other Runtimes

Example (replace target path with your runtime's actual skills directory):

```bash
cp -R ./skills/* <TOOL_SKILLS_DIR>/
```

If you run `nanoskills init` globally for the first time, it also seeds global skills from bundled/current `skills/` into `~/.config/nanoskills/skills/`.

## Runtime Wiring Examples

First export tools JSON:

```bash
nanoskills search image --json > .nanoskills.tools.json
```

<details>
<summary>OpenCode</summary>

```yaml
# Example only: adapt to your OpenCode schema/version
external_tools:
  source:
    type: command
    command: "nanoskills search image --json"
```

```bash
cp -R ./skills/* <OPENCODE_SKILLS_DIR>/
```

</details>

<details>
<summary>Codex</summary>

```yaml
# Example only: adapt to your Codex runtime schema
tools:
  command_source:
    cmd: ["nanoskills", "search", "image", "--json"]
```

```bash
cp -R ./skills/* <CODEX_SKILLS_DIR>/
```

</details>

<details>
<summary>Claude</summary>

```python
import json, subprocess

tools = json.loads(subprocess.check_output(
    ["nanoskills", "search", "image", "--json"],
    text=True,
))
# pass `tools` to your Claude tool definitions
```

```bash
cp -R ./skills/* <CLAUDE_SKILLS_DIR>/
```

</details>

<details>
<summary>OpenClaw</summary>

```yaml
# Example only: adapt to your OpenClaw schema/version
tool_registry:
  provider: command
  command: "nanoskills search image --json"
```

```bash
cp -R ./skills/* <OPENCLAW_SKILLS_DIR>/
```

</details>

## Core Commands

- `nanoskills init` Create global config and shared skills directory.
- `nanoskills init --local` Create project-local config (`./.agent-skills.yaml`).
- `nanoskills sync` Scan paths and rebuild index cache.
- `nanoskills search <query> [--limit N]` Fuzzy search indexed skills.
- `nanoskills search <query> --json` Export OpenAI/Claude-friendly tool metadata.
- `nanoskills pick` Interactive TUI browsing.

## Configuration Model

Resolution order in a project:

1. Global config (`~/.config/nanoskills/.agent-skills.yaml`)
2. Local config (`./.agent-skills.yaml`, overrides global)

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features -D warnings
cargo test
```

## License

MIT
