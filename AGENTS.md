# AGENTS.md

## What Matters In This Repo
- Single Rust crate (`edition = 2024`) with bin + lib split.
- Runtime entrypoint is `src/main.rs`; internal modules are exported from `src/lib.rs`.
- No workspace, no CI config, no codegen pipeline; keep changes small and local.
- Runtime depends on local Ollama via `rig::providers::ollama`, not Bedrock.

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
  - `CONTEXT7_ENABLED` (optional, enables Context7 outbound calls)
  - `CONTEXT7_API_KEY` (optional, used to enrich `suggest_fix`)
  - `CONTEXT7_DEBUG` (optional, prints tested Context7 candidates)
- `.env` is gitignored; do not commit local secrets.
- For local execution, Ollama service must be running (for example: `ollama serve`).

## Rig-Specific Gotcha
- `rig-core` is used with `derive` enabled.
- Prefer `#[rig::tool_macro]` for simple stateless tools (`ParseLogTool`, `ClassifyIncidentTool`).
- Keep complex stateful tools implemented manually with `rig::tool::Tool` (`SuggestFixTool`).

## Code Structure That Agents Should Preserve
- Keep the layering: `main` (wiring) -> `config` (env) + `tools` (wrappers Tool Rig) -> `domain` (business logic).
- Core parsing/classification helpers stay in `domain` (`parse_log`, `classify_incident`, `suggest_fix`, `infer_cause`).
- Tool structs stay thin wrappers around domain helpers; keep business logic out of `call` bodies.
- `suggest_fix` enriches output through Context7 by searching available libraries first, then querying docs on top ranked candidates.
- `sample_logs` provides randomized local scenarios for manual runs; keep it focused on representative formats/errors.
- Current operator-facing strings are French-oriented; keep language consistency unless asked to change it.

## Dependency Policy
- Dependency versions are intentionally pinned with `=` in `Cargo.toml`; do not broaden ranges unless explicitly requested.

## Instruction Files Check
- `.cursor/rules/`: not present
- `.cursorrules`: not present
- `.github/copilot-instructions.md`: not present
