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

- `init` creates `~/.config/sks/sks.yaml`, an empty imported `scripts.yaml`, and the `sks-script-authoring` Agent Skill under `~/.agents/skills`
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

When a match is found, the result includes `sks run <id> [args...]` and resource URIs. The model can inspect source when arguments are unclear. An empty search result is a normal success and does not block the model from continuing another way.

`sks init` also installs a concise Agent Skill that teaches compatible coding agents how to author, register, validate, and test new scripts. Existing config and skill files are preserved unless `--force` is supplied.

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
