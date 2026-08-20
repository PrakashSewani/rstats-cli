use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fs, path::Path, time::Duration};

use crate::{
    cli::Cli,
    model::{AlertRule, AlertSeverity, Comparison, RuleMetric},
};

const DEFAULT_INTERVAL_MS: u64 = 1_000;
const DEFAULT_HISTORY_SECONDS: u64 = 300;
const MIN_INTERVAL_MS: u64 = 100;
const MAX_HISTORY_SAMPLES: usize = 3_600;

#[derive(Clone, Debug)]
pub struct Config {
    pub interval: Duration,
    pub history_capacity: usize,
    pub recording_directory: std::path::PathBuf,
    pub no_color: bool,
    pub bell: bool,
    pub alerts: Vec<AlertRule>,
}

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    interval_ms: Option<u64>,
    history_seconds: Option<u64>,
    no_color: Option<bool>,
    bell: Option<bool>,
    recording_dir: Option<std::path::PathBuf>,
    #[serde(default)]
    alerts: Vec<FileAlertRule>,
}

#[derive(Debug, Deserialize)]
struct FileAlertRule {
    name: String,
    metric: String,
    operator: Comparison,
    threshold: f64,
    #[serde(default)]
    duration_seconds: u64,
    #[serde(default)]
    severity: AlertSeverity,
    #[serde(default)]
    cooldown_seconds: u64,
    recovery_threshold: Option<f64>,
}

impl Config {
    pub fn from_cli(cli: &Cli) -> Result<Self> {
        let file = cli.config.as_deref().map(load_file).transpose()?.unwrap_or_default();
        let interval_ms = cli
            .interval_ms
            .or(file.interval_ms)
            .unwrap_or(DEFAULT_INTERVAL_MS)
            .max(MIN_INTERVAL_MS);
        let history_seconds =
            cli.history_seconds.or(file.history_seconds).unwrap_or(DEFAULT_HISTORY_SECONDS);
        let interval = Duration::from_millis(interval_ms);
        let history_capacity = history_capacity(history_seconds, interval);
        let alerts = if file.alerts.is_empty() {
            default_alerts()
        } else {
            file.alerts.into_iter().map(FileAlertRule::try_into_rule).collect::<Result<Vec<_>>>()?
        };

        Ok(Self {
            interval,
            history_capacity,
            recording_directory: cli
                .recording_dir
                .clone()
                .or(file.recording_dir)
                .unwrap_or_else(|| std::path::PathBuf::from("recordings")),
            no_color: cli.no_color || file.no_color.unwrap_or(false),
            bell: file.bell.unwrap_or(false),
            alerts,
        })
    }
}

fn load_file(path: &Path) -> Result<FileConfig> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    toml::from_str(&contents)
        .with_context(|| format!("failed to parse config file {}", path.display()))
}

fn history_capacity(history_seconds: u64, interval: Duration) -> usize {
    let interval_ms = interval.as_millis().max(1);
    let samples = (u128::from(history_seconds) * 1_000).div_ceil(interval_ms);
    samples.clamp(1, MAX_HISTORY_SAMPLES as u128) as usize
}

impl FileAlertRule {
    fn try_into_rule(self) -> Result<AlertRule> {
        let metric = match self.metric.as_str() {
            "cpu.total" => RuleMetric::Cpu,
            "memory.used_percent" => RuleMetric::Memory,
            "swap.used_percent" => RuleMetric::Swap,
            "load.average" => RuleMetric::LoadAverage,
            value => anyhow::bail!("unsupported alert metric: {value}"),
        };
        Ok(AlertRule {
            name: self.name,
            metric,
            comparison: self.operator,
            threshold: self.threshold,
            duration: Duration::from_secs(self.duration_seconds),
            severity: self.severity,
            cooldown: Duration::from_secs(self.cooldown_seconds),
            recovery_threshold: self.recovery_threshold,
        })
    }
}

fn default_alerts() -> Vec<AlertRule> {
    vec![
        AlertRule {
            name: "high_cpu".to_owned(),
            metric: RuleMetric::Cpu,
            comparison: Comparison::GreaterThan,
            threshold: 90.0,
            duration: Duration::from_secs(30),
            severity: AlertSeverity::Warning,
            cooldown: Duration::from_secs(300),
            recovery_threshold: Some(85.0),
        },
        AlertRule {
            name: "low_memory".to_owned(),
            metric: RuleMetric::Memory,
            comparison: Comparison::GreaterThan,
            threshold: 90.0,
            duration: Duration::from_secs(15),
            severity: AlertSeverity::Critical,
            cooldown: Duration::from_secs(300),
            recovery_threshold: Some(85.0),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_interval_and_history() {
        let cli = Cli {
            interval_ms: Some(1),
            history_seconds: Some(u64::MAX),
            config: None,
            recording_dir: None,
            no_color: false,
            log_level: None,
        };
        let config = Config::from_cli(&cli).unwrap();
        assert_eq!(config.interval, Duration::from_millis(MIN_INTERVAL_MS));
        assert_eq!(config.history_capacity, MAX_HISTORY_SAMPLES);
    }
}
