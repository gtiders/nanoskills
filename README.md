# nanoskills

> **Zero-config local skill indexing for AI Agents.**  
> Scan fast, search instantly, export tool-ready JSON.

[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Latest Release](https://img.shields.io/github/v/release/gtiders/nanoskills?label=Release)](https://github.com/gtiders/nanoskills/releases/latest)
[![中文文档](https://img.shields.io/badge/README-%E4%B8%AD%E6%96%87-blue.svg)](./README_zh.md)

## Install

Recommended for most users:

- Download latest binary release: https://github.com/gtiders/nanoskills/releases/latest

Build from source:

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

## Core Commands

- `nanoskills init` Create global config and shared skills directory.
- `nanoskills init --local` Create project-local config (`./.agent-skills.yaml`).
- `nanoskills sync` Scan paths and rebuild local index cache.
- `nanoskills search <query> [--limit N]` Fuzzy search indexed skills.
- `nanoskills search <query> --json` Export OpenAI/Claude-style tool metadata.
- `nanoskills pick` Browse/search in interactive TUI.

## Agent Runtime Config Examples

First, export tools JSON:

```bash
nanoskills search image --json > .nanoskills.tools.json
```

<details>
<summary>OpenCode Example</summary>

```yaml
# Example only: field names may vary by OpenCode version
external_tools:
  source:
    type: command
    command: "nanoskills search image --json"
```

</details>

<details>
<summary>Codex Example</summary>

```yaml
# Example wiring pattern
tools:
  command_source:
    cmd: ["nanoskills", "search", "image", "--json"]
```

</details>

<details>
<summary>Claude Example</summary>

```python
import json, subprocess

tools = json.loads(subprocess.check_output(
    ["nanoskills", "search", "image", "--json"],
    text=True,
))
# pass `tools` into Claude tool definitions
```

</details>

<details>
<summary>OpenClaw Example</summary>

```yaml
# Example only: adapt to your OpenClaw runtime schema
tool_registry:
  provider: command
  command: "nanoskills search image --json"
```

</details>

## Configuration Model

Resolution order inside a project:

1. Global config (`~/.config/nanoskills/.agent-skills.yaml`)
2. Local config (`./.agent-skills.yaml`, overrides global)

This lets you keep shared global skills while customizing per-project scan paths and limits.

## How It Works

1. Parallel file scan with `ignore` rules.
2. Parse YAML skill headers from script/comment blocks.
3. Build cache index under `~/.cache/nanoskills/`.
4. Output through CLI table, TUI, or JSON.

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features -D warnings
cargo test
```

## FAQ

<details>
<summary>How do I create arrow-expandable sections in README?</summary>

Use HTML `details/summary`:

```markdown
<details>
<summary>Click to expand</summary>

Your hidden content here.

</details>
```

GitHub renders a disclosure arrow automatically.

</details>

## License

MIT
