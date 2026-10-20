use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::{app::DeepCaptureIntent, tui::theme};

pub fn render_deep_capture_dialog(frame: &mut Frame, intent: DeepCaptureIntent) {
    let area = centered(68, 10, frame.size());
    frame.render_widget(Clear, area);
    let action = match intent {
        DeepCaptureIntent::Enable => "[y] enable deep capture   [n] cancel",
        DeepCaptureIntent::Start => "[y] start deep recording   [n] cancel",
    };
    let text = format!(
        "Deep capture records the full process table on every tick.\n\n\
         Recording files grow roughly 100x faster (~15 MB/min\n\
         versus ~0.1 MB/min at the default 1-second interval).\n\n\
         Best for short, deliberate diagnostic sessions.\n\n\
         {action}"
    );
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .title("Deep capture")
                    .borders(Borders::ALL)
                    .border_style(theme::warning()),
            )
            .style(theme::muted())
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn centered(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}
