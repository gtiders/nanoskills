# AGENTS.md

This guide is for coding agents working in `nanoskills`. It reflects current repo structure, Cargo config, tests, and README.

## 1) Project Snapshot

- Language: Rust (edition `2024`)
- Build system: Cargo
- Binary crate: `nanoskills`
- Domain: local-first skill indexing/search CLI
- Entrypoint: `src/main.rs`

```text
src/
  cli/         # clap parsing + command handlers + output
  io/          # config loading, scanning, parsing, index persistence
  model/       # domain structs + serde models
  services/    # engine, indexing, search, sync orchestration
tests/         # integration tests (assert_cmd + predicates)
skills/        # seed skills packaged with release artifacts
```

## 2) Build / Lint / Test Commands

Run from repo root: `/home/gwins/Documents/nanoskills`.

### Core commands

```bash
cargo build
cargo build --release
cargo check
cargo fmt
cargo clippy --all-targets --all-features -D warnings
cargo test
```

### Single-test workflows (important)

Run one integration test file:

```bash
cargo test --test cli_search_test
```

Run one specific test function:

```bash
cargo test --test cli_search_test cli_search_json_outputs_lightweight_skill_array -- --nocapture
```

Alternative name filter:

```bash
cargo test cli_search_json_outputs_lightweight_skill_array -- --nocapture
```

Run tests in a specific module area under `src`:

```bash
cargo test io::config_loader
```

## 3) CI / Release Behavior

- Workflow: `.github/workflows/release.yml`
- Trigger: push tag `v*`
- Release targets:
  - `x86_64-unknown-linux-gnu`
  - `x86_64-unknown-linux-musl`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-pc-windows-msvc`
- Packaged assets include: `LICENSE`, `README.md`, `README_zh.md`, `skills/`

If you touch packaging or startup defaults, verify this workflow.

## 4) Code Style and Conventions

### Visibility and API boundaries

- Prefer `pub(crate)` for internal APIs.
- Keep public surface minimal (single binary crate).

### Naming

- Types/traits/enums: `PascalCase`
- Functions/modules/variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Tests: descriptive `snake_case` behavior names

### Imports and formatting

Typical import order:
1. `use crate::...`
2. external crates (`anyhow`, `clap`, `serde`, ...)
3. `std::...`

Let `rustfmt` own formatting; avoid manual style drift.

### Error handling

- Prefer `anyhow::Result` in application flow.
- Add context at IO/serialization boundaries via `.context(...)` / `.with_context(...)`.
- Use `bail!` for validated early exits.
- Prefer `?` propagation over manual branching.
- Preserve `main` error-chain output style (`error.chain().skip(1)`).

Avoid in non-test code: `unwrap()`, `expect()`, and swallowed errors.

### Serialization / config behavior

- Use `serde` derives + field attributes for defaults/custom parsing.
- Keep parsing backward-compatible where practical.
- Preserve human-readable values in config (`1MB`, `1h`, etc.; see `model/config.rs`).

### CLI and output behavior

- CLI shape is defined in `src/cli/cli.rs` via `clap` derive macros.
- Command handlers should remain thin and delegate to `services`.
- JSON output is a core contract; avoid breaking machine-readable fields.

### Determinism and comments

- Keep search/list ordering deterministic (tests assert stable tie behavior).
- Keep comments brief and intent-focused; existing code mixes concise English docs with occasional Chinese explanatory comments.

## 5) Testing Conventions

- Integration tests: `tests/*.rs`
- Shared harness: `tests/common/mod.rs` (`TestEnv`)
- Binary invocation pattern: `assert_cmd::Command::cargo_bin("nanoskills")`
- Test envs set locale and HOME (`LANG`, `LC_ALL`, `HOME`) for reproducibility.

When adding features:
- Prefer integration tests for CLI-visible behavior.
- Assert stdout/stderr with `predicates`.
- Validate JSON via `serde_json` parsing, not only string matching.

## 6) Agent Workflow Expectations

Before editing: read nearby modules/tests and match existing naming/import/error-context patterns.

After editing: run `cargo fmt`, `cargo clippy --all-targets --all-features -D warnings`, then targeted tests (and `cargo test` for broader changes).

## 7) Cursor / Copilot Rule Files

No repository-level rule files found at time of writing:

- `.cursorrules` (not found)
- `.cursor/rules/` (not found)
- `.github/copilot-instructions.md` (not found)

If these are added later, treat them as higher-priority instructions and update this file.
