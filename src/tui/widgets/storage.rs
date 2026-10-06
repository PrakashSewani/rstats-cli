use crate::{format::format_bytes, model::DiskSnapshot, tui::theme};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub fn render_storage(frame: &mut Frame, area: Rect, disks: &[DiskSnapshot]) {
    let block = Block::default().title("Storage").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if disks.is_empty() {
        frame.render_widget(Paragraph::new("No disks detected").style(theme::muted()), inner);
        return;
    }

    let ordered = ordered_disks(disks);
    let columns = storage_columns(area.width);
    let rows = inner.height as usize;
    let capacity = rows.saturating_mul(columns);
    let overflow = ordered.len() > capacity;
    let data_rows = if overflow { rows.saturating_sub(1) } else { rows };
    let visible = data_rows.saturating_mul(columns).min(ordered.len());
    let column_areas = if columns == 2 {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(inner)
    };

    for column in 0..columns {
        let lines = (0..data_rows)
            .filter_map(|row| ordered.get(row * columns + column))
            .map(|disk| disk_line(disk, column_areas[column].width as usize))
            .collect::<Vec<_>>();
        let mut lines = lines;
        if column == 0 && overflow {
            lines.push(Line::from(Span::styled(
                format!("+{} more disks", ordered.len().saturating_sub(visible)),
                theme::muted(),
            )));
        }
        frame.render_widget(Paragraph::new(lines), column_areas[column]);
    }
}

fn ordered_disks(disks: &[DiskSnapshot]) -> Vec<&DiskSnapshot> {
    let mut ordered = disks.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.mount_point.cmp(&right.mount_point).then_with(|| left.name.cmp(&right.name))
    });
    ordered
}

fn storage_columns(width: u16) -> usize {
    if width >= 80 {
        2
    } else {
        1
    }
}

fn disk_line(disk: &DiskSnapshot, width: usize) -> Line<'static> {
    let percent = normalized_percent(disk.used_percent);
    let capacity = format!(
        "{}/{}",
        format_bytes(disk.total_bytes.saturating_sub(disk.available_bytes)),
        format_bytes(disk.total_bytes),
    );
    let detail = format!("{percent:>5.1}% {capacity}");
    let label = disk_label(disk);
    let bar_width = if width >= 42 {
        10
    } else if width >= 28 {
        6
    } else {
        3
    };
    let label_width = width.saturating_sub(detail.chars().count() + bar_width + 4).min(18);
    let label = truncate_label(&label, label_width);
    let bar = usage_bar(percent, bar_width);
    let content = format!("{label:<label_width$} {bar} {detail}");
    let content = content.chars().take(width).collect::<String>();
    Line::from(Span::styled(content, disk_style(percent)))
}

fn disk_label(disk: &DiskSnapshot) -> String {
    if !disk.mount_point.is_empty() {
        disk.mount_point.clone()
    } else if !disk.name.is_empty() {
        disk.name.clone()
    } else {
        "disk".to_owned()
    }
}

fn truncate_label(label: &str, width: usize) -> String {
    if label.chars().count() <= width {
        return label.to_owned();
    }
    if width <= 1 {
        return label.chars().take(width).collect();
    }
    format!("{}…", label.chars().take(width - 1).collect::<String>())
}

fn usage_bar(percent: f64, width: usize) -> String {
    let filled = ((percent / 100.0) * width as f64).round() as usize;
    format!("{}{}", "█".repeat(filled.min(width)), "░".repeat(width.saturating_sub(filled)))
}

fn normalized_percent(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

fn disk_style(percent: f64) -> Style {
    if percent >= 90.0 {
        theme::critical()
    } else if percent >= 80.0 {
        theme::warning()
    } else {
        theme::gauge()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disk(mount_point: &str, name: &str, used_percent: f64) -> DiskSnapshot {
        DiskSnapshot {
            name: name.to_owned(),
            mount_point: mount_point.to_owned(),
            total_bytes: 1024 * 1024 * 1024,
            available_bytes: 512 * 1024 * 1024,
            used_percent,
        }
    }

    #[test]
    fn orders_disks_by_mount_point_then_name() {
        let disks = vec![disk("D:\\", "New Volume", 10.0), disk("C:\\", "", 20.0)];
        let ordered = ordered_disks(&disks);
        assert_eq!(ordered[0].mount_point, "C:\\");
        assert_eq!(ordered[1].mount_point, "D:\\");
    }

    #[test]
    fn uses_mount_point_and_name_fallbacks() {
        assert_eq!(disk_label(&disk("C:\\", "", 10.0)), "C:\\");
        assert_eq!(disk_label(&disk("", "New Volume", 10.0)), "New Volume");
        assert_eq!(disk_label(&disk("", "", 10.0)), "disk");
    }

    #[test]
    fn clamps_invalid_percentages_and_formats_capacity() {
        let line = disk_line(&disk("C:\\", "", f64::NAN), 60);
        let content = line.spans[0].content.to_string();
        assert!(content.contains("0.0%"));
        assert!(content.contains("512.0 MiB/1.0 GiB"));
    }

    #[test]
    fn truncates_long_labels_to_fit_the_indicator() {
        let line = disk_line(&disk("/a/very/long/mount/point", "", 42.0), 30);
        assert!(line.width() <= 30);
        assert!(line.spans[0].content.contains("42.0%"));
    }
}
