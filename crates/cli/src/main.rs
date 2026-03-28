mod cli;
mod path;

use std::ffi::OsStr;
use std::path::Path;

use anyhow::{anyhow, bail};
use clap::Parser;
use gitlane::{Gitlane, InitOptions, config::ConfigFileExtension, paths::GITLANE_DIR};

use crate::cli::{Command, InitFormatArg};

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let project_root = cli.project.unwrap_or(std::env::current_dir()?);

    match cli.command {
        Command::Init(args) => {
            let name = args
                .name
                .map_or_else(|| infer_project_name(&project_root), Ok)?;
            let format = args.format.map_or(ConfigFileExtension::Toml, Into::into);
            let options = InitOptions::new(name, args.description, args.homepage, format)?;
            let project_path = project_root.join(GITLANE_DIR);

            let _service = Gitlane::init(project_path, options)?;
            Ok(())
        }
        _ => {
            let project_path = path::resolve_project(&project_root)?;
            let _service = Gitlane::load(project_path)?;
            bail!("command not implemented")
        }
    }
}

impl From<InitFormatArg> for ConfigFileExtension {
    fn from(format: InitFormatArg) -> Self {
        match format {
            InitFormatArg::Toml => Self::Toml,
            InitFormatArg::Json => Self::Json,
            InitFormatArg::Yaml => Self::Yaml,
            InitFormatArg::Yml => Self::Yml,
        }
    }
}

fn infer_project_name(project_root: &Path) -> anyhow::Result<String> {
    project_root
        .file_name()
        .and_then(OsStr::to_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            anyhow!(
                "failed to infer project name from `{}`; pass `--name`",
                project_root.display()
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    #[test]
    fn infers_project_name_from_directory_name() {
        let project_root = PathBuf::from("/tmp/example-project");

        let name = infer_project_name(&project_root).expect("project name should be inferred");

        assert_eq!(name, "example-project");
    }

    #[test]
    fn errors_when_project_name_cannot_be_inferred() {
        let err = infer_project_name(Path::new("/")).expect_err("root path should fail");

        assert!(
            err.to_string()
                .contains("failed to infer project name from `/`; pass `--name`")
        );
    }
}
