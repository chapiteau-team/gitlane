# AGENTS.md

Guide for agentic coding assistants operating in this repository.

## 1) Repository Overview
- Language: Rust
- Build tool: Cargo
- Edition: 2024 (workspace members)
- Current crate shape: Cargo workspace with two crates:
  - `crates/core`: reusable core library package (`gitlane`)
  - `crates/cli`: CLI binary package (`gitlane-cli`, binary name: `gitlane`)
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
- Build all (debug): `cargo build --workspace`
- Build all (release): `cargo build --workspace --release`
- Typecheck quickly: `cargo check --workspace`
- Run CLI app: `cargo run -p gitlane-cli -- <command> [options]`
- Run CLI app (release): `cargo run -p gitlane-cli --release -- <command> [options]`

### Formatting
- Format code: `cargo fmt`
- Check formatting (CI style): `cargo fmt --all -- --check`

### Linting
- Recommended strict lint command:
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- If needed for speed in large dependency graphs:
  `cargo clippy --workspace --all-targets --all-features --no-deps -- -D warnings`

### Tests
- Run all tests: `cargo test --workspace`
- Run tests and show stdout/stderr: `cargo test --workspace -- --nocapture`
- List discovered tests: `cargo test --workspace -- --list`

### GitHub CI
- Workflow file: `.github/workflows/ci.yml`
- Triggers: pushes to `main`, pull requests, and `workflow_dispatch`
- `lint` job (`ubuntu-latest`):
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `test` job matrix (`ubuntu-latest`, `windows-latest`, `macos-latest`):
  - `cargo test --workspace`

### Running a Single Test (important)
- By substring match: `cargo test -p <package> name_substring`
- Exact unit test name: `cargo test -p <package> exact_test_name -- --exact`
- Entire integration test file: `cargo test -p <package> --test integration_file_stem`
- One test in one integration target:
  `cargo test -p <package> --test integration_file_stem exact_test_name -- --exact`

### Useful test runner flags
- Serial execution: `cargo test --workspace -- --test-threads=1`
- Skip by pattern: `cargo test --workspace -- --skip flaky_case`
- Ignored tests only: `cargo test --workspace -- --ignored`
- Include ignored + normal: `cargo test --workspace -- --include-ignored`

## 4) Suggested Local Verification Before Handoff
Run these before finalizing substantial changes:
1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo test --workspace`

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

### Domain Docs (Read First)
- Treat `docs/` as the canonical source for product behavior and data-model semantics.
- Current `init` behavior is create-only: it may complete a partial `.gitlane/` scaffold when no supported project
  config file exists (`.gitlane/project.toml`, `.gitlane/project.json`, `.gitlane/project.yaml`, or
  `.gitlane/project.yml`), but it must fail once one already exists.
- `init` accepts `--format toml|json|yaml|yml` for config files it creates and defaults to `toml`.
- `init` also scaffolds `.gitlane/issues/templates/default/issue.md`; the default template front matter format matches
  `init --format`.
- If more than one supported config file exists for the same logical config (`project`, `workflow`, `issues`, or
  `labels`), treat that as an error rather than guessing.
- Workflow state ids and `issue_prefix` values become filesystem path segments; keep them portable across Linux,
  macOS, and Windows per `docs/issues/workflow.md` and `docs/issues/issues.md`.
- Planned `issue create` template resolution uses `.gitlane/issues/templates/default/` when `--template` is omitted,
  `.gitlane/issues/templates/<name>/` when `--template <name>` is provided, or `<templates-path>/<name-or-default>/`
  when `--templates-path` is provided. Template names are single directory names, template roots must contain
  `issue.md`, and omitted reporters fall back to `git config user.name` but must still satisfy any configured
  `people` list.
- Before changing CLI behavior or config/data validation, read the relevant doc first:
  - `docs/cli.md` (command surface and expected behavior)
  - `docs/project.md` (`.gitlane/project.{toml,json,yaml,yml}` schema)
  - `docs/issues/workflow.md` (state graph and transition rules)
  - `docs/issues/issues.md` (issue IDs, front matter, and filesystem layout)
  - `docs/issues/labels.md` (label/group schema and constraints)
- Keep `AGENTS.md` as a pointer/index; do not duplicate full schema rules from `docs/`.
- If implementation and docs diverge, call out the mismatch in handoff; when behavior changes intentionally, update `docs/` and `AGENTS.md` in the same change set.

## 8) Quick Command Cheat Sheet
- Build: `cargo build --workspace`
- Check: `cargo check --workspace`
- Run CLI: `cargo run -p gitlane-cli -- <command>`
- Format check: `cargo fmt --all -- --check`
- Lint: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Test all: `cargo test --workspace`
- GitHub CI workflow: `.github/workflows/ci.yml`
- Test one (substring): `cargo test -p <package> name_substring`
- Test one (exact): `cargo test -p <package> exact_test_name -- --exact`
- List tests: `cargo test --workspace -- --list`

## 9) File Maintenance Policy
When build, lint, test, or style conventions change, update `AGENTS.md` in the same change set so future agents stay aligned with real repo behavior.

## 10) Notes for Future Growth
- If workspace members are added, include per-crate commands and preferred run targets.
- If CI is added, mirror exact CI commands here.
- If custom lint configs appear (`clippy.toml`, `rustfmt.toml`), document key rules.
- If Cursor/Copilot rules are added later, copy key constraints into this file.
