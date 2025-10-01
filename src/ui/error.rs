use crate::ui::theme::FocusFiveTheme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::time::Instant;

pub struct ErrorDisplay {
    message: Option<String>,
    level: ErrorLevel,
    shown_at: Option<Instant>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorLevel {
    Info,
    Warning,
    Error,
}

impl ErrorDisplay {
    pub fn new() -> Self {
        Self {
            message: None,
            level: ErrorLevel::Info,
            shown_at: None,
        }
    }

    pub fn show(&mut self, message: String, level: ErrorLevel) {
        self.message = Some(message);
        self.level = level;
        self.shown_at = Some(Instant::now());
    }

    pub fn show_info(&mut self, message: String) {
        self.show(message, ErrorLevel::Info);
    }

    pub fn show_warning(&mut self, message: String) {
        self.show(message, ErrorLevel::Warning);
    }

    pub fn show_error(&mut self, message: String) {
        self.show(message, ErrorLevel::Error);
    }

    pub fn clear(&mut self) {
        self.message = None;
        self.shown_at = None;
    }

    pub fn is_active(&self) -> bool {
        if let Some(shown_at) = self.shown_at {
            // Auto-hide after 3 seconds
            shown_at.elapsed().as_secs() < 3 && self.message.is_some()
        } else {
            false
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &FocusFiveTheme) {
        if !self.is_active() {
            return;
        }

        if let Some(ref msg) = self.message {
            // Create a centered popup area
            let popup_area = centered_rect(60, 20, area);

            // Clear the background for the popup
            f.render_widget(Clear, popup_area);

            let color = match self.level {
                ErrorLevel::Info => theme.completed,
                ErrorLevel::Warning => theme.partial,
                ErrorLevel::Error => theme.pending,
            };

            let title = match self.level {
                ErrorLevel::Info => " INFO ",
                ErrorLevel::Warning => " WARNING ",
                ErrorLevel::Error => " ERROR ",
            };

            let error_widget = Paragraph::new(msg.clone())
                .style(Style::default().fg(theme.text_primary))
                .block(
                    Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                        .style(Style::default().bg(theme.panel_bg)),
                )
                .alignment(Alignment::Center)
                .wrap(ratatui::widgets::Wrap { trim: true });

            f.render_widget(error_widget, popup_area);
        }
    }

    pub fn render_inline(&self, f: &mut Frame, area: Rect, theme: &FocusFiveTheme) {
        if !self.is_active() {
            return;
        }

        if let Some(ref msg) = self.message {
            let color = match self.level {
                ErrorLevel::Info => theme.completed,
                ErrorLevel::Warning => theme.partial,
                ErrorLevel::Error => theme.pending,
            };

            let prefix = match self.level {
                ErrorLevel::Info => "ℹ ",
                ErrorLevel::Warning => "⚠ ",
                ErrorLevel::Error => "✗ ",
            };

            let error_line = Line::from(vec![
                Span::styled(
                    prefix,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(msg.clone(), Style::default().fg(color)),
            ]);

            let error_widget = Paragraph::new(vec![error_line])
                .style(Style::default())
                .alignment(Alignment::Left);

            f.render_widget(error_widget, area);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl Default for ErrorDisplay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_lifecycle() {
        let mut display = ErrorDisplay::new();

        // Initially inactive
        assert!(!display.is_active());

        // Show a message
        display.show_info("Test message".to_string());
        assert!(display.is_active());
        assert_eq!(display.level, ErrorLevel::Info);

        // Clear the message
        display.clear();
        assert!(!display.is_active());
    }

    #[test]
    fn test_error_levels() {
        let mut display = ErrorDisplay::new();

        display.show_info("Info".to_string());
        assert_eq!(display.level, ErrorLevel::Info);

        display.show_warning("Warning".to_string());
        assert_eq!(display.level, ErrorLevel::Warning);

        display.show_error("Error".to_string());
        assert_eq!(display.level, ErrorLevel::Error);
    }
}
