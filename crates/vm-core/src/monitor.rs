use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("{0}")]
    Stream(#[from] std::io::Error),

    #[error("{0}")]
    CommandHandlerConflict(String),

    #[error("{0}")]
    Serde(#[from] serde_json::Error),

    #[error("unknown subcommand {0:?}")]
    UnknownSubcommand(Vec<String>),

    #[error("{0}")]
    Error(String),
}

#[async_trait]
pub trait MonitorCommandOps: Send + Sync {
    async fn handle_command(&self, subcommands: &[&str]) -> Result<String, MonitorError>;
}
