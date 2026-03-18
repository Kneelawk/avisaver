#[macro_use]
extern crate tracing;

pub mod error;
pub mod format;
pub mod queryserver;
pub mod zeroconf;

/// Options to describe which OSC parameters to accept
#[derive(Debug, Default, Clone)]
pub struct QueryOptions {
    pub app_name: String,
    /// A set of all the directories to advertise that we listen to.
    pub directories: Vec<String>,
}
