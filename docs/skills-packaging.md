# Skills Packaging Layout

## Layout rule

Use **one skill per folder**, folder name aligned with skill identity.

```text
skills/
  nanoskills_project_builder/
    nanoskills_project_builder.md
  nanoskills_usage_guide/
    nanoskills_usage_guide.md
```

## Why this layout

- easier bulk copy to another tool/runtime
- clearer ownership and versioning per skill
- lower risk when merging team changes

## Distribution workflow

1. unpack release archive
2. copy `skills/*` into target runtime's `skills/` path
3. run `nanoskills sync` where needed

## Global init seeding

On first global `nanoskills init`, the project can seed initial skill files into:

```text
~/.config/nanoskills/skills/
```

Existing files are preserved (non-overwrite behavior).
