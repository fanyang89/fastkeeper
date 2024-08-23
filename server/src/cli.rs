use clap;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start service
    Run {
        /// Node id
        #[arg(long, short)]
        id: u16,

        /// Config file path
        #[arg(long, short)]
        config: String,
    },
    /// Start local cluster
    Local {
        /// Config file path
        #[arg(long, short)]
        config: String,
    },
}
