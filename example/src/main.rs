use clap::Parser;
use std::time::Duration;
use zookeeper::{
    client::{Client, Config},
    event,
    messages::proto::GetDataResponse,
};

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    hosts: Vec<String>,
}

fn on_event(event_type: event::Type, state: event::State, path: String) {
    println!("type: {}, state: {}, path: {}", event_type, state, path);
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    tracing_subscriber::fmt::init();
    let config = Config {
        connect_attempt: Some(Duration::from_secs(1)),
        ..Config::default()
    };
    let mut conn = Client::new(&cli.hosts, Box::new(on_event), config)?;
    let GetDataResponse { data, stat } = conn.get("/zookeeper/config", false).await?;
    println!("Config: {:?}, stat: {:?}", data, stat);

    Ok(())
}