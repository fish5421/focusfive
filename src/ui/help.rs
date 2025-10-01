use crate::ui::theme::FocusFiveTheme;
use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_help(f: &mut Frame, area: Rect, theme: &FocusFiveTheme) {
    let accent = Style::default().fg(theme.header);
    let lines = vec![
        Line::from(vec![
            Span::styled("j/k", accent),
            Span::raw(" Navigate  "),
            Span::styled("Space", accent),
            Span::raw(" Toggle  "),
            Span::styled("Enter", accent),
            Span::raw(" View/Expand  "),
            Span::styled("Esc", accent),
            Span::raw(" Close Popups"),
        ]),
        Line::from(vec![
            Span::styled("o", accent),
            Span::raw(" Objectives  "),
            Span::styled("i", accent),
            Span::raw(" Update Indicator  "),
            Span::styled("v", accent),
            Span::raw(" Vision  "),
            Span::styled("d", accent),
            Span::raw(" Dashboard  "),
            Span::styled("q", accent),
            Span::raw(" Quit"),
        ]),
    ];

    let help = Paragraph::new(lines)
        .style(Style::default().fg(theme.text_secondary))
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}

pub fn render_detailed_help(f: &mut Frame, area: Rect, theme: &FocusFiveTheme) {
    let help_lines = vec![
        Line::from(vec![Span::styled(
            "Keyboard Shortcuts",
            Style::default().fg(theme.header),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default().fg(theme.text_primary),
        )]),
        Line::from(vec![
            Span::styled("  j           ", Style::default().fg(theme.header)),
            Span::styled("Move down", Style::default().fg(theme.text_secondary)),
        ]),
        Line::from(vec![
            Span::styled("  k           ", Style::default().fg(theme.header)),
            Span::styled("Move up", Style::default().fg(theme.text_secondary)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions:",
            Style::default().fg(theme.text_primary),
        )]),
        Line::from(vec![
            Span::styled("  Space/Enter ", Style::default().fg(theme.header)),
            Span::styled(
                "Toggle action completion",
                Style::default().fg(theme.text_secondary),
            ),
        ]),
        Line::from(vec![
            Span::styled("  e           ", Style::default().fg(theme.header)),
            Span::styled(
                "Edit action text",
                Style::default().fg(theme.text_secondary),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Application:",
            Style::default().fg(theme.text_primary),
        )]),
        Line::from(vec![
            Span::styled("  q           ", Style::default().fg(theme.header)),
            Span::styled(
                "Quit application",
                Style::default().fg(theme.text_secondary),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Note: ", Style::default().fg(theme.partial)),
            Span::styled(
                "Changes are auto-saved",
                Style::default().fg(theme.text_secondary),
            ),
        ]),
    ];

    let help = Paragraph::new(help_lines)
        .style(Style::default())
        .alignment(Alignment::Left);

    f.render_widget(help, area);
}

pub fn get_context_help(focused_panel: &str) -> String {
    match focused_panel {
        "outcomes" => {
            "j/k: Select outcome | Space: View details".to_string()
        }
        "actions" => "j/k: Select action | Space: Toggle | e: Edit text"
            .to_string(),
        "editor" => "Type to edit | Enter: Save | Esc: Cancel".to_string(),
        _ => "j/k: Select | Space: Action | q: Quit".to_string(),
    }
}
