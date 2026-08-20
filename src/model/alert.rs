use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Comparison {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl Comparison {
    pub fn matches(self, value: f64, threshold: f64) -> bool {
        match self {
            Self::GreaterThan => value > threshold,
            Self::GreaterThanOrEqual => value >= threshold,
            Self::LessThan => value < threshold,
            Self::LessThanOrEqual => value <= threshold,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    #[default]
    Warning,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuleMetric {
    Cpu,
    Memory,
    Swap,
    LoadAverage,
}

#[derive(Clone, Debug)]
pub struct AlertRule {
    pub name: String,
    pub metric: RuleMetric,
    pub comparison: Comparison,
    pub threshold: f64,
    pub duration: Duration,
    pub severity: AlertSeverity,
    pub cooldown: Duration,
    pub recovery_threshold: Option<f64>,
}
