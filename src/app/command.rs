use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
    Dashboard,
    Processes,
    Alerts,
    History,
}

#[derive(Debug)]
pub enum Command {
    Quit,
    Screen(Screen),
    Pause,
    ResetHistory,
    Help,
    Filter,
    SortCpu,
    SortMemory,
    ReverseSort,
    ToggleRecording,
    Up,
    Down,
    None,
}

pub fn command_for(key: KeyEvent) -> Command {
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Command::Quit;
    }
    match key.code {
        KeyCode::Char('q') => Command::Quit,
        KeyCode::Char('1') => Command::Screen(Screen::Dashboard),
        KeyCode::Char('2') => Command::Screen(Screen::Processes),
        KeyCode::Char('3') => Command::Screen(Screen::Alerts),
        KeyCode::Char('4') => Command::Screen(Screen::History),
        KeyCode::Char(' ') => Command::Pause,
        KeyCode::Char('R') => Command::ResetHistory,
        KeyCode::Char('?') => Command::Help,
        KeyCode::Char('/') => Command::Filter,
        KeyCode::Char('c') => Command::SortCpu,
        KeyCode::Char('m') => Command::SortMemory,
        KeyCode::Char('r') => Command::ReverseSort,
        KeyCode::Char('s') => Command::ToggleRecording,
        KeyCode::Up | KeyCode::Char('k') => Command::Up,
        KeyCode::Down | KeyCode::Char('j') => Command::Down,
        _ => Command::None,
    }
}
