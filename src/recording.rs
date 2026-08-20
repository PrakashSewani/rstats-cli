use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::model::Snapshot;

pub struct Recorder {
    path: PathBuf,
    writer: BufWriter<File>,
    sample_count: u64,
    started_at: SystemTime,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Record<'a> {
    Header { version: u8, started_at_ms: u64 },
    Sample { snapshot: &'a Snapshot },
    Footer { stopped_at_ms: u64, samples: u64 },
}

impl Recorder {
    pub fn start(directory: &Path) -> Result<Self> {
        create_dir_all(directory).with_context(|| {
            format!("failed to create recording directory {}", directory.display())
        })?;
        let started_at = SystemTime::now();
        let path = unique_path(directory, started_at);
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .with_context(|| format!("failed to create recording file {}", path.display()))?;
        let mut recorder = Self { path, writer: BufWriter::new(file), sample_count: 0, started_at };
        recorder.write_record(&Record::Header {
            version: 1,
            started_at_ms: timestamp_ms(started_at),
        })?;
        Ok(recorder)
    }

    pub fn record(&mut self, snapshot: &Snapshot) -> Result<()> {
        self.write_record(&Record::Sample { snapshot })?;
        self.sample_count += 1;
        Ok(())
    }

    pub fn finish(mut self) -> Result<RecordingSummary> {
        let stopped_at = SystemTime::now();
        self.write_record(&Record::Footer {
            stopped_at_ms: timestamp_ms(stopped_at),
            samples: self.sample_count,
        })?;
        self.writer.flush().context("failed to flush recording file")?;
        Ok(RecordingSummary {
            path: self.path,
            started_at: self.started_at,
            stopped_at,
            samples: self.sample_count,
        })
    }

    pub fn sample_count(&self) -> u64 {
        self.sample_count
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn write_record<T: Serialize>(&mut self, record: &T) -> Result<()> {
        serde_json::to_writer(&mut self.writer, record)
            .context("failed to serialize recording sample")?;
        self.writer.write_all(b"\n").context("failed to write recording newline")?;
        self.writer.flush().context("failed to flush recording sample")?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct RecordingSummary {
    pub path: PathBuf,
    pub started_at: SystemTime,
    pub stopped_at: SystemTime,
    pub samples: u64,
}

#[derive(Clone, Debug)]
pub struct RecordedSession {
    pub path: PathBuf,
    pub started_at: Option<SystemTime>,
    pub stopped_at: Option<SystemTime>,
    pub samples: u64,
    pub cpu: Vec<f64>,
    pub memory: Vec<f64>,
    pub swap: Vec<f64>,
    pub load_average: Vec<f64>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum StoredRecord {
    Header { started_at_ms: u64 },
    Sample { snapshot: Box<Snapshot> },
    Footer { stopped_at_ms: u64, samples: u64 },
}

impl RecordedSession {
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read recording file {}", path.display()))?;
        let mut session = Self {
            path: path.to_path_buf(),
            started_at: None,
            stopped_at: None,
            samples: 0,
            cpu: Vec::new(),
            memory: Vec::new(),
            swap: Vec::new(),
            load_average: Vec::new(),
        };
        for line in contents.lines() {
            let Ok(record) = serde_json::from_str::<StoredRecord>(line) else {
                continue;
            };
            match record {
                StoredRecord::Header { started_at_ms } => {
                    session.started_at = Some(from_timestamp_ms(started_at_ms));
                }
                StoredRecord::Sample { snapshot } => {
                    session.cpu.push(snapshot.cpu.total_usage);
                    session.memory.push(snapshot.memory.used_percent);
                    session.swap.push(snapshot.swap.used_percent);
                    if let Some(load) = snapshot.load_average {
                        session.load_average.push(load);
                    }
                    session.samples += 1;
                }
                StoredRecord::Footer { stopped_at_ms, samples } => {
                    session.stopped_at = Some(from_timestamp_ms(stopped_at_ms));
                    if session.samples == 0 {
                        session.samples = samples;
                    }
                }
            }
        }
        Ok(session)
    }

    pub fn duration_seconds(&self) -> u64 {
        match (self.started_at, self.stopped_at) {
            (Some(started), Some(stopped)) => {
                stopped.duration_since(started).unwrap_or_default().as_secs()
            }
            _ => 0,
        }
    }
}

pub fn list_recordings(directory: &Path) -> Result<Vec<RecordedSession>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut sessions = Vec::new();
    for entry in std::fs::read_dir(directory)
        .with_context(|| format!("failed to list recording directory {}", directory.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("jsonl") {
            continue;
        }
        if let Ok(session) = RecordedSession::load(&path) {
            sessions.push(session);
        }
    }
    sessions.sort_by(|left, right| right.path.cmp(&left.path));
    Ok(sessions)
}

impl RecordingSummary {
    pub fn duration_seconds(&self) -> u64 {
        self.stopped_at.duration_since(self.started_at).unwrap_or_default().as_secs()
    }
}

fn unique_path(directory: &Path, started_at: SystemTime) -> PathBuf {
    let stem = format!("rstats-{}", timestamp_ms(started_at));
    let first = directory.join(format!("{stem}.jsonl"));
    if !first.exists() {
        return first;
    }
    for suffix in 1.. {
        let candidate = directory.join(format!("{stem}-{suffix}.jsonl"));
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!()
}

fn timestamp_ms(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

fn from_timestamp_ms(milliseconds: u64) -> SystemTime {
    UNIX_EPOCH + std::time::Duration::from_millis(milliseconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use tempfile::tempdir;

    #[test]
    fn writes_header_samples_and_footer_as_jsonl() {
        let directory = tempdir().unwrap();
        let mut recorder = Recorder::start(directory.path()).unwrap();
        recorder.record(&Snapshot::default()).unwrap();
        let summary = recorder.finish().unwrap();
        let lines = std::fs::read_to_string(summary.path).unwrap();
        let records: Vec<Value> =
            lines.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0]["type"], "header");
        assert_eq!(records[1]["type"], "sample");
        assert_eq!(records[2]["type"], "footer");
        assert_eq!(records[2]["samples"], 1);
    }
}
