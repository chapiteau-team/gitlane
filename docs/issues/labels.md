# Label Configuration

Labels are configured separately from workflow movement rules and from issue configuration.

## Configuration File

Label configuration may live in exactly one of:

- `.gitlane/issues/labels.toml`
- `.gitlane/issues/labels.json`
- `.gitlane/issues/labels.yaml`
- `.gitlane/issues/labels.yml`

If more than one supported labels config file is present at the same time, Gitlane returns an error because the labels
config is ambiguous.

`gitlane init --format <FORMAT>` chooses which labels config file is created when one is missing. Supported values are
`toml`, `json`, `yaml`, and `yml`. If `--format` is omitted, `gitlane init` defaults to `toml`.

## Required top-level fields

- `labels`: map of label id to label config.

## Optional top-level fields

- `label_groups`: map of label group id to label group config.

## Label Group Schema

Each label group id is the key in `[label_groups]`.

Each label group entry must contain:

- `name`

Each label group entry may contain:

- `description`
- `color`

## Label Schema

Each label id is the key in `[labels]`.

Each label entry must contain:

- `name`

Each label entry may contain:

- `description`
- `color`
- `group`: label group id.

If a label belongs to a group and does not define `color`, it inherits the group's `color`.

## Group Selection Semantics

- Labels in the same group are single-select per issue (at most one label from a given group).
- Labels without `group` are ungrouped and can be combined freely.
- `group` must reference an existing key in `[label_groups]`.

## Color Inheritance Semantics

- If `labels.<id>.color` is set, that value is used.
- Otherwise, if the label has `group` and `[label_groups].<group>.color` is set, the group color is inherited.
- Otherwise, the label has no configured color.

## Default init labels

When `gitlane init` is run without `--format`, it uses the following default `.gitlane/issues/labels.toml` when the
file is missing:

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

In this example, all `type_*` labels are mutually exclusive because they are in the same `type` group, and they all
inherit color `#334155` from that group.

When `gitlane init --format json`, `gitlane init --format yaml`, or `gitlane init --format yml` is used, Gitlane
writes the same logical label data as JSON or YAML into `.gitlane/issues/labels.json`, `.gitlane/issues/labels.yaml`,
or `.gitlane/issues/labels.yml`.
