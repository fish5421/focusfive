use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub header: Rect,
    pub outcomes: Rect,
    pub actions: Rect,
    pub stats: Rect,
    pub footer: Rect,
}

pub fn create_layout(area: Rect) -> AppLayout {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Content
            Constraint::Length(2), // Footer
        ])
        .split(area);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Outcomes
            Constraint::Percentage(45), // Actions
            Constraint::Percentage(30), // Stats
        ])
        .split(main_chunks[1]);

    AppLayout {
        header: main_chunks[0],
        outcomes: content_chunks[0],
        actions: content_chunks[1],
        stats: content_chunks[2],
        footer: main_chunks[2],
    }
}
