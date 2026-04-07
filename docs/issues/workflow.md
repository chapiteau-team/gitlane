# Workflow Definition

Gitlane workflow is fully declarative and not hardcoded.
It is defined at the repository level in exactly one of:

- `.gitlane/issues/workflow.toml`
- `.gitlane/issues/workflow.json`
- `.gitlane/issues/workflow.yaml`
- `.gitlane/issues/workflow.yml`

## Conceptual Model

A workflow is a directed graph:

- States are nodes.
- Transitions are directed edges.

## Workflow Configuration

If more than one supported workflow config file is present at the same time, Gitlane returns an error because the
workflow config is ambiguous.

`gitlane init --format <FORMAT>` chooses which workflow config file is created when one is missing. Supported values
are `toml`, `json`, `yaml`, and `yml`. If `--format` is omitted, `gitlane init` defaults to `toml`.

Required top-level fields:

- `initial_state`: state id used for newly created issues.
- `states`: map of state id to state config.
- `transitions`: map of source state id to transition map.

Workflow state ids must:

- Be non-empty.
- Not have leading or trailing whitespace.
- Be portable filesystem-safe path segments because they map directly to directories under `.gitlane/issues`.
- Not contain `/`, `\`, `<`, `>`, `:`, `"`, `|`, `?`, or `*`.
- Not be `.` or `..`.
- Not end with `.` or a space.
- Not use Windows reserved device names such as `CON`, `PRN`, `AUX`, `NUL`, `COM1`-`COM9`, or `LPT1`-`LPT9`.
- Not use the reserved name `templates`, because `.gitlane/issues/templates/` stores issue templates.

Transition ids must be non-empty and must not have leading or trailing whitespace.

`initial_state`, transition source ids, and transition `to` values must use valid workflow state ids.

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

This is the default format produced by `gitlane init` when `--format` is not specified.

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

`.gitlane/issues/in_progress/ISS-m8x4gn8/issue.md`

Its state is `in_progress`.

## Transitioning State

Transitioning an issue from one state to another is done by moving the issue directory between state directories.

Example:

```bash
git mv \
  .gitlane/issues/in_progress/ISS-m8x4gn8 \
  .gitlane/issues/review/
```

The directory move is the state change.

## Related Configuration

Workflow does not define issue metadata.

- Project metadata is documented in [`project.md`](../project.md).
- Issue config lives in exactly one of `.gitlane/issues/issues.toml`, `.gitlane/issues/issues.json`, `.gitlane/issues/issues.yaml`, or `.gitlane/issues/issues.yml`.
- Label config lives in exactly one of `.gitlane/issues/labels.toml`, `.gitlane/issues/labels.json`, `.gitlane/issues/labels.yaml`, or `.gitlane/issues/labels.yml`.

See [`issues.md`](issues.md) and [`labels.md`](labels.md) for issue metadata schemas.
