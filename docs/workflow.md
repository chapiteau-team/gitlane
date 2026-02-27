# Workflow Definition

Gitlane workflow is fully declarative and not hardcoded.
It is defined at the repository level in `.gitlane/issues/workflow.toml`.

## Conceptual Model

A workflow is a directed graph:

- States are nodes.
- Transitions are directed edges.

## Workflow Configuration

Required top-level fields:

- `initial_state`: state id used for newly created issues.
- `states`: map of state id to state config.
- `transitions`: map of source state id to transition map.

### State schema

Each state id is the key in `[states]`.

Each state entry must contain:

- `name`: human-readable display name.

### Transition schema

Each source state id is the key in `[transitions]`.

Each transition id is then a key in `[transitions.<from_state_id>]`.

Each transition entry must contain:

- `name`: human-readable transition name.
- `to`: destination state id.

`initial_state`, transition source ids, and transition `to` values must reference state ids declared in `[states]`.

If a state has no outgoing transitions, it can be omitted from `[transitions]`.

### Example `.gitlane/issues/workflow.toml`

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

## Derived State Ordering

Workflow order is derived from transitions.

- Compute shortest directed distance from `initial_state` to every reachable state.
- Sort states by `(distance, state_id)` for deterministic output.
- Place unreachable states last, sorted by `state_id`.

## Filesystem Mapping

Every state defined in the workflow maps to a directory under `.gitlane/issues`.

Each state directory contains zero or more issue directories named by issue id.

Examples:

- `.gitlane/issues/todo`
- `.gitlane/issues/in_progress`
- `.gitlane/issues/review`
- `.gitlane/issues/done`

Workflow state equals the `<state>` segment in the issue path.

If an issue is stored at:

`.gitlane/issues/in_progress/ISSUE-m8x4gn8/issue.md`

Its state is `in_progress`.

## Transitioning State

Transitioning an issue from one state to another is done by moving the issue directory between state directories.

Example:

```bash
git mv \
  .gitlane/issues/in_progress/ISSUE-m8x4gn8 \
  .gitlane/issues/review/
```

The directory move is the state change.

## Related Configuration

Workflow does not define issue metadata.

- Issue config lives in `.gitlane/issues/issues.toml`.
- Label config lives in `.gitlane/issues/labels.toml`.

See [`docs/issues.md`](issues.md) and [`docs/labels.md`](labels.md) for issue metadata schemas.
