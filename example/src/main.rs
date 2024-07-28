use clap::Parser;
use tracing::info;
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
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    tracing_subscriber::fmt::init();
    let config = Config::default();
    let mut conn = Client::new(&cli.hosts, &on_event, config)?;
    let GetDataResponse { data, stat } = conn.get("/zookeeper/config", false).await?;
    info!("Config: {:?}, stat: {:?}", data, stat);
    Ok(())
}
