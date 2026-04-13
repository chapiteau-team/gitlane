//! Clap definitions for the Gitlane command-line interface.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Top-level Gitlane CLI arguments.
#[derive(Debug, Parser)]
#[command(
    name = "gitlane",
    version,
    about = "Git-native task tracker",
    long_about = None
)]
pub struct Cli {
    /// Project directory or nested path used to locate `.gitlane`.
    #[arg(long, global = true, value_name = "PATH")]
    pub project: Option<PathBuf>,

    /// Command to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// Supported top-level Gitlane commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize a new Gitlane project scaffold.
    Init(InitArgs),
    /// Validate project configuration and data.
    Validate,
    /// Manage issues.
    Issue {
        /// Issue subcommand to execute.
        #[command(subcommand)]
        command: IssueCommand,
    },
    /// Inspect workflow configuration.
    Workflow {
        /// Workflow subcommand to execute.
        #[command(subcommand)]
        command: WorkflowCommand,
    },
    /// Inspect labels.
    Label {
        /// Label subcommand to execute.
        #[command(subcommand)]
        command: LabelCommand,
    },
}

/// Arguments for `gitlane init`.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Project name to write into the generated config.
    #[arg(long)]
    pub name: Option<String>,

    /// Optional project description for the generated config.
    #[arg(long)]
    pub description: Option<String>,

    /// Optional homepage URL for the generated config.
    #[arg(long)]
    pub homepage: Option<String>,

    /// Config file format to use for generated files.
    #[arg(long, value_enum)]
    pub format: Option<InitFormatArg>,
}

/// Supported values for `gitlane init --format`.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum InitFormatArg {
    /// Generate TOML config files.
    Toml,
    /// Generate JSON config files.
    Json,
    /// Generate YAML config files using `.yaml`.
    Yaml,
    /// Generate YAML config files using `.yml`.
    Yml,
}

/// Supported `gitlane issue` subcommands.
#[derive(Debug, Subcommand)]
pub enum IssueCommand {
    /// Create a new issue.
    Create,
    /// List issues.
    List,
    /// Show one issue by id.
    Show(IssueShowArgs),
    /// Apply a workflow transition to an issue.
    Transition(IssueTransitionArgs),
}

/// Arguments for `gitlane issue show`.
#[derive(Debug, Args)]
pub struct IssueShowArgs {
    /// Issue identifier to display.
    pub id: String,
}

/// Arguments for `gitlane issue transition`.
#[derive(Debug, Args)]
pub struct IssueTransitionArgs {
    /// Issue identifier to transition.
    pub id: String,
    /// Transition identifier to apply.
    pub transition_id: String,
}

/// Supported `gitlane workflow` subcommands.
#[derive(Debug, Subcommand)]
pub enum WorkflowCommand {
    /// Show the full workflow configuration.
    Show,
    /// List workflow states.
    States,
    /// List transitions, optionally filtered by source state.
    Transitions(WorkflowTransitionsArgs),
}

/// Arguments for `gitlane workflow transitions`.
#[derive(Debug, Args)]
pub struct WorkflowTransitionsArgs {
    /// Optional source state id to filter transitions.
    #[arg(long = "from", value_name = "STATE_ID")]
    pub from_state: Option<String>,
}

/// Supported `gitlane label` subcommands.
#[derive(Debug, Subcommand)]
pub enum LabelCommand {
    /// List labels.
    List,
    /// Show one label by id.
    Show(LabelShowArgs),
}

/// Arguments for `gitlane label show`.
#[derive(Debug, Args)]
pub struct LabelShowArgs {
    /// Label identifier to display.
    pub id: String,
}
