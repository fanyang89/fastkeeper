use clap::{Args, Parser, Subcommand};
use std::time::Duration;
use zookeeper::ZooKeeper;
use bytesize::ByteSize;
use crate::bench;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
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
    pub fn connect(&self) -> anyhow::Result<ZooKeeper> {
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
pub enum Commands {
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

fn parse_human_bytes(arg: &str) -> anyhow::Result<usize, String> {
    arg.parse::<ByteSize>().map(|x| x.as_u64() as usize)
}

fn parse_duration(arg: &str) -> anyhow::Result<Duration, std::num::ParseIntError> {
    Ok(Duration::from_secs(arg.parse()?))
}
