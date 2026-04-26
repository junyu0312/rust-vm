use thiserror::Error;

#[derive(Error, Debug)]
pub enum CpuError {
    #[error("Vcpu command channel disconnected")]
    VcpuCommandDisconnected,
}
