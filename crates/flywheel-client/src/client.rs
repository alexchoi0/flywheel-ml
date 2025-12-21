use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Request failed: {0}")]
    Request(String),
    #[error("Not found: {0}")]
    NotFound(String),
}

pub struct FlywheelClient {
    endpoint: String,
}

impl FlywheelClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    // TODO: Implement gRPC client methods
}
