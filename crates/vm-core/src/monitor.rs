use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    #[error("{0}")]
    Stream(#[from] std::io::Error),

    #[error("{0}")]
    CommandHandlerConflicat(String),

    #[error("{0}")]
    Serde(#[from] serde_json::Error),

    #[error("Unknown cmd {0}")]
    UnknownCmd(String),

    #[error("unknown subcommand {0:?}")]
    UnknownSubcommand(Vec<String>),

    #[error("{0}")]
    Error(String),
}

#[async_trait]
pub trait MonitorCommand: Send + Sync {
    async fn handle_command(&self, subcommands: &[&str]) -> Result<String, MonitorError>;
}
