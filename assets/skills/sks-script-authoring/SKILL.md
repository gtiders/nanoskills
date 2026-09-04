---
name: sks-script-authoring
description: Create, register, update, and validate reusable local scripts managed by sks. Use when writing automation for the user, adding a script to the sks MCP registry, choosing useful script tags and descriptions, or troubleshooting an sks registration.
---

# Author an sks script

1. Call the sks MCP `search_scripts` tool before creating a script so existing automation is reused.
2. Choose the YAML registry and a nearby scripts directory.
3. Write a focused script with explicit CLI arguments, useful errors, and no embedded secrets.
4. Register it with a unique Python-style `name`, a relative Unix-style `path` using `/`, a `command` containing `{{path}}`, a concise capability-oriented `comment`, and 2–5 useful `tags`.
5. Run `sks list` to validate the complete registry.
6. Run `sks run <name> [args...]` with representative inputs to verify the registration and script together.

Do not use absolute paths or backslashes in registry YAML. Inspect the existing registry before choosing a name. Names must match `[A-Za-z_][A-Za-z0-9_]*`. If a script's arguments are unclear, inspect its source instead of inventing arguments.
