use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Debug, Parser)]
#[command(name = "fsearch", author, version, about)]
pub struct FsearchArgs {
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start the fsearch daemon
    Daemon,

    /// Get daemon status
    Status,

    /// Stop the fsearch daemon
    Stop,

    /// Apply specific configuration to fsearch
    Config(ConfigArgs),

    /// Generate command completion
    Completion(CompletionArgs),
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    /// The path of the config file .toml
    #[clap(short, long, value_name = "FILE_PATH")]
    pub config: Option<String>,

    /// The path of the css file
    #[clap(short, long, value_name = "FILE_PATH")]
    pub css: Option<String>,
}

#[derive(Debug, Args)]
pub struct CompletionArgs {
    /// The shell to generate the completion for
    #[clap(short, long, value_name = "SHELL")]
    pub shell: Shell,
}
