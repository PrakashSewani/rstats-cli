use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

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
    Select,
    Up,
    Down,
    None,
}

pub fn command_for(key: KeyEvent) -> Command {
    if key.kind != KeyEventKind::Press {
        return Command::None;
    }
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
        KeyCode::Enter => Command::Select,
        KeyCode::Up | KeyCode::Char('k') => Command::Up,
        KeyCode::Down | KeyCode::Char('j') => Command::Down,
        _ => Command::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_key_presses_trigger_recording_toggle() {
        assert!(matches!(
            command_for(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE)),
            Command::ToggleRecording
        ));
        assert!(matches!(
            command_for(KeyEvent::new_with_kind(
                KeyCode::Char('s'),
                KeyModifiers::NONE,
                KeyEventKind::Repeat,
            )),
            Command::None
        ));
        assert!(matches!(
            command_for(KeyEvent::new_with_kind(
                KeyCode::Char('s'),
                KeyModifiers::NONE,
                KeyEventKind::Release,
            )),
            Command::None
        ));
    }

    #[test]
    fn only_key_presses_trigger_navigation() {
        assert!(matches!(
            command_for(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Command::Down
        ));
        assert!(matches!(
            command_for(KeyEvent::new_with_kind(
                KeyCode::Down,
                KeyModifiers::NONE,
                KeyEventKind::Repeat,
            )),
            Command::None
        ));
        assert!(matches!(
            command_for(KeyEvent::new_with_kind(
                KeyCode::Down,
                KeyModifiers::NONE,
                KeyEventKind::Release,
            )),
            Command::None
        ));
    }
}
