mod cli;

use clap::Parser;
use gitlane::Gitlane;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let repo_root = cli.repo_root.unwrap_or(std::env::current_dir()?);

    let _service = Gitlane::new(repo_root);
    Ok(())
}
