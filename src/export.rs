use anyhow::{bail, Context, Result};
use std::{
    fmt::Write as _,
    fs,
    io::{self, Write},
    path::Path,
    str::FromStr,
};

use crate::{
    format::{format_rate, format_uptime},
    recording::RecordedSession,
};

const CSV_HEADER: &str = "timestamp_ms,cpu_total_pct,memory_used_pct,swap_used_pct,load_average,net_received_bps,net_transmitted_bps,disk_max_used_pct";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExportFormat {
    #[default]
    Csv,
    Json,
}

impl FromStr for ExportFormat {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "csv" => Ok(ExportFormat::Csv),
            "json" => Ok(ExportFormat::Json),
            other => Err(format!("invalid format `{other}` (expected csv or json)")),
        }
    }
}

pub fn run_export(path: &Path, format: ExportFormat, output: Option<&Path>) -> Result<()> {
    let session = load_with_samples(path)?;
    let rendered = match format {
        ExportFormat::Csv => render_csv(&session),
        ExportFormat::Json => render_json(&session)?,
    };
    write_output(rendered.as_bytes(), output)
}

pub fn run_report(path: &Path) -> Result<()> {
    let session = load_with_samples(path)?;
    let mut rendered = render_report(&session);
    rendered.push('\n');
    write_output(rendered.as_bytes(), None)
}

pub fn render_csv(session: &RecordedSession) -> String {
    let mut rendered = String::from(CSV_HEADER);
    rendered.push('\n');
    let load_aligned = load_is_aligned(session);
    for index in 0..session.cpu.len() {
        let load = if load_aligned {
            format!("{:.2}", session.load_average[index])
        } else {
            String::new()
        };
        let _ = writeln!(
            rendered,
            "{},{:.2},{:.2},{:.2},{},{:.0},{:.0},{:.2}",
            session.timestamps[index],
            session.cpu[index],
            session.memory[index],
            session.swap[index],
            load,
            session.net_received[index],
            session.net_transmitted[index],
            session.disk_max[index],
        );
    }
    rendered
}

pub fn render_json(session: &RecordedSession) -> Result<String> {
    let load_aligned = load_is_aligned(session);
    let mut rows = Vec::with_capacity(session.cpu.len());
    for index in 0..session.cpu.len() {
        let load = if load_aligned { Some(session.load_average[index]) } else { None };
        rows.push(serde_json::json!({
            "timestamp_ms": session.timestamps[index],
            "cpu_total_pct": session.cpu[index],
            "memory_used_pct": session.memory[index],
            "swap_used_pct": session.swap[index],
            "load_average": load,
            "net_received_bps": session.net_received[index],
            "net_transmitted_bps": session.net_transmitted[index],
            "disk_max_used_pct": session.disk_max[index],
        }));
    }
    let mut rendered = serde_json::to_string_pretty(&rows)?;
    rendered.push('\n');
    Ok(rendered)
}

pub fn render_report(session: &RecordedSession) -> String {
    let mut lines = vec![
        "rstats recording report".to_owned(),
        format!("file        {}", session.path.display()),
        format!("samples     {}", session.cpu.len()),
        format!("duration    {}", format_uptime(session.duration_seconds())),
        String::new(),
    ];
    let mut section = Vec::new();
    section.extend(summary_line("cpu %", &session.cpu, &session.timestamps, |value| {
        format!("{value:>6.1}%")
    }));
    section.extend(summary_line("memory %", &session.memory, &session.timestamps, |value| {
        format!("{value:>6.1}%")
    }));
    section.extend(summary_line("swap %", &session.swap, &session.timestamps, |value| {
        format!("{value:>6.1}%")
    }));
    if load_is_aligned(session) {
        section.extend(summary_line("load", &session.load_average, &session.timestamps, |value| {
            format!("{value:>6.2}")
        }));
    }
    section.extend(summary_line("net rx", &session.net_received, &session.timestamps, format_rate));
    section.extend(summary_line(
        "net tx",
        &session.net_transmitted,
        &session.timestamps,
        format_rate,
    ));
    section.extend(summary_line("disk %", &session.disk_max, &session.timestamps, |value| {
        format!("{value:>6.1}%")
    }));
    lines.extend(section);
    lines.join("\n")
}

fn load_with_samples(path: &Path) -> Result<RecordedSession> {
    let session = RecordedSession::load(path)?;
    if session.cpu.is_empty() {
        bail!("recording has no samples: {}", path.display());
    }
    Ok(session)
}

fn load_is_aligned(session: &RecordedSession) -> bool {
    session.load_average.len() == session.cpu.len()
}

fn summary_line(
    label: &str,
    values: &[f64],
    timestamps: &[u64],
    format: impl Fn(f64) -> String,
) -> Option<String> {
    let average = average(values)?;
    let (index, peak) = peak(values)?;
    let start = timestamps.first().copied().unwrap_or_default();
    let offset = timestamps.get(index).copied().unwrap_or(start).saturating_sub(start) / 1_000;
    Some(format!(
        "{label:<8} avg {:>12}   peak {:>12}   (+{offset}s)",
        format(average),
        format(peak)
    ))
}

fn average(values: &[f64]) -> Option<f64> {
    let mut sum = 0.0;
    let mut count = 0usize;
    for value in values.iter().filter(|value| value.is_finite()) {
        sum += value;
        count += 1;
    }
    if count == 0 {
        None
    } else {
        Some(sum / count as f64)
    }
}

fn peak(values: &[f64]) -> Option<(usize, f64)> {
    values.iter().enumerate().filter(|(_, value)| value.is_finite()).fold(
        None,
        |best: Option<(usize, f64)>, (index, value)| {
            if best.map_or(true, |(_, best)| *value > best) {
                Some((index, *value))
            } else {
                best
            }
        },
    )
}

fn write_output(bytes: &[u8], output: Option<&Path>) -> Result<()> {
    if let Some(path) = output {
        fs::write(path, bytes)
            .with_context(|| format!("failed to write output file {}", path.display()))?;
        return Ok(());
    }
    let stdout = io::stdout();
    let mut out = stdout.lock();
    if let Err(error) = out.write_all(bytes).and_then(|()| out.flush()) {
        if error.kind() == io::ErrorKind::BrokenPipe {
            return Ok(());
        }
        return Err(error.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_export_formats() {
        assert_eq!("csv".parse::<ExportFormat>(), Ok(ExportFormat::Csv));
        assert_eq!("json".parse::<ExportFormat>(), Ok(ExportFormat::Json));
        assert!("xml".parse::<ExportFormat>().is_err());
    }

    #[test]
    fn averages_and_peaks_ignore_non_finite_values() {
        let values = [1.0, f64::NAN, 3.0, f64::INFINITY, 2.0];
        assert_eq!(average(&values), Some(2.0));
        assert_eq!(peak(&values), Some((2, 3.0)));
        assert_eq!(average(&[f64::NAN]), None);
        assert_eq!(peak(&[]), None);
    }
}
