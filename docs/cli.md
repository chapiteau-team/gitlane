# CLI Reference

This document tracks Gitlane CLI commands.

## Usage Pattern

```bash
gitlane [OPTIONS] <COMMAND>
```

Global options:

- `--project <PATH>`: path used as the starting point for `.gitlane` discovery. If omitted, starts from the current
  directory. Discovery walks up parent directories (same style as `.git` discovery) and accepts either a `.gitlane`
  directory directly or a directory containing `.gitlane`. The resolved `.gitlane` directory must contain
  `project.toml`.
  Project config schema is documented in [`docs/project.md`](project.md).

## Supported Commands

- `init`
    - Purpose: prepare `.gitlane` repository structure and baseline config files, including `.gitlane/project.toml`.
    - Status: scaffolded, not implemented.
- `validate`
    - Purpose: validate workflow, issue, and label configuration and data shape.
    - Status: scaffolded, not implemented.
- `issue`
    - `create`: create a new issue in the workflow initial state.
    - `list`: list issues with deterministic ordering.
    - `show <id>`: display one issue by id.
    - `transition <id> <transition_id>`: move an issue using a workflow transition.
    - Status: scaffolded, not implemented.
- `workflow`
    - `show`: display workflow graph and initial state.
    - `states`: display known workflow states.
    - `transitions [--from <state_id>]`: display transitions, optionally filtered by source state.
    - Status: scaffolded, not implemented.
- `label`
    - `list`: list configured labels and groups.
    - `show <id>`: display one label definition.
    - Status: scaffolded, not implemented.

Current scaffold behavior: each command returns a deterministic not-implemented error.

## Command Entry Template

### `<command>`

- Purpose:
- Usage:
  ```bash
  gitlane <command> [options]
  ```
- Arguments:
- Options:
- Examples:
- Notes:
