use std::collections::HashMap;

use super::command::Screen;
use crate::{
    alerts::{AlertEvaluator, AlertEvent},
    config::Config,
    history::History,
    model::{MetricKind, ProcessSort, ProcessView, Snapshot},
    recording::{list_recordings, RecordedSession, Recorder, RecordingSummary},
};

const MAX_RECORDING_SAMPLES: usize = 86_400;

pub struct AppState {
    pub config: Config,
    pub screen: Screen,
    pub snapshot: Snapshot,
    pub histories: HashMap<MetricKind, History>,
    pub recording_histories: HashMap<MetricKind, History>,
    pub alerts: Vec<AlertEvent>,
    pub alert_evaluator: AlertEvaluator,
    pub process_view: ProcessView,
    pub paused: bool,
    pub help_visible: bool,
    pub collector_error: Option<String>,
    pub recording_error: Option<String>,
    pub recorder: Option<Recorder>,
    pub last_recording: Option<RecordingSummary>,
    pub saved_recordings: Vec<RecordedSession>,
    pub selected_recording: usize,
    pub loaded_recording: Option<RecordedSession>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let mut histories = HashMap::new();
        let mut recording_histories = HashMap::new();
        for kind in [MetricKind::Cpu, MetricKind::Memory, MetricKind::Swap, MetricKind::LoadAverage]
        {
            histories.insert(kind, History::new(config.history_capacity));
            recording_histories.insert(kind, History::new(MAX_RECORDING_SAMPLES));
        }
        Self {
            config,
            screen: Screen::Dashboard,
            snapshot: Snapshot::default(),
            histories,
            recording_histories,
            alerts: Vec::new(),
            alert_evaluator: AlertEvaluator::default(),
            process_view: ProcessView {
                sort: ProcessSort::Cpu,
                descending: true,
                filter: String::new(),
            },
            paused: false,
            help_visible: false,
            collector_error: None,
            recording_error: None,
            recorder: None,
            last_recording: None,
            saved_recordings: Vec::new(),
            selected_recording: 0,
            loaded_recording: None,
        }
    }

    pub fn apply_snapshot(&mut self, snapshot: Snapshot) {
        if self.paused {
            return;
        }
        self.snapshot = snapshot;
        self.history_push(MetricKind::Cpu, self.snapshot.cpu.total_usage);
        self.history_push(MetricKind::Memory, self.snapshot.memory.used_percent);
        self.history_push(MetricKind::Swap, self.snapshot.swap.used_percent);
        if let Some(load) = self.snapshot.load_average {
            self.history_push(MetricKind::LoadAverage, load);
        }
        if let Some(recorder) = self.recorder.as_mut() {
            if let Err(error) = recorder.record(&self.snapshot) {
                self.recording_error = Some(error.to_string());
                self.recorder = None;
            } else {
                self.recording_history_push(MetricKind::Cpu, self.snapshot.cpu.total_usage);
                self.recording_history_push(MetricKind::Memory, self.snapshot.memory.used_percent);
                self.recording_history_push(MetricKind::Swap, self.snapshot.swap.used_percent);
                if let Some(load) = self.snapshot.load_average {
                    self.recording_history_push(MetricKind::LoadAverage, load);
                }
            }
        }
        self.alerts = self.alert_evaluator.evaluate(&self.config.alerts, &self.snapshot);
        self.collector_error = None;
    }

    pub fn refresh_recordings(&mut self) -> anyhow::Result<()> {
        self.saved_recordings = list_recordings(&self.config.recording_directory)?;
        if self.saved_recordings.is_empty() {
            self.selected_recording = 0;
        } else {
            self.selected_recording = self.selected_recording.min(self.saved_recordings.len() - 1);
        }
        Ok(())
    }

    pub fn move_recording_selection(&mut self, offset: isize) {
        if self.saved_recordings.is_empty() {
            return;
        }
        let length = self.saved_recordings.len() as isize;
        let selected_recording =
            (self.selected_recording as isize + offset).rem_euclid(length) as usize;
        if selected_recording != self.selected_recording {
            self.loaded_recording = None;
        }
        self.selected_recording = selected_recording;
    }

    pub fn load_selected_recording(&mut self) -> anyhow::Result<()> {
        let Some(session) = self.saved_recordings.get(self.selected_recording) else {
            return Ok(());
        };
        self.loaded_recording = Some(RecordedSession::load(&session.path)?);
        Ok(())
    }

    pub fn toggle_recording(&mut self) -> anyhow::Result<()> {
        if self.recorder.is_some() {
            self.stop_recording()?;
        } else {
            self.start_recording()?;
        }
        self.recording_error = None;
        Ok(())
    }

    pub fn stop_recording(&mut self) -> anyhow::Result<Option<RecordingSummary>> {
        let Some(recorder) = self.recorder.take() else {
            return Ok(None);
        };
        let summary = recorder.finish()?;
        self.last_recording = Some(summary.clone());
        self.refresh_recordings()?;
        Ok(Some(summary))
    }

    pub fn recording_samples(&self) -> u64 {
        self.recorder.as_ref().map_or(0, Recorder::sample_count)
    }

    pub fn recording_action(&self) -> &'static str {
        if self.recorder.is_some() {
            "s stop recording"
        } else {
            "s start recording"
        }
    }

    fn start_recording(&mut self) -> anyhow::Result<()> {
        let recorder = Recorder::start(&self.config.recording_directory)?;
        self.recording_histories.values_mut().for_each(History::clear);
        self.last_recording = None;
        self.loaded_recording = None;
        self.recorder = Some(recorder);
        Ok(())
    }

    fn history_push(&mut self, kind: MetricKind, value: f64) {
        if let Some(history) = self.histories.get_mut(&kind) {
            history.push(value);
        }
    }

    fn recording_history_push(&mut self, kind: MetricKind, value: f64) {
        if let Some(history) = self.recording_histories.get_mut(&kind) {
            history.push(value);
        }
    }

    pub fn reset_history(&mut self) {
        self.histories.values_mut().for_each(History::clear);
    }
}
