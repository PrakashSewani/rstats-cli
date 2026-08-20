use crate::{app::AppState, tui::widgets::render_processes};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub fn render_process_screen(frame: &mut Frame, state: &AppState) {
    render_processes(frame, frame.size(), state);
    if !state.process_view.filter.is_empty() {
        let area = Rect { x: 2, y: 1, width: frame.size().width.saturating_sub(4), height: 3 };
        frame.render_widget(
            Paragraph::new(format!("Filter: {}", state.process_view.filter))
                .block(Block::default().borders(Borders::ALL)),
            area,
        );
    }
}
