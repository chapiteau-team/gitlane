# AGENTS.md

Guide for agentic coding assistants operating in this repository.

## 1) Repository Overview
- Language: Rust
- Build tool: Cargo
- Edition: 2024 (`Cargo.toml`)
- Current crate shape: single binary target (`src/main.rs`)
- Root to run commands from: `.` (repository root)

This repo is currently small; use idiomatic Rust defaults and keep changes minimal.

## 2) Rule Files (Cursor/Copilot)
The following rule locations were checked:
- `.cursor/rules/` -> not present
- `.cursorrules` -> not present
- `.github/copilot-instructions.md` -> not present

If any of these files appear later, treat them as high-priority constraints and update this document in the same PR.

## 3) Build / Run / Lint / Test Commands

### Build + Run
- Build (debug): `cargo build`
- Build (release): `cargo build --release`
- Typecheck quickly: `cargo check`
- Run app: `cargo run`
- Run app (release): `cargo run --release`

### Formatting
- Format code: `cargo fmt`
- Check formatting (CI style): `cargo fmt --all -- --check`

### Linting
- Recommended strict lint command:
  `cargo clippy --all-targets --all-features -- -D warnings`
- If needed for speed in large dependency graphs:
  `cargo clippy --all-targets --all-features --no-deps -- -D warnings`

### Tests
- Run all tests: `cargo test`
- Run tests and show stdout/stderr: `cargo test -- --nocapture`
- List discovered tests: `cargo test -- --list`

### Running a Single Test (important)
- By substring match: `cargo test name_substring`
- Exact unit test name: `cargo test exact_test_name -- --exact`
- Entire integration test file: `cargo test --test integration_file_stem`
- One test in one integration target:
  `cargo test --test integration_file_stem exact_test_name -- --exact`

### Useful test runner flags
- Serial execution: `cargo test -- --test-threads=1`
- Skip by pattern: `cargo test -- --skip flaky_case`
- Ignored tests only: `cargo test -- --ignored`
- Include ignored + normal: `cargo test -- --include-ignored`

## 4) Suggested Local Verification Before Handoff
Run these before finalizing substantial changes:
1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test`

If one cannot be run locally, explicitly note that in your handoff.

## 5) Rust Style Guidelines

### Formatting and structure
- Use `rustfmt` defaults unless repo-level config is added.
- Prefer small, focused functions and clear control flow.
- Avoid clever one-liners when they reduce readability.

### Imports
- Organize imports by groups: `std`, external crates, local crate modules.
- Prefer explicit imports over wildcard imports.
- Remove unused imports; do not silence warnings unnecessarily.

### Naming
- Types/traits/enums: `UpperCamelCase`
- Functions/modules/variables: `snake_case`
- Constants/statics: `SCREAMING_SNAKE_CASE`
- Test names should describe behavior and expected outcome.

### Types and API boundaries
- Prefer domain-specific types at important boundaries.
- Prefer borrowing (`&str`, `&[T]`, `&T`) over ownership when possible.
- Keep public surface area minimal; default to private visibility.
- Derive traits intentionally (`Debug`, `Clone`, `PartialEq`, etc.) only where needed.

### Error handling
- Prefer `Result<T, E>` and propagate with `?`.
- Avoid `unwrap()`/`expect()` in non-test production paths.
- Add context to errors at I/O, parsing, and subprocess boundaries.
- Use `Option<T>` only when absence is expected and non-exceptional.
- In tests, `expect("clear failure reason")` is encouraged.

### Control flow and ownership
- Prefer early returns over deep nesting.
- Use `match` when it clarifies exhaustive cases.
- Minimize cloning; borrow first, clone when ownership is required.
- Keep lifetime annotations implicit unless explicit lifetimes aid comprehension.

### Comments and docs
- Favor self-explanatory code over excessive comments.
- Add comments for invariants, assumptions, or non-obvious tradeoffs.
- Add doc comments to public APIs as modules become library-like.

### Logging and output
- Keep CLI output deterministic and user-focused.
- Do not leave debug prints in committed code unless intentional.
- If structured logging is introduced, use consistent field naming.

## 6) Testing Guidelines
- Keep unit tests close to code in `mod tests`.
- Put integration tests in `tests/` with descriptive file names.
- Prefer deterministic tests; avoid sleep-based timing checks.
- Use fixtures/helpers to reduce duplication as suite grows.
- Validate both success and failure paths for non-trivial logic.

## 7) Agent Behavior in This Repo
- Make focused diffs; avoid unrelated refactors.
- Preserve existing conventions unless user requests a style shift.
- Do not add dependencies without clear need.
- If adding tooling/scripts, document new commands here immediately.
- Mention verification commands executed in final handoff.

## 8) Quick Command Cheat Sheet
- Build: `cargo build`
- Check: `cargo check`
- Format check: `cargo fmt --all -- --check`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Test all: `cargo test`
- Test one (substring): `cargo test name_substring`
- Test one (exact): `cargo test exact_test_name -- --exact`
- List tests: `cargo test -- --list`

## 9) File Maintenance Policy
When build, lint, test, or style conventions change, update `AGENTS.md` in the same change set so future agents stay aligned with real repo behavior.

## 10) Notes for Future Growth
- If workspace members are added, include per-crate commands.
- If CI is added, mirror exact CI commands here.
- If custom lint configs appear (`clippy.toml`, `rustfmt.toml`), document key rules.
- If Cursor/Copilot rules are added later, copy key constraints into this file.
