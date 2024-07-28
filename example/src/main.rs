use anyhow::Result;
use clap::Parser;
use tracing::{error, info};
use zookeeper::config::Config;
use zookeeper::{client::Client, event, messages::proto::GetDataResponse};

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    hosts: Vec<String>,
}

fn on_event(event_type: event::Type, state: event::State, path: String) {
    info!(
        "New event arrived, type: {}, state: {}, path: '{}'",
        event_type, state, path
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt::init();

    let config = Config::default();
    let mut conn = Client::new(&cli.hosts, &on_event, config)?;
    match conn.get("/zookeeper/config", false).await {
        Ok(GetDataResponse { data, .. }) => {
            let s = String::from_utf8(data)?;
            info!("config: \n{}", s);
        }
        Err(e) => error!("Get failed: {}", e),
    }

    Ok(())
}
