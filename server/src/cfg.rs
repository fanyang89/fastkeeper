use config::{Config, FileFormat, Map};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct NodeConfig {}

impl NodeConfig {
    pub fn load(id: u16, name: &str) -> Result<NodeConfig, config::ConfigError> {
        let settings = Config::builder()
            .add_source(config::File::with_name(name))
            .build()
            .unwrap();
        let s = settings.try_deserialize::<HashMap<String, String>>()?;
        Ok(NodeConfig {})
    }
}

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    path: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ClusterConfig {
    server: Map<String, ServerConfig>,
}

impl ClusterConfig {
    pub fn load(name: &str) -> Result<ClusterConfig, config::ConfigError> {
        let settings = Config::builder()
            .add_source(config::File::with_name(name))
            .build()
            .unwrap();
        let s = settings.try_deserialize::<HashMap<String, String>>()?;
        todo!()
    }

    pub fn load_str(config_str: &str) -> Result<ClusterConfig, config::ConfigError> {
        let settings = Config::builder()
            .add_source(config::File::from_str(config_str, FileFormat::Toml))
            .build()
            .unwrap();
        settings.try_deserialize::<ClusterConfig>()
    }
}

#[cfg(test)]
mod tests {
    use super::ClusterConfig;

    #[test]
    fn test_cluster_config_load() -> anyhow::Result<()> {
        let config = r#"[server]
            [server.1]
            path = "test-data/node1"

            [server.2]
            path = "test-data/node2"

            [server.3]
            path = "test-data/node3"
        "#;

        let c = ClusterConfig::load_str(&config)?;
        dbg!(&c);
        Ok(())
    }
}
