use std::path::PathBuf;

use clap::Parser;
use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum Accel {
    #[cfg(feature = "kvm")]
    Kvm,
    #[cfg(feature = "hvp")]
    Hvp,
}

#[derive(Debug, Parser)]
pub struct Command {
    #[arg(short, long)]
    pub cpus: usize,

    #[arg(short, long)]
    pub memory: usize,

    #[arg(short, long)]
    pub accel: Accel,

    #[arg(short, long)]
    pub kernel: PathBuf,

    #[arg(short, long)]
    pub cmdline: Option<String>,

    #[arg(short, long)]
    pub initramfs: Option<PathBuf>,
}
