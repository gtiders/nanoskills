# Script Skillization

## Principle

A skill does not need to be Markdown-only. In many cases, a lightly structured script is already a skill.

`nanoskills` recognizes files that contain a YAML header near the top.

## Minimal example (Python)

```python
# ---
# name: disk_check
# description: Check disk usage and warn on high usage
# tags: [ops, monitoring]
# tool_name: disk_check
# args:
#   disk_path:
#     type: string
#     description: Target disk path
#     required: false
# ---
print("...")
```

## Recommended practice

- Keep original script logic unchanged when possible.
- Add clear `name`, `description`, and `args` for agent understanding.
- Set explicit `tool_name` for stable integration across environments.
- Use one-skill-per-folder for portability.

## Validate

```bash
nanoskills sync --strict
nanoskills search disk --json
```

`--strict` helps catch malformed headers early.
