use thiserror::Error;

#[derive(Debug, Error)]
pub enum OscError {
    #[error("Zeroconf error: {0:?}")]
    Zeroconf(#[from] zeroconf::error::Error)
}
