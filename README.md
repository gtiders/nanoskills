# sks

Registry-based launcher for local scripts. `sks` loads explicit YAML registrations, lists and searches them, runs a selected script, and exposes the registry through MCP.

## Features

- Exact execution by Python-style ASCII name: `[A-Za-z_][A-Za-z0-9_]*`.
- YAML registry with imports, descriptions, and tags.
- Interactive picker with source preview and syntax highlighting.
- MCP search and read-only resources for script metadata and source.
- Pre-execution snapshot to `.sks/<filename>` in the current directory.
- Self-update from the latest GitHub Release with target-aware asset selection and checksum verification.
- Built-in instructions for script use and script creation.

## Requirements

- Rust 1.85 or newer when building from source.
- Python or another runtime required by each registered script.
- An MCP client is required only for MCP integration.

## Installation

Build and install from source:

```bash
git clone https://github.com/gtiders/skillscripts.git
cd skillscripts
cargo install --path .
```

Initialize the global configuration:

```bash
sks init
```

The configuration directory is `~/.config/sks`. `init` creates `sks.yaml`, an empty `scripts.yaml`, and the `sks-script-use` and `sks-script-create` Agent Skills under `~/.agents/skills`.

## Usage

```bash
sks list
sks pick
sks run <name> [args...]
sks skill use
sks skill create
sks update
sks update --check
sks update --force
```

`run` matches `name` exactly. All arguments after the name are appended to the registered command. Before execution, the source file is copied to `.sks/<filename>` in the current directory; an existing file with the same name is replaced. The copy is made even if the command exits with an error.

`list` prints the effective registry as YAML. `pick` provides an interactive name and comment list with a source preview.

`update` queries the GitHub latest Release, selects the asset matching the binary's compiled Rust target (including GNU or musl), verifies `checksums.txt`, and replaces the current executable. `--check` does not install; `--force` installs even when the version comparison is inconclusive.

### MCP

Configure an MCP client to start the server over stdio:

```json
{
  "command": "sks",
  "args": ["mcp"]
}
```

The server provides `search_scripts` and read-only resources:

```text
sks://registry
sks://scripts/<name>
sks://scripts/<name>/source
```

Search results include the command form `sks run <name> [args...]`. `mcp.search_limit` controls the default result count from 1 to 10.

## Configuration

Global file: `~/.config/sks/sks.yaml`

```yaml
mcp:
  search_limit: 5

imports:
  - scripts.yaml
  - imports/tools.yaml

scripts: []
```

Script registration:

```yaml
scripts:
  - name: ase_to_xyz
    path: tools/ase2xyz.py
    command: python {{path}}
    comment: Convert ASE-readable structure files to extended XYZ
    tags: [ase, structure, extxyz, conversion]
```

Rules:

- `name` is required, case-sensitive, and globally unique.
- A name must match `[A-Za-z_][A-Za-z0-9_]*`. Empty, Unicode, numeric-leading, dotted, dashed, slashed, and spaced names are invalid.
- `path` must be a relative Unix-style path. It is resolved relative to the YAML file that defines it.
- Only the global file may declare `imports`; imported files cannot import another file or declare `mcp`.
- `command` must contain `{{path}}`. The placeholder is replaced with the resolved script path.
- `comment` and `tags` are optional. Tags are used as search ranking hints.

## Common Issues

### `Global config not found`

Run `sks init`, then add registrations to `~/.config/sks/sks.yaml` or an imported YAML file.

### `invalid script name`

Rename the registration to an ASCII Python-style identifier, for example `convert_csv` or `_internal`.

### `unknown script name`

Run `sks list` and use the exact registered name. Matching is case-sensitive and does not use fuzzy guessing.

### `command` validation fails

Add `{{path}}` to the command. For example: `python {{path}}`.

### `sks update` cannot find an asset

The release must provide an archive for the binary's compiled target and a matching `checksums.txt`. Check the network connection and the available assets on the latest GitHub Release.
