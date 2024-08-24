mod bench;
mod cli;
mod error;
mod zkdb;

use crate::bench::BenchOption;
use anyhow::Result;
use clap::Parser;
use cli::Commands;
use indicatif::ProgressBar;
use log::info;
use rand::RngCore;
use std::time::Duration;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info)?;
    let cli = cli::Cli::parse();

    match cli.command {
        Commands::Bench {
            iteration,
            threads,
            node_size,
            ephemeral,
            prefix,
            connection_option,
        } => {
            let mut buf = vec![0; node_size];
            rand::thread_rng().fill_bytes(&mut buf);
            let option = BenchOption {
                connect_option: connection_option.clone(),
                iteration,
                threads,
                ephemeral,
                node_value: buf,
                prefix: prefix.clone(),
            };
            let result = bench::bench(&option)?;
            info!("{}", result);
        }

        Commands::Clean {
            prefix,
            connection_option,
        } => {
            let pb = ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(100));
            pb.set_message(format!("Deleting {}", prefix));
            {
                let zk = connection_option.connect()?;
                bench::delete_prefix(&zk, &prefix)?;
            }
            pb.finish_and_clear();
        }

        Commands::GetLast {
            prefix,
            connection_option,
        } => {
            let prefix = bench::base_node(&prefix);
            let zk = connection_option.connect()?;
            let mut children: Vec<u64> = zk
                .get_children(prefix, false)?
                .iter()
                .map(|x| x.strip_prefix("test-node").unwrap_or(x))
                .map(|x| x.parse::<u64>().unwrap())
                .collect();
            info!("children len: {}", children.len());
            children.sort();
            if let Some(last) = children.last() {
                info!("last key: {}/{}", prefix, last);
            } else {
                info!("key: {} don't have children", prefix);
            }
        }

        Commands::DeleteChildren {
            connection_option,
            prefix,
        } => {
            const WORKER_COUNT: usize = 4;

            let zk = connection_option.connect()?;
            let children = zk.get_children(prefix.as_str(), false)?;

            let pb = ProgressBar::new(children.len() as u64);
            pb.enable_steady_tick(Duration::from_millis(100));

            let job_per_worker = children.len() / WORKER_COUNT;

            let workers: Vec<JoinHandle<()>> = (0..WORKER_COUNT)
                .map(|worker_index| {
                    let start = worker_index * job_per_worker;
                    let end = start + job_per_worker;
                    let children: Vec<String> = children[start..end]
                        .iter()
                        .map(|x| x.clone())
                        .collect::<Vec<_>>();
                    let prefix = prefix.clone();
                    let connection_option = connection_option.clone();
                    tokio::spawn(async move {
                        let zk = connection_option.connect().unwrap();
                        for child in children.iter() {
                            zk.delete(format!("{}/{}", prefix, child).as_str(), None)
                                .unwrap();
                        }
                    })
                })
                .collect();

            pb.finish_and_clear();
        }
    }

    Ok(())
}
