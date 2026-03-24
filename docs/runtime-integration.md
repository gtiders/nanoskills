# Runtime Integration

## Integration contract

Use `nanoskills` as the local registry layer:

1. build index: `nanoskills sync`
2. export tools: `nanoskills search <query> --json`
3. pass exported JSON to your agent runtime

## Example: generic JSON export

```bash
nanoskills search image --json > .nanoskills.tools.json
```

## Claude / Codex / OpenCode / OpenClaw

Different runtimes use different config schemas, but the same pattern applies:

- copy skill folders into runtime-visible `skills/` directory
- source tool metadata from `nanoskills search --json`

```bash
cp -R ./skills/* <RUNTIME_SKILLS_DIR>/
```

## Notes

- This project currently exports OpenAI-tools-compatible JSON shape.
- Runtime-specific adapters are maintained by each runtime environment.

## Extend scan_paths (global/local)

Global (`~/.config/nanoskills/.agent-skills.yaml`):

```yaml
scan_paths:
  - skills
  - /path/to/opencode/skills
  - /path/to/codex/skills
  - /path/to/claude/skills
  - /path/to/openclaw/skills
```

Project local (`./.agent-skills.yaml`):

```yaml
scan_paths:
  - .
  - ./skills
  - ./automation
```

Rebuild after any config change:

```bash
nanoskills sync
```

## System prompt policy (recommended)

Add this rule into your runtime's system prompt to make tool selection consistent:

```text
Before using other tools, first run:
`nanoskills search <intent> --json`
Choose the best matching tool from returned results.
If no good match exists, use fallback tools/reasoning.
```
