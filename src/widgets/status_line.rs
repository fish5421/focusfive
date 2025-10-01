use crate::ui::theme::FinancialTheme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct StatusLineWidget<'a> {
    text: Option<&'a str>,
    theme: &'a FinancialTheme,
}

impl<'a> StatusLineWidget<'a> {
    pub fn new(theme: &'a FinancialTheme) -> Self {
        Self {
            text: None,
            theme,
        }
    }

    pub fn text(mut self, text: &'a str) -> Self {
        self.text = Some(text);
        self
    }
}

impl<'a> Widget for StatusLineWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content = if let Some(text) = self.text {
            Line::from(vec![
                Span::styled("Selected: ", Style::default().fg(self.theme.text_secondary)),
                Span::styled(text, Style::default()
                    .fg(self.theme.text_primary)
                    .add_modifier(Modifier::BOLD)),
            ])
        } else {
            Line::from(Span::styled(
                "Use ↑/↓ or j/k to navigate metrics",
                Style::default().fg(self.theme.text_secondary),
            ))
        };

        let paragraph = Paragraph::new(content)
            .block(Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(self.theme.text_secondary)))
            .style(Style::default().bg(self.theme.bg_panel));

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widget_renders_with_no_text() {
        let theme = FinancialTheme::default();
        let widget = StatusLineWidget::new(&theme);

        // Test that widget can be rendered without panicking
        let area = Rect::new(0, 0, 80, 2);
        let mut buffer = Buffer::empty(area);
        widget.render(area, &mut buffer);

        // Basic smoke test
        assert!(!buffer.content.is_empty());
    }

    #[test]
    fn widget_renders_with_text() {
        let theme = FinancialTheme::default();
        let widget = StatusLineWidget::new(&theme)
            .text("Customer Satisfaction Score");

        // Test that widget can be rendered without panicking
        let area = Rect::new(0, 0, 80, 2);
        let mut buffer = Buffer::empty(area);
        widget.render(area, &mut buffer);

        // Basic smoke test
        assert!(!buffer.content.is_empty());
    }
}