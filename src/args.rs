use clap::{
    Args,
    Parser,
    Subcommand,
};


#[derive(Debug, Parser)]
#[clap(name = "fsearch", version, author)]
pub struct FsearchArgs {
    #[clap(subcommand)]
    pub entity: Option<Entity>,

}


#[derive(Debug, Subcommand)]
pub enum Entity {
    /// Start the fsearch daemon
    Daemon,

    /// Get daemon status 
    Status,
    
    /// Stop the fsearch daemon
    Stop,

    /// Apply specific configuration to fsearch 
    Config(ConfigArgs),
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
