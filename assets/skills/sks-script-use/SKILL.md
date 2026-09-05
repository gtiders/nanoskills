---
name: sks-script-use
description: Discover and reuse registered sks scripts before writing one-off code or shell commands. Use for concrete tasks that could be performed by a script or local automation, including calculations, conversions, file or data processing, content generation, validation, and build or development workflows, even when the user does not explicitly mention sks, local scripts, or existing tools.
---

# Use an sks script

1. For an executable task, search once with the sks MCP `search_scripts` tool before writing or running ad-hoc code or shell commands. Use a concise capability query and preserve the user's domain terms.
2. Treat every request to use a script as an explicit search trigger, regardless of task size.
3. Select matches by `comment` and `tags`. Read the source resource or run `--help` when arguments or behavior are unclear; do not invent arguments.
4. Run a selected script with `sks run <name> [args...]` when execution is available and appropriate. The name is case-sensitive and must be copied exactly from the result.
5. If no useful match exists, continue with another approach. Do not repeat equivalent searches.

Skip discovery for purely conceptual discussion that requires no execution. Do not force a script when no registered script matches the task.
