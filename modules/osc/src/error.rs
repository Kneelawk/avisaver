use searchlight::broadcast::errors::{BroadcasterBuilderError, ServiceBuilderError};
use searchlight::errors::{BadDnsNameError, ShutdownError};
use thiserror::Error;

/// Type describing all OSC startup errors.
#[derive(Debug, Error)]
pub enum OSCStartupError {
    #[error("IO error: {0:?}")]
    Io(#[from] std::io::Error),
    #[error("Bad DNS name error: {0:?}")]
    BadDnsName(#[from] BadDnsNameError),
    #[error("Error building service: {0:?}")]
    ServiceBuild(#[from] ServiceBuilderError),
    #[error("Error building broadcaster: {0:?}")]
    BroadcasterBuild(#[from] BroadcasterBuilderError),
}

/// Type describing all OSC shutdown errors.
#[derive(Debug, Error)]
pub enum OSCShutdownError {
    #[error("Error shutting down zeroconf broadcaster: {0:?}")]
    ZeroconfShutdown(#[from] ShutdownError),
}
