# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [1.0.1] - 2026-03-24

### Added

- Project-level `skills/` pack with one-skill-per-folder layout for easier runtime portability
- Extended integration docs for `scan_paths` and system-prompt-first tool routing
- Additional docs pages for positioning, script skillization, runtime integration, and packaging

### Changed

- `init` global seeding now copies skill files recursively from bundled/current `skills/`
- Unified default config/cache paths to `~/.config/nanoskills` and `~/.cache/nanoskills`
- README and README_zh restructured around product positioning, pain points, and runtime copy flow

### Added

- Layered project structure split into `app`, `domain`, `infra`, and `presentation`
- Stable, legal, deduplicated tool identifiers for `search --json`
- Agent-ready JSON output with TTY-aware syntax highlighting
- Structured integration test suite based on `assert_cmd`, `predicates`, and `tempfile`
- Bilingual documentation with `README.md` and `README_zh.md`

### Changed

- Polished CLI help text and user-facing copy across CLI, TUI, and localized messages
- Refined error messages to be more human-friendly and action-oriented
- Split CLI output, sync pipeline, and TUI internals into smaller focused modules

### Fixed

- Preserved plain JSON output in non-TTY environments for Agent and pipeline compatibility
- Improved fallback handling for missing or corrupted local indexes
- Hardened strict-mode sync behavior around invalid headers and duplicate `tool_name` values
