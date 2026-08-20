use ratatui::{prelude::*, widgets::Sparkline};

pub fn render_sparklines(frame: &mut Frame, area: Rect, values: &[u64]) {
    frame.render_widget(Sparkline::default().data(values), area);
}
