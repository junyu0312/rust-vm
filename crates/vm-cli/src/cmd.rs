use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;

pub mod device;
pub mod json;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Json {
        #[arg(long)]
        path: PathBuf,
    },

    Snapshot {
        #[arg(long)]
        path: PathBuf,
    },
}
