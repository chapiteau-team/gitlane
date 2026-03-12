# CLI Reference

This document tracks Gitlane CLI commands.

## Usage Pattern

```bash
gitlane [OPTIONS] <COMMAND>
```

Global options:

- `--project <PATH>`:
    - For `init`: target project root where `.gitlane` is created or updated. If omitted, uses current directory.
    - For other commands: path used as the starting point for `.gitlane` discovery. If omitted, starts from current
      directory. Discovery walks up parent directories (same style as `.git` discovery) and accepts either a
      `.gitlane` directory directly or a directory containing `.gitlane`. The resolved `.gitlane` directory must
      contain `project.toml`.
      Project config schema is documented in [`docs/project.md`](project.md).

## Supported Commands

- `init`
    - Purpose: create or update `.gitlane` repository structure and baseline config files.
    - Status: implemented.
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

Commands other than `init` are currently scaffolded.

## `init`

- Purpose: bootstrap Gitlane config and issue layout in a repository.
- Usage:

  ```bash
  gitlane [--project <PATH>] init [--name <NAME>] [--description <TEXT>] [--homepage <URL>]
  ```

- Options:
    - `--name <NAME>`: set project name in `.gitlane/project.toml`.
    - `--description <TEXT>`: set project description in `.gitlane/project.toml`.
    - `--homepage <URL>`: set project homepage in `.gitlane/project.toml`.

### Safety and idempotency

- `init` is safe-idempotent.
- Target project root is created when missing.
- Missing files and directories are created.
- Existing files are not overwritten, except `.gitlane/project.toml` when one or more of `--name`, `--description`,
  or `--homepage` is provided.
- When updating existing `project.toml`, only the provided fields are changed; all other keys are preserved.

### Files and directories created when missing

```text
.gitlane/
  project.toml
  issues/
    workflow.toml
    issues.toml
    labels.toml
    todo/
    in_progress/
    review/
    done/
```

### Default bootstrap configs

`workflow.toml`:

```toml
initial_state = "todo"

[states]
todo = { name = "To Do" }
in_progress = { name = "In Progress" }
review = { name = "In Review" }
done = { name = "Done" }

[transitions.todo]
start_work = { name = "Start work", to = "in_progress" }

[transitions.in_progress]
request_review = { name = "Request review", to = "review" }

[transitions.review]
approve = { name = "Approve", to = "done" }
request_changes = { name = "Request changes", to = "in_progress" }
```

`issues.toml`:

```toml
issue_prefix = "ISS"

priority_order = ["p1", "p2", "p3", "p4", "p0"]

[priorities]
p0 = { name = "No Priority", description = "Default when urgency is not assigned" }
p1 = { name = "Urgent", description = "Needs immediate attention" }
p2 = { name = "High", description = "Important and should be scheduled soon" }
p3 = { name = "Medium", description = "Normal planned work" }
p4 = { name = "Low", description = "Can be deferred" }
```

`labels.toml`:

```toml
[label_groups]
type = { name = "Type", description = "Issue classification", color = "#334155" }

[labels]
type_bug = { name = "Bug", description = "Unexpected behavior", group = "type" }
type_feature = { name = "Feature", description = "Net-new capability", group = "type" }
type_docs = { name = "Docs", description = "Documentation updates", group = "type" }
type_chore = { name = "Chore", description = "Maintenance and tooling work", group = "type" }
type_refactor = { name = "Refactor", description = "Internal structure improvements", group = "type" }

blocked = { name = "Blocked", description = "Waiting on external dependency", color = "#b91c1c" }
needs_decision = { name = "Needs Decision", description = "Requires product or technical decision", color = "#b45309" }
good_first_issue = { name = "Good First Issue", description = "Suitable for new contributors", color = "#0369a1" }
```

### Examples

```bash
gitlane init
gitlane --project ../my-repo init --name "My Repo"
gitlane init --description "Git-native tracker" --homepage "https://example.com"
```

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
