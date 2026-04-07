# AGENTS.md

## Purpose
- This file gives coding agents repository-specific guidance for `log_analizor`.
- Follow these instructions first, then standard Rust best practices.
- Prefer small, focused changes over broad refactors.

## Project Snapshot
- Language: Rust (edition `2024`).
- Crate type: Binary crate.
- Entry point: `src/main.rs`.
- Current domain: log parsing and incident triage tools for a Strands agent.
- Main dependencies: `strands-agents`, `strands`, `tokio`, `serde`, `serde_json`.

## Repository Layout
- `Cargo.toml`: crate metadata and dependencies.
- `src/main.rs`: all runtime code, tools, and main workflow.
- `target/`: build artifacts (ignored by git).

## Setup
- Install stable Rust via `rustup`.
- Ensure `cargo`, `rustfmt`, and `clippy` are available.
- Optional but useful: `rustup component add rustfmt clippy`.

## Build Commands
- Build debug: `cargo build`
- Build release: `cargo build --release`
- Type-check quickly (no binary output): `cargo check`
- Run app: `cargo run`

## Test Commands
- Run all tests: `cargo test`
- Run tests in one file/module: `cargo test module_name`
- Run a single test by exact name: `cargo test test_name`
- Run a single test exactly: `cargo test test_name -- --exact`
- Show `println!` output: `cargo test test_name -- --nocapture`
- Run ignored tests: `cargo test -- --ignored`

## Lint And Formatting Commands
- Format code: `cargo fmt --all`
- Check formatting only: `cargo fmt --all -- --check`
- Lint: `cargo clippy --all-targets --all-features`
- Strict linting (recommended in CI): `cargo clippy --all-targets --all-features -- -D warnings`

## Recommended Local Validation Order
- 1) `cargo fmt --all -- --check`
- 2) `cargo clippy --all-targets --all-features -- -D warnings`
- 3) `cargo test`
- 4) `cargo run` for runtime sanity when behavior changes.

## Coding Style
- Use Rust idioms and keep functions focused on one responsibility.
- Favor clear, explicit code over clever compact patterns.
- Keep public behavior deterministic and easy to test.
- Avoid adding dependencies unless clearly justified.

## Imports
- Group imports by crate and keep them minimal.
- Prefer explicit imports over wildcard imports.
- Remove unused imports before finishing.
- Keep import ordering rustfmt-compatible.

## Formatting
- Always follow `rustfmt` output; do not hand-format against it.
- Keep lines reasonably short and readable.
- Use trailing commas in multiline literals/calls where rustfmt expects them.
- Prefer one statement per line unless expression chaining is clearer.

## Types And Data Modeling
- Prefer concrete domain structs (like `AppLog`) over loose maps.
- Derive traits intentionally (`Debug`, `Serialize`, `Deserialize`, etc.).
- Use `Option<T>` for truly optional values.
- Avoid `unwrap()` and `expect()` in production paths.
- Use `Result<T, E>` for fallible operations.
- Keep ownership/borrowing simple; pass references when cloning is unnecessary.

## Naming Conventions
- Types/traits/enums: `PascalCase`.
- Functions/variables/modules: `snake_case`.
- Constants/statics: `SCREAMING_SNAKE_CASE`.
- Use descriptive names tied to domain intent (`classify_incident`, `infer_cause`).
- Avoid abbreviations unless they are common domain terms.

## Error Handling
- Propagate errors with `?` when caller can act on them.
- Convert errors to user-friendly strings only at boundaries (CLI/tool output).
- Preserve context when mapping errors.
- Never silently swallow errors.
- Prefer typed errors for complex flows; `Box<dyn Error>` is acceptable at top-level boundaries.

## Async And Concurrency
- Use `#[tokio::main]` for async entrypoint (already in use).
- Keep async functions non-blocking; avoid long CPU-bound work in async contexts.
- If heavy CPU work is introduced, use blocking task strategies explicitly.

## Tool Function Guidelines (Strands)
- Keep each `#[tool]` function narrowly scoped and predictable.
- Validate and parse inputs defensively.
- Return structured, stable response formats when possible.
- Keep tool outputs concise and action-oriented.
- Reuse shared helper functions (like `infer_cause`) to avoid duplicated logic.

## Logging And Observability
- Prefer consistent message formats for machine readability.
- Avoid leaking secrets or sensitive tokens in outputs.
- Include enough context to support debugging (service, severity, cause).

## Testing Guidance
- Add unit tests for pure helper logic first.
- Add edge-case tests for malformed JSON and missing optional fields.
- Use exact-name test runs during iteration to keep feedback fast.
- Keep tests deterministic; avoid network-dependent tests by default.

## Dependency Guidance
- Keep dependency list small and maintained.
- Prefer mature crates with strong ecosystem support.
- Document rationale in PRs when adding a new crate.

## Commit And PR Guidance For Agents
- Make atomic commits per logical change.
- Write commit messages explaining why the change exists.
- Include validation steps run locally.
- Do not commit generated `target/` artifacts.

## Cursor/Copilot Rules In This Repository
- Checked for `.cursor/rules/`: not present.
- Checked for `.cursorrules`: not present.
- Checked for `.github/copilot-instructions.md`: not present.
- If any of these files are added later, treat them as authoritative and merge their guidance into this file.

## Agent Execution Checklist
- Confirm scope and touched files are minimal.
- Read `Cargo.toml` and relevant Rust source before editing.
- Apply changes consistent with existing naming and patterns.
- Run formatting, linting, and tests.
- Summarize changes, risks, and follow-up work.

## Notes Specific To Current Code
- Current code includes French user-facing strings; preserve language consistency unless asked otherwise.
- Existing severity logic uses simple thresholds; keep behavior explicit if modified.
- Main workflow builds a Strands agent and invokes tools on one sample log.

## When Extending The Project
- Prefer moving new logic into modules under `src/` once `main.rs` grows.
- Add tests alongside new modules.
- Keep CLI output readable and structured for operators.
