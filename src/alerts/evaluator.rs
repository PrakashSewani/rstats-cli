use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use super::rules::metric_value;
use crate::model::{AlertRule, AlertSeverity, Snapshot};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum AlertState {
    #[default]
    Inactive,
    Pending,
    Firing,
    Cooldown,
}

#[derive(Clone, Debug)]
pub struct AlertEvent {
    pub rule_name: String,
    pub severity: AlertSeverity,
    pub value: f64,
    pub threshold: f64,
    pub state: AlertState,
    pub timestamp: SystemTime,
}

#[derive(Clone, Debug, Default)]
struct RuleStatus {
    state: AlertState,
    condition_since: Option<SystemTime>,
    cooldown_since: Option<SystemTime>,
}

#[derive(Clone, Debug, Default)]
pub struct AlertEvaluator {
    statuses: HashMap<String, RuleStatus>,
}

impl AlertEvaluator {
    pub fn evaluate(&mut self, rules: &[AlertRule], snapshot: &Snapshot) -> Vec<AlertEvent> {
        rules.iter().filter_map(|rule| self.evaluate_rule(rule, snapshot)).collect()
    }

    pub fn state(&self, name: &str) -> AlertState {
        self.statuses.get(name).map(|status| status.state.clone()).unwrap_or_default()
    }

    fn evaluate_rule(&mut self, rule: &AlertRule, snapshot: &Snapshot) -> Option<AlertEvent> {
        let value = metric_value(rule.metric, snapshot)?;
        let now = snapshot.timestamp;
        let status = self.statuses.entry(rule.name.clone()).or_default();
        let recovery_threshold = rule.recovery_threshold.unwrap_or(rule.threshold);
        let condition = rule.comparison.matches(value, rule.threshold);
        let recovered = match rule.comparison {
            crate::model::Comparison::GreaterThan
            | crate::model::Comparison::GreaterThanOrEqual => value <= recovery_threshold,
            crate::model::Comparison::LessThan | crate::model::Comparison::LessThanOrEqual => {
                value >= recovery_threshold
            }
        };

        match status.state {
            AlertState::Inactive => {
                if condition {
                    status.condition_since = Some(now);
                    if rule.duration.is_zero() {
                        status.state = AlertState::Firing;
                    } else {
                        status.state = AlertState::Pending;
                    }
                }
            }
            AlertState::Pending => {
                if !condition {
                    status.state = AlertState::Inactive;
                    status.condition_since = None;
                } else if status
                    .condition_since
                    .is_some_and(|since| elapsed(since, now) >= rule.duration)
                {
                    status.state = AlertState::Firing;
                }
            }
            AlertState::Firing => {
                if recovered {
                    status.state = if rule.cooldown.is_zero() {
                        AlertState::Inactive
                    } else {
                        AlertState::Cooldown
                    };
                    status.cooldown_since = Some(now);
                }
            }
            AlertState::Cooldown => {
                if condition {
                    status.state = AlertState::Firing;
                    status.cooldown_since = None;
                } else if status
                    .cooldown_since
                    .is_some_and(|since| elapsed(since, now) >= rule.cooldown)
                {
                    status.state = AlertState::Inactive;
                    status.cooldown_since = None;
                    status.condition_since = None;
                }
            }
        }

        if matches!(status.state, AlertState::Pending | AlertState::Firing) {
            Some(AlertEvent {
                rule_name: rule.name.clone(),
                severity: rule.severity,
                value,
                threshold: rule.threshold,
                state: status.state.clone(),
                timestamp: now,
            })
        } else {
            None
        }
    }
}

fn elapsed(start: SystemTime, end: SystemTime) -> Duration {
    end.duration_since(start).unwrap_or_default()
}
