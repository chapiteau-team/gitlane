# Project Configuration

Project metadata is defined at the repository level.

## Configuration File

Project configuration may live in exactly one of:

- `.gitlane/project.toml`
- `.gitlane/project.json`
- `.gitlane/project.yaml`
- `.gitlane/project.yml`

If more than one supported project config file is present at the same time, Gitlane returns an error because the
project config is ambiguous.

`gitlane init --format <FORMAT>` chooses which file is created for a new project config. Supported values are `toml`,
`json`, `yaml`, and `yml`. If `--format` is omitted, `gitlane init` defaults to `toml`.

## Required top-level fields

- `name`: human-readable project name. Must be a non-empty string.

## Optional top-level fields

- `description`: short project description.
- `homepage`: project homepage URL string (recommended to use an absolute `https://` URL).
- `people`: ordered list of person handles used in issue metadata (for example, `"@kalaninja"`).

### `people` semantics

- `people` is optional.
- Each entry must be a non-empty string handle.
- Entries must be unique.
- Order is preserved for deterministic display.

## Example minimal `.gitlane/project.toml`

This is the default format produced by `gitlane init` when `--format` is not specified.

```toml
name = "Gitlane"
```

## Example full `.gitlane/project.toml`

Equivalent `.json`, `.yaml`, and `.yml` files with the same fields are also valid.

```toml
name = "Gitlane"
description = "Git-native task tracker"
homepage = "https://github.com/example/gitlane"
people = ["@alice", "@bob", "@carol"]
```

## Related Configuration

- Workflow states and transitions are documented in [`docs/issues/workflow.md`](issues/workflow.md).
- Issue metadata is documented in [`docs/issues/issues.md`](issues/issues.md).
- Labels and label groups are documented in [`docs/issues/labels.md`](issues/labels.md).

## `init` command behavior

- `gitlane init` creates `.gitlane/project.<ext>` when the project config is missing.
- Default `name` is the target directory name.
- `--name`, `--description`, and `--homepage` can be used during `init` to set those fields when the file is created.
- `--format toml|json|yaml|yml` selects the config format used during `init`; the default is `toml`.
- If a supported project config file already exists, `gitlane init` returns an error and leaves the file unchanged.
- If multiple supported project config files exist, Gitlane returns an error instead of guessing which one to use.
