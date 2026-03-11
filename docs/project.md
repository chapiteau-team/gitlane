# Project Configuration

Project metadata is defined at the repository level.

## Configuration File

Project configuration lives in:

`.gitlane/project.toml`

## Required top-level fields

- `name`: human-readable project name. Must be a non-empty string.

## Optional top-level fields

- `description`: short project description.
- `homepage`: project homepage URL string (recommended to use an absolute `https://` URL).

## Example minimal `.gitlane/project.toml`

```toml
name = "Gitlane"
```

## Example full `.gitlane/project.toml`

```toml
name = "Gitlane"
description = "Git-native task tracker"
homepage = "https://github.com/example/gitlane"
```

## Related Configuration

- Workflow states and transitions are documented in [`docs/issues/workflow.md`](issues/workflow.md).
- Issue metadata is documented in [`docs/issues/issues.md`](issues/issues.md).
- Labels and label groups are documented in [`docs/issues/labels.md`](issues/labels.md).
