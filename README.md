# nanoskills

> **Zero-config local skill indexing for AI Agents.**  
> Build a blazing-fast local skill library, search it in milliseconds, and feed it straight into your Agent runtime.

[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![中文文档](https://img.shields.io/badge/README-%E4%B8%AD%E6%96%87-blue.svg)](./README_zh.md)

![Demo](assets/demo.gif)

## Why nanoskills?

`nanoskills` is a Rust CLI for turning your local scripts, prompts, and automation snippets into an **Agent-ready skill library**.

It is built for one job:

- index local skills **fast**
- search them **interactively**
- export them as **tool-call JSON** for LLMs

## Features

### ⚡ Nanospeed
Scan **150,000+ files in ~1.5 seconds**.

- Built on top of `ignore`
- Multi-threaded filesystem walking
- Zero-config by default
- Binary-file guardrails with size gate + NUL sniffing
- Designed for local-first brute-force indexing

### 🤖 Agent Ready
Export skills as **OpenAI / Claude-friendly tool metadata**.

- `search --json` returns machine-readable JSON
- Stable, legal, deduplicated tool identifiers
- Parameters mapped into JSON Schema-style structures
- Easy to plug into tool/function calling pipelines

### 🎨 Immersive TUI
A terminal UI that actually feels good to use.

- Real-time fuzzy search
- Master-detail browsing
- Syntax-highlighted previews
- Keyboard-first flow
- Built with `ratatui` + `syntect`

## Demo

![Demo](assets/demo.gif)

## Quick Start

### 1. Install

```bash
cargo install --path .
```

### 2. Create a config file

```bash
nanoskills init
```

By default, this creates the global config at:

```text
~/.config/nanoskills/.agent-skills.yaml
```

To create a project-local config in the current directory instead:

```bash
nanoskills init --local
```

### 3. Build the local index

```bash
nanoskills sync
```

### 4. Search skills

```bash
nanoskills search image
```

### 5. Export Agent-ready JSON

```bash
nanoskills search image --json
```

### 6. Browse in the TUI

```bash
nanoskills pick
```

## Usage

### Build the index

```bash
nanoskills sync
nanoskills sync --strict
```

Example output:

```text
⚡ Index built in 1487 ms. Scanned 150243 files, indexed 312 skills.
```

### Search from the CLI

```bash
nanoskills search resize
nanoskills search resize --limit 10
```

### Export JSON for Agents

```bash
nanoskills search resize --json
```

Example output:

```json
[
  {
    "type": "function",
    "function": {
      "name": "image_resize_1a2b3c4d",
      "description": "Resize an image to a target width and height.",
      "parameters": {
        "type": "object",
        "properties": {
          "input": {
            "type": "string",
            "description": "Input image path"
          },
          "width": {
            "type": "integer",
            "description": "Target width"
          }
        },
        "required": ["input", "width"]
      }
    }
  }
]
```

### Browse with the TUI

```bash
nanoskills pick
```

## Configuration Scopes

`nanoskills` supports both **global** and **project-local** configuration.

### Global config

```bash
nanoskills init
```

This creates:

```text
~/.config/nanoskills/.agent-skills.yaml
~/.config/nanoskills/skills/
```

The default global config scans the shared `skills/` directory under `~/.config/nanoskills`.

### Local config

```bash
nanoskills init --local
```

This creates:

```text
./.agent-skills.yaml
```

The default local config scans the current directory (`.`).

### Resolution order

When `nanoskills` runs inside a project directory, it reads configuration in this order:

1. global config as the base layer
2. local config in the current directory as the override layer

That means you can keep shared skills globally while still customizing `scan_paths`, limits, or ignore rules per project.

## AI Integration

Use `nanoskills` as a local tool registry for your LLM stack.

### Python

```python
import json
import subprocess

tools_json = subprocess.check_output(
    ["nanoskills", "search", "image", "--json"],
    text=True,
)
tools = json.loads(tools_json)

# pass `tools` into your OpenAI / Claude tool-calling request
print(tools)
```

### Bash

```bash
TOOLS="$(nanoskills search image --json)"
echo "$TOOLS"
# pass this JSON into your Agent runtime
```

## How It Works

`nanoskills` scans local paths, extracts structured YAML headers from scripts, builds a cache index, and exposes the result through:

- human-friendly CLI output
- an interactive TUI
- machine-readable JSON for Agents

## Use Cases

- Personal AI automation toolbox
- Local prompt / script registry
- Team-internal skill catalogs
- Tool discovery for coding Agents
- Local-first alternatives to hosted registries

## Tech Stack

- Rust
- `ignore`
- `rayon`
- `ratatui`
- `crossterm`
- `syntect`
- `serde_json`
- `serde_yaml`

## Philosophy

- **Local-first**
- **Fast enough to feel instant**
- **No ceremony**
- **Agent-native output**
- **Terminal UX matters**

## License

MIT
