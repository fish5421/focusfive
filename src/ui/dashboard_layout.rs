use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct DashboardLayout {
    pub header: Rect,
    pub live_metrics: Rect,
    pub performance: Rect,
    pub sentiment: Rect,
    pub signals: Rect,
    pub status_line: Rect,
    pub footer: Rect,
}

impl DashboardLayout {
    pub fn new(area: Rect) -> Self {
        // Main vertical split
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Header
                Constraint::Min(20),   // Content
                Constraint::Length(2), // Status line
                Constraint::Length(2), // Footer
            ])
            .split(area);

        // Split content into two rows
        let content_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Top row
                Constraint::Percentage(50), // Bottom row
            ])
            .split(main_chunks[1]);

        // Top row: Live metrics and performance
        let top_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Live metrics - aligned with sentiment
                Constraint::Percentage(50), // Performance charts - aligned with signals
            ])
            .split(content_rows[0]);

        // Bottom row: Sentiment and signals
        let bottom_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Sentiment analysis - aligned with live metrics
                Constraint::Percentage(50), // Alternative signals - aligned with performance
            ])
            .split(content_rows[1]);

        Self {
            header: main_chunks[0],
            live_metrics: top_row[0],
            performance: top_row[1],
            sentiment: bottom_row[0],
            signals: bottom_row[1],
            status_line: main_chunks[2],
            footer: main_chunks[3],
        }
    }
}
