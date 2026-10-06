use ratatui::{prelude::*, widgets::Sparkline};

pub fn render_sparklines(frame: &mut Frame, area: Rect, values: &[u64]) {
    frame.render_widget(Sparkline::default().data(values), area);
}

pub fn percent_values(values: &[f64]) -> Vec<u64> {
    values.iter().map(|value| value.clamp(0.0, 100.0) as u64).collect()
}

pub fn rate_values(received: &[f64], transmitted: &[f64]) -> Vec<u64> {
    received
        .iter()
        .zip(transmitted)
        .map(|(received, transmitted)| (received.max(0.0) + transmitted.max(0.0)) as u64)
        .collect()
}
