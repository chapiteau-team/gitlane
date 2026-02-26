# Issue Configuration

Issue is configured separately from workflow movement rules and labels.

## Configuration File

Issue configuration lives in:

`.gitlane/issues/issues.toml`

## Required top-level fields

- `issue_prefix`: prefix used in issue identifiers and filenames (for example, `ISSUE`).
- `priorities`: map of priority id to priority config.
- `priority_order`: ordered list of priority ids from highest to lowest.

## Priority schema

Each priority id is the key in `[priorities]`.

Each priority entry must contain:

- `name`

Each priority entry may contain:

- `description`

### Priority order semantics

- `priority_order` is required and defines display/semantic order.
- Earlier ids in `priority_order` are higher priority.
- Every id in `priority_order` must exist in `[priorities]`.
- Every `[priorities]` id must appear exactly once in `priority_order`.

### Popular priority example (p0-p4)

One common scheme is:

- `p0`: No Priority
- `p1`: Urgent
- `p2`: High
- `p3`: Medium
- `p4`: Low

In this scheme, `p1` is the highest priority and `p0` is the lowest priority.

## Example `.gitlane/issues/issues.toml`

```toml
issue_prefix = "ISSUE"

priority_order = ["p1", "p2", "p3", "p4", "p0"]

[priorities]
p0 = { name = "No Priority", description = "Default when urgency is not assigned" }
p1 = { name = "Urgent", description = "Needs immediate attention" }
p2 = { name = "High", description = "Important and should be scheduled soon" }
p3 = { name = "Medium", description = "Normal planned work" }
p4 = { name = "Low", description = "Can be deferred" }
```

## Related Configuration

- Labels and label groups are documented in `docs/labels.md`.
- Workflow states and transitions are documented in `docs/workflow.md`.
