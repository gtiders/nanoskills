# Repository Guidelines

## Project Structure & Module Organization
- `src/` contains the Rust application code, split by layers:
- `src/domain/`: core models (`skill`, `index`, `tool_name`).
- `src/infra/`: filesystem scanning, parsing, config loading, and index storage.
- `src/app/`: orchestration/services (`sync`, `search`, engine logic).
- `src/presentation/`: CLI commands and TUI/output rendering.
- `tests/` holds integration-style CLI tests (for example `cli_search_test.rs`) plus shared helpers in `tests/common/`.
- `locales/` stores i18n YAML files; `test_scripts/` contains sample skill files used for parsing/index coverage.

## Build, Test, and Development Commands
- `cargo build` compiles debug binaries.
- `cargo build --release` builds optimized binaries.
- `cargo run -- <command>` runs locally, e.g. `cargo run -- search image`.
- `cargo test` runs all tests in `tests/` and unit tests.
- `cargo fmt` formats code with `rustfmt` defaults.
- `cargo clippy --all-targets --all-features -D warnings` enforces lint cleanliness.
- `cargo install --path .` installs the local CLI as `nanoskills`.

## Coding Style & Naming Conventions
- Follow idiomatic Rust 2024 style and keep code `rustfmt`-clean.
- Use `snake_case` for modules/functions/files, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for constants.
- Keep module responsibilities narrow by layer (domain vs infra vs app vs presentation).
- Prefer descriptive command/service names (`config_loader`, `index_service`, `tool_name_resolver`).

## Testing Guidelines
- Place behavior-driven CLI coverage in `tests/` using `assert_cmd`, `predicates`, and `tempfile`.
- Name test files with the `*_test.rs` pattern and group helper utilities under `tests/common/`.
- Add regression tests for each command change (`init`, `sync`, `search`, `list`, `pick`) and config-resolution edge cases.
- Run `cargo test` before pushing; include tests for bug fixes.

## Commit & Pull Request Guidelines
- Follow Conventional Commit style seen in history: `feat:`, `fix:`, `refactor:`, `docs:`, `build:`, `release:`.
- Keep subjects imperative and scoped (example: `fix: support global and local init config scopes`).
- PRs should include a concise behavior summary, linked issue(s) when relevant, before/after CLI output (or TUI screenshots), and confirmation that `cargo fmt`, `cargo clippy`, and `cargo test` passed.
