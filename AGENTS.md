# AGENTS.md

## What Matters In This Repo
- Single Rust binary crate (`edition = 2024`), entrypoint is `src/main.rs`.
- No workspace, no CI config, no codegen pipeline; keep changes small and local.
- Runtime depends on local Ollama (`OllamaModel`), not Bedrock.

## Verified Commands
- Prefer Make targets when possible:
  - `make check` -> `cargo check`
  - `make fmt` -> `cargo fmt --all`
  - `make clippy` -> `cargo clippy --all-targets --all-features`
  - `make test` -> `cargo test`
  - `make test-one TEST=name` -> exact single test
  - `make run` -> `cargo run`
- Equivalent direct commands are valid if Make is unavailable.

## Validation Order
- Run in this order before finishing code changes:
  1. `cargo fmt --all -- --check`
  2. `cargo clippy --all-targets --all-features`
  3. `cargo test`

## Environment And Runtime
- `.env` is loaded via `dotenvy::dotenv()` in `main`.
- Expected vars (see `.env.example`):
  - `OLLAMA_MODEL` (required)
  - `OLLAMA_HOST` (required)
- `.env` is gitignored; do not commit local secrets.
- For local execution, Ollama service must be running (for example: `ollama serve`).

## Strands-Specific Gotcha
- Do not reintroduce `#[tool]` macro usage for tools in this repo.
- With the pinned dependency set in `Cargo.toml`, macro-generated code is incompatible here; tools are implemented manually via `AgentTool` (`ParseLogTool`, `ClassifyIncidentTool`, `SuggestFixTool`).

## Code Structure That Agents Should Preserve
- Core parsing/classification helpers are plain functions (`parse_log`, `classify_incident`, `suggest_fix`, `infer_cause`).
- Tool structs are thin wrappers around those helpers; keep business logic in helpers, not in `invoke` bodies.
- Current operator-facing strings are French-oriented; keep language consistency unless asked to change it.

## Dependency Policy
- Dependency versions are intentionally pinned with `=` in `Cargo.toml`; do not broaden ranges unless explicitly requested.

## Instruction Files Check
- `.cursor/rules/`: not present
- `.cursorrules`: not present
- `.github/copilot-instructions.md`: not present
