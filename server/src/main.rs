mod cfg;
mod cli;
mod storage;

use crate::cfg::{ClusterConfig, NodeConfig};
use crate::cli::Command;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = cli::Args::parse();
    match args.command {
        Command::Run { id, config } => {}
        Command::Local { config } => {}
    }
    Ok(())
}
