use zookeeper::{
    client::{Client, Config, EventType, State},
    messages::proto::GetDataResponse,
};
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    hosts: Vec<String>,
}

fn on_event(event_type: EventType, state: State, path: String) {
    println!("type: {}, state: {}, path: {}", event_type, state, path);
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    tracing_subscriber::fmt::init();

    let mut conn = Client::new(&cli.hosts, Box::new(on_event), Config::default())?;
    let GetDataResponse { data, stat } = conn.get("/zookeeper/config", false).await?;
    println!("Config: {:?}, stat: {:?}", data, stat);

    Ok(())
}
