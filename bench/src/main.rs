mod bench;
mod error;

use crate::bench::BenchOption;
use anyhow::Result;
use bytesize::ByteSize;
use clap::{Args, Parser, Subcommand};
use indicatif::ProgressBar;
use log::info;
use rand::RngCore;
use std::time::Duration;
use tokio::task::JoinHandle;
use zookeeper::ZooKeeper;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug, Clone)]
pub struct ConnectOption {
    /// ZooKeeper hosts
    hosts: String,

    /// Connection timeout
    #[arg(long, short = 't', value_parser = parse_duration, default_value = "10")]
    timeout: Duration,

    /// Auth digest
    #[arg(long, short)]
    digest: Option<String>,
}

impl ConnectOption {
    pub fn connect(&self) -> Result<ZooKeeper> {
        let zk = ZooKeeper::connect(
            self.hosts.as_str(),
            self.timeout.clone(),
            bench::LoggingWatcher,
        )?;
        if let Some(digest) = &self.digest {
            zk.add_auth("digest", digest.to_string().into_bytes())?;
        }
        Ok(zk)
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    Bench {
        #[command(flatten)]
        connection_option: ConnectOption,

        /// Number of total znodes
        #[arg(long, short = 'n', default_value_t = 1000)]
        iteration: u32,

        /// Number of threads
        #[arg(long, short = 'j', default_value_t = 8)]
        threads: u32,

        /// ZNode value size in bytes
        #[arg(long, short = 's', value_parser = parse_human_bytes, default_value = "128K")]
        node_size: usize,

        /// Create ephemeral znode or not
        #[arg(long, short, default_value_t = false)]
        ephemeral: bool,

        /// Test node prefix
        #[arg(long, short, default_value = "/zoobench/test-node")]
        prefix: String,
    },

    Clean {
        #[command(flatten)]
        connection_option: ConnectOption,

        /// Test node prefix
        #[arg(long, short, default_value = "/zoobench/test-node")]
        prefix: String,
    },

    GetLast {
        #[command(flatten)]
        connection_option: ConnectOption,

        #[arg(long, short, default_value = "/zoobench/test-node")]
        prefix: String,
    },

    DeleteChildren {
        #[command(flatten)]
        connection_option: ConnectOption,

        prefix: String,
    },
}

fn parse_human_bytes(arg: &str) -> Result<usize, String> {
    arg.parse::<ByteSize>().map(|x| x.as_u64() as usize)
}

fn parse_duration(arg: &str) -> Result<Duration, std::num::ParseIntError> {
    Ok(Duration::from_secs(arg.parse()?))
}

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info)?;
    let cli = Cli::parse();

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
                iteration: iteration,
                threads: threads,
                ephemeral: ephemeral,
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
