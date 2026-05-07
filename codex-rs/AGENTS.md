# Repository Guidelines

## Project Structure & Module Organization
The workspace is rooted at `codex-rs`, with Rust crates in directories prefixed by `codex-`. Core platform logic lives in `core/` (`codex-core`) and shared utilities in `common/`. Protocol types and transport code sit in `protocol/`, while the interactive terminal client is in `tui/` (`codex-tui`) and command-line tooling in `cli/` (`codex-cli`). Front-end bindings and ancillary tools are kept in sibling folders such as `protocol-ts/` and `docs/`. Generated or build artifacts land in `target/`; do not commit its contents.

## Build, Test, and Development Commands
Run `just install` to verify the toolchain and fetch dependencies. Use `cargo run -p codex-cli -- <args>` or the shorthand recipes `just codex`, `just exec`, and `just tui-with-exec-server` for local execution. Format the workspace with `just fmt` immediately after Rust edits, and prefer `just fix -p <crate>` to apply scoped clippy fixes. For fast end-to-end checks, `cargo nextest run --no-fail-fast` mirrors CI.

## Coding Style & Naming Conventions
Follow Rust 2021 standards with four-space indentation. Crate names remain `codex-*`, and modules use snake_case. Inline variables directly in `format!` calls (e.g., `format!("Hello {name}")`). TUI code should rely on ratatui’s `Stylize` helpers (`"text".dim().into()`) instead of manual `Style` construction. Avoid touching constants tied to sandbox env vars.

## Editing & Safety Rules
- Prefer `apply_patch` for single-file edits. It’s acceptable to use other tools when `apply_patch` is impractical (generated files, scripted search/replaces, etc.).
- Never run destructive git commands like `git reset --hard` or `git checkout --` unless the user explicitly asks or approves.
- Always set the `workdir` parameter when calling `shell`; avoid using `cd` unless absolutely required.

## Agent Response Formatting
- Code samples or multi-line snippets must be wrapped in fenced code blocks, including an info string (language hint) whenever possible.

## Testing Guidelines
Targeted crates run with `cargo test -p codex-<crate>`. When snapshots change in `codex-tui`, recreate them via `cargo insta show`/`accept` after reviewing `*.snap.new`. Prefer `pretty_assertions::assert_eq` in new tests for readable diffs. CI expects critical paths to be covered; add focused integration tests under each crate’s `tests/` directory when behavior spans modules.

## Commit & Pull Request Guidelines
Write imperative, scope-aware commit subjects, mirroring history (`render • as dim (#4467)`). Reference the related issue or PR number when possible. Pull requests should include a concise summary of changes, affected crates, tests executed, and screenshots for UI shifts. Keep diffs minimal, and update documentation or snapshots alongside feature work.

## Environment & Security Notes
Respect the sandbox signals (`CODEX_SANDBOX`, `CODEX_SANDBOX_NETWORK_DISABLED`) that gate networked or seatbelt-specific flows. Tests that check for these flags should short-circuit instead of forcing network access. Never commit secrets; prefer `.env`-style local overrides.
