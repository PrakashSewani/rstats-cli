use ratatui::style::{Color, Modifier, Style};

pub fn title() -> Style {
    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
}
pub fn muted() -> Style {
    Style::default().fg(Color::DarkGray)
}
pub fn gauge() -> Style {
    Style::default().fg(Color::Green)
}
pub fn warning() -> Style {
    Style::default().fg(Color::Yellow)
}
pub fn critical() -> Style {
    Style::default().fg(Color::Red)
}
