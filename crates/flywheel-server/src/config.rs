use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub conveyor: ConveyorConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            conveyor: ConveyorConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

impl Config {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    #[serde(default = "default_metrics_address")]
    pub metrics_address: String,
}

fn default_bind_address() -> String {
    "0.0.0.0:50051".to_string()
}

fn default_metrics_address() -> String {
    "0.0.0.0:9090".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: default_bind_address(),
            metrics_address: default_metrics_address(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_url")]
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_db_url() -> String {
    "postgres://flywheel:flywheel@localhost/flywheel".to_string()
}

fn default_max_connections() -> u32 {
    20
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: default_db_url(),
            max_connections: default_max_connections(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConveyorConfig {
    pub router_endpoint: Option<String>,
}

impl Default for ConveyorConfig {
    fn default() -> Self {
        Self {
            router_endpoint: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub training_data_bucket: Option<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            training_data_bucket: None,
        }
    }
}
