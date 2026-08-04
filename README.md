# sks

Minimal registry-driven script launcher and picker CLI.

## Overview

`sks` reads a single global config file at `~/.config/sks/sks.yaml`.  
Scripts are registered explicitly; the tool does not scan directories or parse script headers.

Each registered script has three required fields plus an optional description and search tags:

- `id`
- `path`
- `command`
- `comment` (optional description)
- `tags` (optional search terms)

`path` is resolved relative to the YAML file that defines it. Only the global config may declare `imports`.

## Config Format

Global config:

```yaml
mcp:
  search_limit: 5

imports:
  - lang/python.yaml

scripts:
  - id: 1
    path: scripts/hello.py
    command: python {{path}}
    comment: Say hello to a user
    tags: [hello, text]
```

Imported config:

```yaml
scripts:
  - id: 2
    path: tools/build.py
    command: python {{path}}
```

Rules:

- only relative paths are allowed
- registry paths always use Unix-style `/` separators on every platform
- configuration always lives under `~/.config/sks`, never AppData
- imported files cannot declare `imports`
- `id` must be globally unique
- `command` must contain `{{path}}`

## Commands

```bash
sks init
sks list
sks pick
sks run 1 foo --bar baz
sks mcp
```

- `init` creates `~/.config/sks/sks.yaml`, an empty imported `scripts.yaml`, and the `sks-script-discovery` and `sks-script-authoring` Agent Skills under `~/.agents/skills`
- `list` outputs all registered scripts as YAML
- `pick` opens the interactive picker with a table-style list and syntax-highlighted file preview
- `run <id> [args...]` replaces `{{path}}` in `command` and appends all remaining args
- `mcp` runs a local MCP server over stdio

## MCP

Configure an MCP client to launch:

```json
{
  "command": "sks",
  "args": ["mcp"]
}
```

The server is read-only. It exposes one model-controlled tool, `search_scripts`, plus resources for the registry, script metadata, and source code. Search uses the same skim fuzzy matcher as the picker. Natural-language query terms drive recall; optional tags are soft ranking hints rather than required filters. `mcp.search_limit` in the global config controls the default number of results from 1 to 10; it defaults to 5, and a tool call can temporarily override it with `limit`. Imported configs cannot declare MCP options. Every request reloads the registry, so YAML changes are visible on the next search without restarting the server.

The MCP instructions use a search-before-authoring policy: for executable tasks, the model should search once before writing ad-hoc code or shell commands even when the user does not mention sks, local scripts, or existing tools. Calculations, conversions, file and data processing, generation, validation, and build workflows are triggers; purely conceptual discussion is not. Every explicit request to use a script must trigger discovery, regardless of whether the task appears simple or writing new code seems faster. The tool is also annotated as read-only, idempotent, and closed-world so clients can treat exploratory searches as low risk.

When a match is found, the result includes `sks run <id> [args...]` and resource URIs. The model can inspect source when arguments are unclear. An empty search result is a normal success and does not block the model from continuing another way.

`sks init` installs two complementary Agent Skills. `sks-script-discovery` tells compatible agents to discover and reuse registered scripts before one-off programming, while `sks-script-authoring` teaches them to author, register, validate, and test new scripts. Existing config and skill files are preserved unless `--force` is supplied. The final tool decision still belongs to the MCP client and model: server instructions and Skills improve invocation behavior but cannot force it at the protocol level.

## Picker

`pick` shows the script ID and comment in its result list:

- `ID`
- `COMMENT`

The preview pane renders the full script file with embedded `syntect` highlighting. The current default theme is GitHub Dark, with preview background handled by skim.

## Run Semantics

`run` is intentionally simple:

```bash
sks run 12 input.txt --mode fast
```

This means:

1. find script `id: 12`
2. replace `{{path}}` in `command`
3. append `input.txt --mode fast` to the command

`run` treats everything after `<id>` as passthrough arguments. It does not keep its own option parsing layer.

## Install

From source:

```bash
cargo install --path .
```
