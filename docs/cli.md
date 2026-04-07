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
    - `create [options]`: create a new issue in the workflow initial state.
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

Commands other than `init` are currently scaffolded. Detailed sections below may describe planned behavior for
commands that are not yet implemented.

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
- Issue state directories are derived from `.gitlane/issues/workflow.<ext>`; `init` fails if the workflow config is
  invalid.
- `init` also creates `.gitlane/issues/templates/default/issue.md` when it is missing. The default template front
  matter format matches the `--format` value used for that `init` invocation, and its placeholder metadata includes
  `title`, `assignees`, and `labels`.

### Files and directories created when missing

```text
.gitlane/
  project.<ext>
  issues/
    workflow.<ext>
    issues.<ext>
    labels.<ext>
    templates/
      default/
        issue.md
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

## `issue create`

- Purpose: create a new issue in the workflow initial state.
- Status: planned, not implemented.
- Usage:

  ```bash
  gitlane [--project <PATH>] issue create --title <TITLE> [--template <NAME>] [--templates-path <PATH>] [--reporter <PERSON>] [--assignee <PERSON>]... [--priority <ID>] [--label <ID>]... [--body <MARKDOWN>] [--body-file <PATH>]
  ```

- Options:
    - `--title <TITLE>`: required title stored in issue metadata.
    - `--template <NAME>`: copy the template directory `<root>/<NAME>/` into the new issue directory. Template names
      are single directory names only, not nested paths. If omitted, Gitlane uses `default`. The resolved template
      directory must contain `issue.md` at its root.
    - `--templates-path <PATH>`: override the template root directory used with `--template`. Relative paths resolve
      from the project root. The resolved template directory becomes `<PATH>/<NAME>/`, where `<NAME>` is the explicit
      `--template` value or `default` when `--template` is omitted.
    - `--reporter <PERSON>`: reporter stored in issue metadata. If omitted, Gitlane uses `git config user.name` and
      fails when that value is missing, blank, or invalid for the current project.
    - `--assignee <PERSON>`: add an assignee. Repeatable.
    - `--priority <ID>`: priority id from the issue config. If omitted, Gitlane uses the lowest-priority id, meaning
      the last entry in `priority_order`.
    - `--label <ID>`: add a label. Repeatable.
    - `--body <MARKDOWN>`: append inline Markdown to the created issue body.
    - `--body-file <PATH>`: append Markdown from a file to the created issue body.

### Behavior

- New issues are created in the workflow `initial_state`.
- New issue ids use the `<prefix>-<base36(unix_ms)>` scheme described in [`docs/issues/issues.md`](issues/issues.md).
- `issue create` always resolves a template directory before creating an issue. If `--template` is omitted, Gitlane
  uses `default`.
- `--title` is always required, even when `--template` is used.
- When `--template` is present, Gitlane resolves `<NAME>` under `.gitlane/issues/templates/` by default, or under the
  root provided by `--templates-path`.
- When `--template` is omitted, Gitlane resolves `default` under `.gitlane/issues/templates/` by default, or under the
  root provided by `--templates-path`.
- Template names cannot contain path separators. Use names like `bug-regression`, not `bug/regression`.
- Gitlane copies the entire resolved template directory into the new issue directory and uses the copied `issue.md` to
  determine the created issue front matter format.
- Explicit CLI flags override metadata values supplied by the template.
- When a value is not provided as a flag, Gitlane uses the template value when available; otherwise it falls back to
  creation defaults.
- Creation defaults are:
    - `reporter`: `git config user.name`
    - `priority`: the last id in `priority_order`
    - `assignees`: empty list
    - `labels`: empty list
- `--body-file` and `--body` append content after any template-provided body, in that order, with one blank line
  inserted between non-empty body segments.
- Labels remain subject to the label-group single-select rules described in [`docs/issues/labels.md`](issues/labels.md).

### Precedence

Issue creation resolves values in this order:

1. Resolved template contents.
2. Explicit CLI metadata overrides such as `--reporter`, `--priority`, `--assignee`, and `--label`.
3. Creation defaults for any remaining required metadata.
4. Body assembly in this order: template body, `--body-file`, `--body`.

### Errors

- Template names containing path separators are invalid.
- A resolved template directory that does not exist is an error.
- A resolved template directory without `issue.md` at its root is an error.
- If `--reporter` is omitted and `git config user.name` is missing or blank, `issue create` fails.
- If project `people` is configured and the resolved `reporter` is not in that list, `issue create` fails.
- If project `people` is configured and any `assignee` is not in that list, `issue create` fails.

### Examples

```bash
gitlane issue create --title "Document issue create"
gitlane issue create --title "Document issue create" --priority p2 --label type_docs --body "Capture the planned command behavior."
gitlane issue create --title "Triage release blocker" --template bug --label blocked --body-file ./notes/repro.md --body "Need release decision this week."
gitlane issue create --title "Design review" --template default --templates-path ./docs/issue-templates
```
