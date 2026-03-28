# CLI Reference

This document tracks Gitlane CLI commands.

## Usage Pattern

```bash
gitlane [OPTIONS] <COMMAND>
```

Global options:

- `--project <PATH>`:
    - For `init`: target project root where `.gitlane` is created. If `.gitlane/project.toml`, `.gitlane/project.json`,
      `.gitlane/project.yaml`, or `.gitlane/project.yml` already exists, `init` returns an error. If omitted, uses
      current directory.
    - For other commands: path used as the starting point for `.gitlane` discovery. If omitted, starts from current
      directory. Discovery walks up parent directories (same style as `.git` discovery) and accepts either a
      `.gitlane` directory directly or a directory containing `.gitlane`. The resolved `.gitlane` directory must
      contain exactly one supported project config file: `project.toml`, `project.json`, `project.yaml`, or
      `project.yml`.
      Project config schema is documented in [`docs/project.md`](project.md).

## Supported Commands

- `init`
    - Purpose: create `.gitlane` repository structure and baseline config files.
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
  gitlane [--project <PATH>] init [--name <NAME>] [--description <TEXT>] [--homepage <URL>] [--format <FORMAT>]
  ```

- Options:
    - `--name <NAME>`: set project name in the project config file created by `init`.
    - `--description <TEXT>`: set project description in the project config file created by `init`.
    - `--homepage <URL>`: set project homepage in the project config file created by `init`.
    - `--format <FORMAT>`: choose config file format for files created by `init`. Supported values: `toml`, `json`,
      `yaml`, `yml`. Default: `toml`.

### Behavior

- Target project root is created when missing.
- If a supported project config file already exists, `init` fails and leaves the existing project unchanged.
- If `.gitlane/` exists but no supported project config file exists, `init` treats it as a partial scaffold and creates
  the missing files and directories.
- `--name`, `--description`, and `--homepage` are used only when creating a new project config file.
- `--format` selects the extension and serialization used for any config files created during that `init` invocation.
  If omitted, `init` defaults to `toml`.
- If more than one supported config file exists for the same logical config (`project`, `workflow`, `issues`, or
  `labels`), Gitlane returns an error instead of guessing which file to use.
- Issue state directories are derived from `.gitlane/issues/workflow.<ext>`; `init` fails if the workflow config is invalid.

### Files and directories created when missing

```text
.gitlane/
  project.<ext>
  issues/
    workflow.<ext>
    issues.<ext>
    labels.<ext>
    <one directory per workflow state>
```

`<ext>` is `toml` by default, or the extension selected by `--format`.

For the default scaffolded workflow, `init` creates `todo/`, `in_progress/`, `review/`, and `done/`.

### Default bootstrap configs

For `--format toml` (the default), `init` creates the following config content.

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

When `--format json`, `--format yaml`, or `--format yml` is used, Gitlane writes the same logical config data as JSON
or YAML into `.json`, `.yaml`, or `.yml` files.

### Examples

```bash
gitlane init
gitlane init --format json
gitlane init --format yaml
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
