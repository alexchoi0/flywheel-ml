pub mod drift;
pub mod export;
pub mod graph;
pub mod health;
pub mod logs;
pub mod model;
pub mod pipeline;
pub mod stats;
pub mod validate;

use flywheel_ml_client::FlywheelClient;

pub struct Context {
    pub server: String,
    pub namespace: String,
    pub verbose: bool,
}

impl Context {
    pub async fn client(&self) -> anyhow::Result<FlywheelClient> {
        let mut client = FlywheelClient::new(&self.server);
        client.connect().await.map_err(|e| anyhow::anyhow!("Failed to connect to server: {}", e))?;
        Ok(client)
    }
}
