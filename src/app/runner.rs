use anyhow::Result;
use crossbeam_channel::{bounded, Receiver, Sender};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Terminal;
use std::{io::Stdout, thread, time::Duration};

use super::{
    command::{command_for, Command, Screen},
    event::AppEvent,
    state::AppState,
};
use crate::{
    collector::{Collector, SysinfoCollector},
    config::Config,
    tui::{
        screens::{
            render_alert_screen, render_dashboard, render_deep_capture_dialog,
            render_history_screen, render_process_screen,
        },
        terminal, theme,
    },
};

pub struct App;

impl App {
    pub fn run(config: Config) -> Result<()> {
        theme::set_theme(config.theme);
        let (sender, receiver) = bounded(4);
        let shutdown = start_sampler(config.interval, sender);
        let mut terminal = terminal::enter()?;
        let result = run_loop(&mut terminal, receiver, config);
        shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
        terminal::restore(terminal)?;
        result
    }
}

fn start_sampler(
    interval: Duration,
    sender: Sender<AppEvent>,
) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let thread_shutdown = shutdown.clone();
    thread::spawn(move || {
        let mut collector = SysinfoCollector::new();
        collector.warm_up();
        while !thread_shutdown.load(std::sync::atomic::Ordering::Relaxed) {
            match collector.collect() {
                Ok(snapshot) => {
                    if sender.send(AppEvent::Snapshot(Box::new(snapshot))).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    if sender.send(AppEvent::CollectorError(error.to_string())).is_err() {
                        break;
                    }
                }
            }
            thread::sleep(interval);
        }
    });
    shutdown
}

fn run_loop(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<Stdout>>,
    receiver: Receiver<AppEvent>,
    config: Config,
) -> Result<()> {
    let mut state = AppState::new(config);
    loop {
        while let Ok(event) = receiver.try_recv() {
            match event {
                AppEvent::Snapshot(snapshot) => state.apply_snapshot(*snapshot),
                AppEvent::CollectorError(error) => state.collector_error = Some(error),
            }
        }
        terminal.draw(|frame| {
            match state.screen {
                Screen::Dashboard => render_dashboard(frame, &state),
                Screen::Processes => render_process_screen(frame, &state),
                Screen::Alerts => render_alert_screen(frame, &state),
                Screen::History => render_history_screen(frame, &state),
            }
            if let Some(intent) = state.deep_capture_dialog {
                render_deep_capture_dialog(frame, intent);
            }
        })?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if matches!(handle_key(&mut state, key), Command::Quit) {
                    break;
                }
            }
        }
    }
    state.stop_recording()?;
    Ok(())
}

fn handle_key(state: &mut AppState, key: KeyEvent) -> Command {
    if state.deep_capture_dialog.is_some() {
        return handle_deep_capture_key(state, key);
    }
    let command = command_for(key);
    match command {
        Command::Quit => {}
        Command::Screen(screen) => {
            state.screen = screen;
            if screen == Screen::History {
                if let Err(error) = state.refresh_recordings() {
                    state.recording_error = Some(error.to_string());
                }
            }
        }
        Command::Pause => state.paused = !state.paused,
        Command::ResetHistory => state.reset_history(),
        Command::Help => state.help_visible = !state.help_visible,
        Command::CycleTheme => {
            theme::cycle_theme();
        }
        Command::SortCpu => {
            state.process_view.sort = crate::model::ProcessSort::Cpu;
        }
        Command::SortMemory => {
            state.process_view.sort = crate::model::ProcessSort::Memory;
        }
        Command::ReverseSort => {
            state.process_view.descending = !state.process_view.descending;
        }
        Command::ToggleRecording => {
            if let Err(error) = state.toggle_recording() {
                state.recording_error = Some(error.to_string());
            }
        }
        Command::ToggleScope => state.toggle_record_scope(),
        Command::Select => {
            if let Err(error) = state.load_selected_recording() {
                state.recording_error = Some(error.to_string());
            }
        }
        Command::Up => state.move_recording_selection(-1),
        Command::Down => state.move_recording_selection(1),
        _ => {}
    }
    command
}

fn handle_deep_capture_key(state: &mut AppState, key: KeyEvent) -> Command {
    if key.kind != KeyEventKind::Press {
        return Command::None;
    }
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Command::Quit;
    }
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => state.confirm_deep_capture(),
        KeyCode::Char('n') | KeyCode::Esc => state.cancel_deep_capture(),
        _ => {}
    }
    Command::None
}
