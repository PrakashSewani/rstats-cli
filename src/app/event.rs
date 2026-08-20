use crate::model::Snapshot;

#[derive(Debug)]
pub enum AppEvent {
    Snapshot(Box<Snapshot>),
    CollectorError(String),
}
