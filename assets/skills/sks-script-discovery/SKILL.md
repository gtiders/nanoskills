---
name: sks-script-discovery
description: Discover and reuse registered sks scripts before writing one-off code or shell commands. Use for concrete tasks that could be performed by a script or local automation, including calculations, conversions, file or data processing, content generation, validation, and build or development workflows, even when the user does not explicitly mention sks, local scripts, or existing tools.
---

# Discover an sks script

1. Before writing, generating, or running ad-hoc code or shell commands for an executable task, call the sks MCP `search_scripts` tool once with a concise capability query. Preserve important domain terms and the user's language.
2. Treat every request to use a script as an explicit search trigger, regardless of the task's apparent simplicity or whether writing new code seems faster.
3. Choose useful matches by description and tags. Read the script resource when its arguments or behavior are unclear; never invent arguments.
4. Run a selected script with `sks run <id> [args...]` when execution is available and appropriate.
5. If no useful match exists, continue with the normal approach. Do not repeat equivalent searches.

Skip discovery for purely conceptual discussion that requires no execution. A search is cheap and read-only; do not require the user to ask for an existing or local tool.
