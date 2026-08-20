mod sysinfo_collector;

use thiserror::Error;

use crate::model::Snapshot;
pub use sysinfo_collector::SysinfoCollector;

#[derive(Debug, Error)]
pub enum CollectorError {
    #[error("system metrics collection failed: {0}")]
    Collection(String),
}

pub trait Collector: Send {
    fn warm_up(&mut self);
    fn collect(&mut self) -> Result<Snapshot, CollectorError>;
}
