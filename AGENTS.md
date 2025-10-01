# Repository Guidelines

## Project Structure & Module Organization
FocusFive's executable code lives in `src/`: `main.rs` seeds sample goals, `models.rs` hosts the 3×3 `DailyGoals` structures, `data.rs` handles atomic Markdown I/O, and `lib.rs` re-exports shared APIs. Reference material stays in `docs/` and focused Markdown primers at the repo root. Integration and regression coverage reside in `tests/`. Runtime data is never versioned—goal files are created under `~/FocusFive/goals/` (e.g. `~/FocusFive/goals/2025-09-19.md`). Utility scripts such as `validate_setup.sh` and `debug_goals.sh` live beside `Cargo.toml`.

## Build, Test, and Development Commands
- `cargo build --release` – compile a production binary into `./target/release/focusfive`.
- `./target/release/focusfive` – launch the terminal client against the home-directory store.
- `cargo test` – run all parser, data-layer, and regression checks.
- `cargo test -- --nocapture` – display inline assertions while debugging.
- `./validate_setup.sh` – confirm Rust toolchain, goal directory permissions, and script prerequisites.

## Coding Style & Naming Conventions
Use idiomatic Rust with four-space indentation and `snake_case` modules, functions, and files. Always surface failures with `Result<T>` and `.context(...)` instead of panics; avoid `.unwrap()`/`.expect()` in application code. Preserve the 3 outcomes × 3 actions invariant when evolving models and document any deviation before merging. Favor descriptive enums (`OutcomeType::Work`) and keep business logic inside `models` or `data`. Run `cargo clippy -- -D warnings`; `cargo fmt` is optional when the local toolchain supports it.

## Testing Guidelines
Add new coverage inside `tests/`, mirroring files such as `regression_tests.rs` or `parser_verification.rs`. Name suites after the feature under examination (`*_integration.rs`, `*_tests.rs`). Reproduce Markdown scenarios with fixtures from `examples/` or the `test_*.sh` scripts. Execute `cargo test` and `./validate_setup.sh` before requesting review, and note any additional fixtures in accompanying docs.

## Commit & Pull Request Guidelines
Follow the Conventional Commit style visible in history (`feat:`, `fix:`, `refactor:`). Each commit should focus on a single behavior change and update relevant guides (`CLAUDE.md`, `USER_GUIDE.md`, this file) when functionality shifts. Pull requests must summarise scope, list verification commands, and mention effects on the home-directory goal files. Link tracking issues or tickets when available and wait for CI or local validation to pass.

## Configuration Notes for Agents
Ensure `$HOME` resolves before running tests; the application falls back to the current directory only when HOME is unset. Do not add network services—FocusFive is local-first by requirement. For concurrency scenarios, rely on the atomic helpers in `data.rs` instead of manual file edits.
