# Issues

Gitlane issue data is stored under `.gitlane/issues`.

## Issues Configuration

Issue configuration is separate from workflow movement rules and labels.

### Configuration File

Issue configuration may live in exactly one of:

- `.gitlane/issues/issues.toml`
- `.gitlane/issues/issues.json`
- `.gitlane/issues/issues.yaml`
- `.gitlane/issues/issues.yml`

If more than one supported issue config file is present at the same time, Gitlane returns an error because the issue
config is ambiguous.

`gitlane init --format <FORMAT>` chooses which issue config file is created when one is missing. Supported values are
`toml`, `json`, `yaml`, and `yml`. If `--format` is omitted, `gitlane init` defaults to `toml`.

### Required top-level fields

- `issue_prefix`: prefix used in issue identifiers and issue directory names (for example, `ISS`).
- `priorities`: map of priority id to priority config.
- `priority_order`: ordered list of priority ids from highest to lowest.

`issue_prefix` must:

- Be non-empty.
- Not have leading or trailing whitespace.
- Be a portable filesystem-safe path segment because issue ids become directory names.
- Not contain `/`, `\`, `<`, `>`, `:`, `"`, `|`, `?`, or `*`.
- Not be `.` or `..`.
- Not end with `.` or a space.
- Not use Windows reserved device names such as `CON`, `PRN`, `AUX`, `NUL`, `COM1`-`COM9`, or `LPT1`-`LPT9`.

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

This is the default format produced by `gitlane init` when `--format` is not specified.

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

### Related Configuration

- Project metadata is documented in [`project.md`](../project.md).
- Labels and label groups are documented in [`labels.md`](labels.md).
- Workflow states and transitions are documented in [`workflow.md`](workflow.md).

## Issue Templates

Issue templates are directories that Gitlane can copy into a new issue directory before applying metadata overrides and
creation defaults.

By default, issue templates live under `.gitlane/issues/templates`.

`gitlane init` creates `.gitlane/issues/templates/default/issue.md` when it is missing. The default template front
matter format matches the `gitlane init --format` value, and its placeholder metadata includes `title`, `assignees`,
and `labels`.

`gitlane issue create --template <NAME>` resolves templates by name, not by nested path.

If `--template` is omitted, `gitlane issue create` resolves the template name `default`.

- Template names must be single directory names and must not contain path separators.
- `bug-regression` is a valid template name.
- `bug/regression` is not a valid template name.
- `--templates-path <PATH>` overrides the template root for a single `gitlane issue create` invocation. Relative paths
  resolve from the project root.

Template directory layout:

```text
templates/
  default/
    issue.md
  <name>/
    issue.md
    ...
```

Rules:

- Each template directory must contain `issue.md` at its root.
- `gitlane issue create` always resolves a template directory and copies the entire resolved template directory into the
  new issue directory.
- Extra files and directories in the template are copied as-is.
- The copied `issue.md` is then updated with CLI-provided metadata overrides, creation defaults, and appended body
  content.
- Template `issue.md` files may provide reusable front matter values and body scaffolding. The final created issue must
  still satisfy the full issue metadata schema described below.
- When templates are stored under `.gitlane/issues/templates`, links such as `../../templates/bug/some.svg` remain
  valid from both the template `issue.md` and the generated issue `issue.md`.
- When `--templates-path` points elsewhere, users are responsible for making any referenced paths work.

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

- `<prefix>` is `issue_prefix` from the issue config file in `.gitlane/issues/issues.toml`, `.gitlane/issues/issues.json`, `.gitlane/issues/issues.yaml`, or `.gitlane/issues/issues.yml`.
- `unix_ms` is current UTC time in milliseconds.
- `base36(unix_ms)` is lowercase base36 encoding of `unix_ms`.

Collision strategy:

- If an issue directory with the candidate id already exists in any workflow state directory, increment `unix_ms` by 1
  and re-encode.
- Repeat until a free issue directory name is found.

## Issue File Structure

`issue.md` is Markdown with required front matter, followed by a user-defined Markdown body.

Front matter format is chosen per issue:

- `gitlane issue create` always preserves the front matter format from the resolved template `issue.md`.
- If `--template` is omitted, the resolved template is `default`.

### Front matter delimiters

- TOML front matter uses opening and closing `+++` fences.
- YAML and YML front matter use opening and closing `---` fences.
- JSON front matter uses a top-level object delimited by `{` and `}` at the start of the file.

```markdown
+++
title = "Document issue create behavior"
created_at = "2026-02-27T10:08:15Z"
updated_at = "2026-02-27T10:08:15Z"
reporter = "@kalaninja"
assignees = []
priority = "p2"
labels = ["type_docs"]
+++

Any Markdown content is valid here.
```

### Metadata schema

Required fields:

- `title`: non-empty issue title string.
- `created_at`: RFC3339 UTC timestamp (`Z` suffix).
- `updated_at`: RFC3339 UTC timestamp (`Z` suffix).
- `reporter`: person reference string (for example, `@kalaninja`).
- `assignees`: array of person reference strings.
- `priority`: priority id from the issue config file.
- `labels`: array of label ids from the labels config file.

Semantics:

- `title` must be non-empty.
- `created_at` is immutable after issue creation.
- `reporter` is immutable after issue creation.
- `updated_at` must be greater than or equal to `created_at`.
- `assignees` must be unique.
- `labels` must be unique.
- If `people` exists in the project config file, `reporter` and every `assignees` value must be present in that list.
- If `people` is omitted in the project config file, any non-empty person reference string is allowed.
- Person reference format is repository-defined (for example, `@kalaninja`).
- Metadata key order has no semantic meaning.
- Unknown metadata keys are allowed for forward compatibility.

### Creation defaults

- `title` must always be provided to `gitlane issue create`, even when a template is used.
- If `reporter` is omitted during `gitlane issue create`, Gitlane uses `git config user.name`.
- If `git config user.name` is missing or blank, `gitlane issue create` fails.
- If project `people` is configured, the resolved `reporter` must be present in that list or `gitlane issue create`
  fails.
- If `priority` is omitted during `gitlane issue create`, Gitlane uses the last id in `priority_order`.
- If `assignees` or `labels` are omitted during `gitlane issue create`, Gitlane stores empty arrays unless a template
  supplies values.
- Missing metadata values may be taken from the resolved template before creation defaults are applied.

Path-derived fields:

- Issue id is derived from `<id>` in `.gitlane/issues/<state>/<id>/issue.md`.
- Workflow state is derived from `<state>` in `.gitlane/issues/<state>/<id>/issue.md`.
- `id` and `state` are not stored in issue front matter.

### Markdown body

- The Markdown body is unconstrained and fully user-defined.
- Tooling should treat body content as opaque unless explicitly editing it.
- During `gitlane issue create`, any template-provided body comes first, followed by `--body-file`, then `--body`,
  with one blank line inserted between non-empty segments.

### History

- Issue history is tracked by Git commits.
- Issue files do not maintain an in-file change log.
