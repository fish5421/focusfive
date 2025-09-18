use ratatui::{
    widgets::Paragraph,
    style::Style,
    text::{Line, Span},
    layout::{Alignment, Rect},
    Frame,
};
use crate::ui::theme::FocusFiveTheme;

pub fn render_help(f: &mut Frame, area: Rect, theme: &FocusFiveTheme) {
    let help_text = vec![
        Line::from(vec![
            Span::styled("[Tab]", Style::default().fg(theme.header)),
            Span::styled(" Switch panels  ", Style::default().fg(theme.text_secondary)),
            Span::styled("[j/k]", Style::default().fg(theme.header)),
            Span::styled(" Navigate  ", Style::default().fg(theme.text_secondary)),
            Span::styled("[Space]", Style::default().fg(theme.header)),
            Span::styled(" Toggle  ", Style::default().fg(theme.text_secondary)),
            Span::styled("[e]", Style::default().fg(theme.header)),
            Span::styled(" Edit  ", Style::default().fg(theme.text_secondary)),
            Span::styled("[q]", Style::default().fg(theme.header)),
            Span::styled(" Quit", Style::default().fg(theme.text_secondary)),
        ])
    ];

    let help = Paragraph::new(help_text)
        .style(Style::default())
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}

pub fn render_detailed_help(f: &mut Frame, area: Rect, theme: &FocusFiveTheme) {
    let help_lines = vec![
        Line::from(vec![
            Span::styled("Keyboard Shortcuts", Style::default().fg(theme.header)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation:", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  Tab         ", Style::default().fg(theme.header)),
            Span::styled("Switch between panels", Style::default().fg(theme.text_secondary)),
        ]),
        Line::from(vec![
            Span::styled("  j/↓         ", Style::default().fg(theme.header)),
            Span::styled("Move down", Style::default().fg(theme.text_secondary)),
        ]),
        Line::from(vec![
            Span::styled("  k/↑         ", Style::default().fg(theme.header)),
            Span::styled("Move up", Style::default().fg(theme.text_secondary)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Actions:", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  Space/Enter ", Style::default().fg(theme.header)),
            Span::styled("Toggle action completion", Style::default().fg(theme.text_secondary)),
        ]),
        Line::from(vec![
            Span::styled("  e           ", Style::default().fg(theme.header)),
            Span::styled("Edit action text", Style::default().fg(theme.text_secondary)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Application:", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  q           ", Style::default().fg(theme.header)),
            Span::styled("Quit application", Style::default().fg(theme.text_secondary)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Note: ", Style::default().fg(theme.partial)),
            Span::styled("Changes are auto-saved", Style::default().fg(theme.text_secondary)),
        ]),
    ];

    let help = Paragraph::new(help_lines)
        .style(Style::default())
        .alignment(Alignment::Left);

    f.render_widget(help, area);
}

pub fn get_context_help(focused_panel: &str) -> String {
    match focused_panel {
        "outcomes" => "Tab: Switch to actions | j/k: Select outcome | Space: View details".to_string(),
        "actions" => "Tab: Switch to outcomes | j/k: Select action | Space: Toggle | e: Edit text".to_string(),
        "editor" => "Type to edit | Enter: Save | Esc: Cancel".to_string(),
        _ => "Tab: Navigate | j/k: Select | Space: Action | q: Quit".to_string(),
    }
}