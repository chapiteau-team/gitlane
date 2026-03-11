# Issues

Gitlane issue data is stored under `.gitlane/issues`.

## Issues Configuration

Issue configuration is separate from workflow movement rules and labels.

### Configuration File

Issue configuration lives in:

`.gitlane/issues/issues.toml`

### Required top-level fields

- `issue_prefix`: prefix used in issue identifiers and issue directory names (for example, `ISSUE`).
- `priorities`: map of priority id to priority config.
- `priority_order`: ordered list of priority ids from highest to lowest.

### Priority schema

Each priority id is the key in `[priorities]`.

Each priority entry must contain:

- `name`

Each priority entry may contain:

- `description`

#### Priority order semantics

- `priority_order` is required and defines display/semantic order.
- Earlier ids in `priority_order` are higher priority.
- Every id in `priority_order` must exist in `[priorities]`.
- Every `[priorities]` id must appear exactly once in `priority_order`.

#### Popular priority example (p0-p4)

One common scheme is:

- `p0`: No Priority
- `p1`: Urgent
- `p2`: High
- `p3`: Medium
- `p4`: Low

In this scheme, `p1` is the highest priority and `p0` is the lowest priority.

### Example `.gitlane/issues/issues.toml`

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

### Related Configuration

- Project metadata is documented in [`docs/project.md`](project.md).
- Labels and label groups are documented in [`docs/labels.md`](labels.md).
- Workflow states and transitions are documented in [`docs/workflow.md`](workflow.md).

## Issue Directory Structure

Each workflow state directory under `.gitlane/issues` contains issue directories.

Canonical issue directory name equals the issue id.

```text
<ID>/
  issue.md
  comments/        # optional
  attachments/     # optional
```

### Issue file naming

Always `issue.md`.

Reason: keeps tooling and validation trivial, and prevents "what is the main file?" ambiguity.

Path invariant:

`.gitlane/issues/<state>/<id>/issue.md`

### Issue ID

Issue ids must be unique across all workflow states.

Format:

`<prefix>-<base36(unix_ms)>`

Where:

- `<prefix>` is `issue_prefix` from `.gitlane/issues/issues.toml`.
- `unix_ms` is current UTC time in milliseconds.
- `base36(unix_ms)` is lowercase base36 encoding of `unix_ms`.

Collision strategy:

- If an issue directory with the candidate id already exists in any workflow state directory, increment `unix_ms` by 1 and re-encode.
- Repeat until a free issue directory name is found.

## Issue File Structure

`issue.md` is Markdown with required TOML front matter, followed by a user-defined Markdown body.

```markdown
+++
created_at = "2026-02-27T10:08:15Z"
updated_at = "2026-02-27T10:08:15Z"
reporter = "@kalaninja"
assignees = []
priority = "p2"
labels = ["docs"]
+++

Any Markdown content is valid here.
```

### Metadata schema

Required fields:

- `created_at`: RFC3339 UTC timestamp (`Z` suffix).
- `updated_at`: RFC3339 UTC timestamp (`Z` suffix).
- `reporter`: person reference string (for example, `@kalaninja`).
- `assignees`: array of person reference strings.
- `priority`: priority id from `.gitlane/issues/issues.toml`.
- `labels`: array of label ids from `.gitlane/issues/labels.toml`.

Semantics:

- `created_at` is immutable after issue creation.
- `reporter` is immutable after issue creation.
- `updated_at` must be greater than or equal to `created_at`.
- `assignees` must be unique.
- `labels` must be unique.
- Person reference format is repository-defined (for example, GitHub handle).
- Metadata key order has no semantic meaning.
- Unknown metadata keys are allowed for forward compatibility.

Path-derived fields:

- Issue id is derived from `<id>` in `.gitlane/issues/<state>/<id>/issue.md`.
- Workflow state is derived from `<state>` in `.gitlane/issues/<state>/<id>/issue.md`.
- `id` and `state` are not stored in issue front matter.

### Markdown body

- The Markdown body is unconstrained and fully user-defined.
- Tooling should treat body content as opaque unless explicitly editing it.

### History

- Issue history is tracked by Git commits.
- Issue files do not maintain an in-file change log.
