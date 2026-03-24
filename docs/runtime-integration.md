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
