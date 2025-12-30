use std::path::PathBuf;

use clap::Parser;
use tracing::warn;

#[derive(Parser, Debug)]
pub struct Command {
    #[arg(short, long)]
    pub cpus: usize,

    #[arg(short, long)]
    pub memory: usize,

    #[arg(short, long)]
    pub kernel: PathBuf,

    #[arg(short, long)]
    pub cmdline: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("The number of cpus {current} exceeds maximum supported {max}")]
    CpuCapacityExceeded { current: usize, max: usize },
}

impl Command {
    pub fn validate(
        &self,
        cap_nr_vcpus: usize,
        cap_max_vcpus: usize,
        cap_max_vcpu_id: usize,
    ) -> Result<(), CommandError> {
        if self.cpus > cap_nr_vcpus {
            warn!(
                "The number of requested cpus {} exceeds the number of cpus recommended by KVM {}",
                self.cpus, cap_nr_vcpus
            );
        }

        if self.cpus > cap_max_vcpus {
            return Err(CommandError::CpuCapacityExceeded {
                current: self.cpus,
                max: cap_max_vcpus,
            });
        }

        if self.cpus > cap_max_vcpu_id {
            return Err(CommandError::CpuCapacityExceeded {
                current: self.cpus,
                max: cap_max_vcpu_id,
            });
        }

        Ok(())
    }
}
