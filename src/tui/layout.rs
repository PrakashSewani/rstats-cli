use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct DashboardLayout {
    pub header: Rect,
    pub overview: Rect,
    pub storage: Option<Rect>,
    pub history: Rect,
    pub footer: Rect,
}

pub fn dashboard_chunks(area: Rect, disk_count: usize) -> DashboardLayout {
    if disk_count == 0 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(7),
                Constraint::Min(8),
                Constraint::Length(2),
            ])
            .split(area);
        return DashboardLayout {
            header: chunks[0],
            overview: chunks[1],
            storage: None,
            history: chunks[2],
            footer: chunks[3],
        };
    }

    let storage_height = storage_height(area, disk_count);
    let constraints = if storage_height == 0 {
        vec![
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Min(1),
            Constraint::Length(2),
        ]
    } else {
        vec![
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(storage_height),
            Constraint::Min(1),
            Constraint::Length(2),
        ]
    };
    let chunks =
        Layout::default().direction(Direction::Vertical).constraints(constraints).split(area);
    if storage_height == 0 {
        DashboardLayout {
            header: chunks[0],
            overview: chunks[1],
            storage: None,
            history: chunks[2],
            footer: chunks[3],
        }
    } else {
        DashboardLayout {
            header: chunks[0],
            overview: chunks[1],
            storage: Some(chunks[2]),
            history: chunks[3],
            footer: chunks[4],
        }
    }
}

pub fn overview_chunks(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area)
        .to_vec()
}

fn storage_height(area: Rect, disk_count: usize) -> u16 {
    let columns = if area.width >= 80 { 2 } else { 1 };
    let rows = disk_count.div_ceil(columns);
    let desired = rows.saturating_add(2) as u16;
    let available = area.height.saturating_sub(3 + 7 + 1 + 2);
    desired.min(available)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_hides_storage_without_disks() {
        let layout = dashboard_chunks(Rect::new(0, 0, 120, 24), 0);
        assert!(layout.storage.is_none());
        assert_eq!(layout.overview.height, 7);
    }

    #[test]
    fn dashboard_allocates_two_column_storage_for_wide_terminals() {
        let layout = dashboard_chunks(Rect::new(0, 0, 120, 24), 6);
        assert_eq!(layout.storage.unwrap().height, 5);
        assert!(layout.history.height >= 4);
    }

    #[test]
    fn dashboard_allocates_compact_storage_for_narrow_terminals() {
        let layout = dashboard_chunks(Rect::new(0, 0, 60, 24), 6);
        assert_eq!(layout.storage.unwrap().height, 8);
        assert!(layout.history.height >= 4);
    }
}
