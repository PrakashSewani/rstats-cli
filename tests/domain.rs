use std::time::{Duration, SystemTime};

use rstats::{
    alerts::{AlertEvaluator, AlertState},
    model::{
        AlertRule, AlertSeverity, Comparison, CpuSnapshot, MemorySnapshot, ProcessSnapshot,
        ProcessSort, ProcessView, RuleMetric, Snapshot,
    },
};

fn snapshot(timestamp: SystemTime, cpu: f64) -> Snapshot {
    Snapshot {
        timestamp,
        cpu: CpuSnapshot { total_usage: cpu, per_core: Vec::new() },
        memory: MemorySnapshot {
            total_bytes: 100,
            used_bytes: 50,
            available_bytes: 50,
            used_percent: 50.0,
        },
        ..Snapshot::default()
    }
}

#[test]
fn process_view_filters_and_sorts_deterministically() {
    let processes = vec![
        ProcessSnapshot {
            pid: 2,
            name: "worker".into(),
            cpu_usage: 20.0,
            ..ProcessSnapshot::default()
        },
        ProcessSnapshot {
            pid: 1,
            name: "Worker helper".into(),
            cpu_usage: 20.0,
            ..ProcessSnapshot::default()
        },
        ProcessSnapshot {
            pid: 3,
            name: "shell".into(),
            cpu_usage: 80.0,
            ..ProcessSnapshot::default()
        },
    ];
    let view = ProcessView { sort: ProcessSort::Cpu, descending: true, filter: "worker".into() };
    let visible = view.visible(&processes);
    assert_eq!(visible.iter().map(|process| process.pid).collect::<Vec<_>>(), vec![1, 2]);
}

#[test]
fn alert_requires_sustained_condition() {
    let rule = AlertRule {
        name: "cpu".into(),
        metric: RuleMetric::Cpu,
        comparison: Comparison::GreaterThan,
        threshold: 80.0,
        duration: Duration::from_secs(10),
        severity: AlertSeverity::Warning,
        cooldown: Duration::ZERO,
        recovery_threshold: Some(70.0),
    };
    let start = SystemTime::UNIX_EPOCH;
    let mut evaluator = AlertEvaluator::default();
    assert_eq!(
        evaluator.evaluate(std::slice::from_ref(&rule), &snapshot(start, 90.0))[0].state,
        AlertState::Pending
    );
    assert_eq!(evaluator.state("cpu"), AlertState::Pending);
    assert_eq!(
        evaluator.evaluate(&[rule], &snapshot(start + Duration::from_secs(10), 90.0))[0].state,
        AlertState::Firing
    );
}

#[test]
fn alert_resolves_using_recovery_threshold() {
    let rule = AlertRule {
        name: "cpu".into(),
        metric: RuleMetric::Cpu,
        comparison: Comparison::GreaterThan,
        threshold: 80.0,
        duration: Duration::ZERO,
        severity: AlertSeverity::Critical,
        cooldown: Duration::ZERO,
        recovery_threshold: Some(70.0),
    };
    let start = SystemTime::UNIX_EPOCH;
    let mut evaluator = AlertEvaluator::default();
    evaluator.evaluate(std::slice::from_ref(&rule), &snapshot(start, 90.0));
    assert_eq!(evaluator.state("cpu"), AlertState::Firing);
    evaluator.evaluate(&[rule], &snapshot(start + Duration::from_secs(1), 70.0));
    assert_eq!(evaluator.state("cpu"), AlertState::Inactive);
}
