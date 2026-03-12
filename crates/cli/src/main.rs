mod cli;
mod path;

use std::ffi::OsStr;
use std::path::Path;

use anyhow::{anyhow, bail};
use clap::Parser;
use gitlane::{Gitlane, InitOptions, paths::GITLANE_DIR};

use crate::cli::Command;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let project_root = cli.project.unwrap_or(std::env::current_dir()?);

    match cli.command {
        Command::Init(args) => {
            let default_name = match args.name.as_ref() {
                Some(name) => name.clone(),
                None => infer_project_name(&project_root)?,
            };
            let options = InitOptions {
                name: args.name,
                default_name,
                description: args.description,
                homepage: args.homepage,
            };
            let project_path = project_root.join(GITLANE_DIR);

            let _service = Gitlane::init(project_path, options)?;
            Ok(())
        }
        _ => {
            let project_path = path::resolve_project(project_root)?;
            let _service = Gitlane::new(project_path)?;
            bail!("command not implemented")
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
