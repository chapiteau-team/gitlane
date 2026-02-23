# Workflow Definition

Gitlane workflow is fully declarative and not hardcoded.
It is defined at the repository level in `.gitlane/issues/workflow.toml`.

## Conceptual Model

A workflow is a directed graph:

- States are nodes.
- Transitions are directed edges.

### Required workflow fields

- `initial_state`: state id used for newly created issues.
- `issue_prefix`: prefix used in issue identifiers and filenames (for example, `ISSUE`).
- `states`: list of workflow states.
- `transitions`: list of allowed state transitions.

### State schema

Each state must contain:

- `id`: stable machine id.
- `name`: human-readable display name.

### Transition schema

Each transition must contain:

- `name`: human-readable transition name.
- `from`: source state id.
- `to`: destination state id.

## Configuration File

Workflow configuration lives in:

`.gitlane/issues/workflow.toml`

Example:

```toml
initial_state = "todo"
issue_prefix = "ISSUE"

[[states]]
id = "todo"
name = "To Do"

[[states]]
id = "in_progress"
name = "In Progress"

[[states]]
id = "review"
name = "In Review"

[[states]]
id = "done"
name = "Done"

[[transitions]]
name = "Start work"
from = "todo"
to = "in_progress"

[[transitions]]
name = "Request review"
from = "in_progress"
to = "review"

[[transitions]]
name = "Approve"
from = "review"
to = "done"
```

## Filesystem Mapping

Every state defined in the workflow maps to a directory under `.gitlane/issues`.

Examples:

- `.gitlane/issues/todo`
- `.gitlane/issues/in_progress`
- `.gitlane/issues/review`
- `.gitlane/issues/done`

Workflow state equals issue file path parent directory name.

If an issue is stored at:

`.gitlane/issues/in_progress/ISSUE-42.md`

Its state is `in_progress`.

## Transitioning State

Transitioning an issue from one state to another is done by moving the issue file between state directories.

Example:

```bash
git mv \
  .gitlane/issues/in_progress/ISSUE-42.md \
  .gitlane/issues/review/
```

This path move is the state change.
