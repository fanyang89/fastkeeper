mod cfg;
mod cli;
mod storage;

use crate::cfg::{ClusterConfig, NodeConfig};
use crate::cli::Command;
use clap::Parser;

fn run_node(nc: &NodeConfig) {}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = cli::Args::parse();
    match args.command {
        Command::Run { id, config } => {
            let c = NodeConfig::load(id, config.as_str())?;
            run_node(&c);
        }
        Command::Local { config } => {
            let c = ClusterConfig::load(config.as_str())?;
            // for nc in c.nodes() {
            //     run_node(nc);
            // }
        }
    }
    Ok(())
}
