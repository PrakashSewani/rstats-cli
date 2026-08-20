use crate::app::AppState;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Row, Table},
};

pub fn render_alerts(frame: &mut Frame, area: Rect, state: &AppState) {
    let rows = state.alerts.iter().map(|alert| {
        Row::new(vec![
            Cell::from(alert.rule_name.clone()),
            Cell::from(format!("{:?}", alert.severity)),
            Cell::from(format!("{:.1}", alert.value)),
            Cell::from(format!("{:?}", alert.state)),
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Min(20),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
        ],
    )
    .block(Block::default().title("Alerts").borders(Borders::ALL));
    frame.render_widget(table, area);
}
