use crate::model::{RuleMetric, Snapshot};

pub fn metric_value(metric: RuleMetric, snapshot: &Snapshot) -> Option<f64> {
    match metric {
        RuleMetric::Cpu => Some(snapshot.cpu.total_usage),
        RuleMetric::Memory => Some(snapshot.memory.used_percent),
        RuleMetric::Swap => Some(snapshot.swap.used_percent),
        RuleMetric::LoadAverage => snapshot.load_average,
    }
}
