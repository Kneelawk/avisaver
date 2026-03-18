use thiserror::Error;

/// Type describing all possible OSC errors.
#[derive(Debug, Error)]
pub enum OscError {
    #[error("IO error: {0:?}")]
    Io(#[from] std::io::Error),
    #[error("Zeroconf error: {0:?}")]
    Zeroconf(#[from] zeroconf::error::Error),
}
