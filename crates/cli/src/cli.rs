use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "gitlane",
    version,
    about = "Git-native task tracker",
    long_about = None
)]
pub struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    pub project: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init(InitArgs),
    Validate,
    Issue {
        #[command(subcommand)]
        command: IssueCommand,
    },
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommand,
    },
    Label {
        #[command(subcommand)]
        command: LabelCommand,
    },
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long)]
    pub name: Option<String>,

    #[arg(long)]
    pub description: Option<String>,

    #[arg(long)]
    pub homepage: Option<String>,

    #[arg(long, value_enum)]
    pub format: Option<InitFormatArg>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum InitFormatArg {
    Toml,
    Yaml,
    Yml,
}

#[derive(Debug, Subcommand)]
pub enum IssueCommand {
    Create,
    List,
    Show(IssueShowArgs),
    Transition(IssueTransitionArgs),
}

#[derive(Debug, Args)]
pub struct IssueShowArgs {
    pub id: String,
}

#[derive(Debug, Args)]
pub struct IssueTransitionArgs {
    pub id: String,
    pub transition_id: String,
}

#[derive(Debug, Subcommand)]
pub enum WorkflowCommand {
    Show,
    States,
    Transitions(WorkflowTransitionsArgs),
}

#[derive(Debug, Args)]
pub struct WorkflowTransitionsArgs {
    #[arg(long = "from", value_name = "STATE_ID")]
    pub from_state: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum LabelCommand {
    List,
    Show(LabelShowArgs),
}

#[derive(Debug, Args)]
pub struct LabelShowArgs {
    pub id: String,
}
