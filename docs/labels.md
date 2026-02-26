# Label Configuration

Labels are configured separately from workflow movement rules and from issue configuration.

## Configuration File

Label configuration lives in:

`.gitlane/issues/labels.toml`

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

## Example `.gitlane/issues/labels.toml`

```toml
[label_groups]
type = { name = "Type", description = "Issue classification", color = "#334155" }

[labels]
type_bug = { name = "Bug", description = "Unexpected behavior", group = "type" }
type_feature = { name = "Feature", description = "Net-new capability", group = "type" }
docs = { name = "Docs", description = "Documentation updates", color = "#0f766e" }
```

In this example, `type_bug` and `type_feature` are mutually exclusive because they are in the same `type` group, and
both inherit color `#334155` from that group.
