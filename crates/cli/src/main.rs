mod cli;
mod path;

use clap::Parser;
use gitlane::Gitlane;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let project_path = cli.project.unwrap_or(std::env::current_dir()?);
    let project_path = path::resolve_project(project_path)?;

    let _service = Gitlane::new(project_path)?;
    Ok(())
}
