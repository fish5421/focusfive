use crate::app::{App, InputMode, Pane};
use crate::models::OutcomeType;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Get the platform-appropriate modifier key name
fn get_modifier_key() -> &'static str {
    if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    }
}

pub fn render_app(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.size());

    render_header(f, chunks[0], app);

    // Show messages if present
    if let Some(ref error_msg) = app.error_message {
        render_error(f, chunks[1], error_msg);
    } else if let Some(ref info_msg) = app.info_message {
        render_info(f, chunks[1], info_msg);
    } else if app.show_help {
        render_help(f, chunks[1]);
    } else if matches!(app.input_mode, InputMode::GoalEditing { .. }) {
        render_goal_editor(f, chunks[1], app);
    } else if matches!(app.input_mode, InputMode::VisionEditing { .. }) {
        render_vision_editor(f, chunks[1], app);
    } else if matches!(app.input_mode, InputMode::CopyingFromYesterday { .. }) {
        render_yesterday_selector(f, chunks[1], app);
    } else if matches!(app.input_mode, InputMode::TemplateSelection { .. }) {
        render_template_selector(f, chunks[1], app);
    } else if matches!(app.input_mode, InputMode::TemplateSaving { .. }) {
        render_template_saving(f, chunks[1], app);
    } else if matches!(app.input_mode, InputMode::Reflecting { .. }) {
        render_reflection_modal(f, chunks[1], app);
    } else {
        // Render phase-specific content
        match app.ritual_phase {
            crate::models::RitualPhase::Evening if app.completion_stats.is_some() => {
                // Split area for progress gauge and main content
                let evening_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(4), // Progress gauge
                        Constraint::Min(0),    // Main content
                    ])
                    .split(chunks[1]);

                if let Some(ref stats) = app.completion_stats {
                    render_progress_gauge(f, evening_chunks[0], stats);
                }
                render_main_content(f, evening_chunks[1], app);
            }
            _ => {
                render_main_content(f, chunks[1], app);
            }
        }
    }

    render_footer(f, chunks[2], app);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let date_str = app.goals.date.format("%B %d, %Y").to_string();
    let day_str = if let Some(day) = app.goals.day_number {
        format!(" - Day {}", day)
    } else {
        String::new()
    };
    let total_actions = app.goals.work.actions.len() + 
                        app.goals.health.actions.len() + 
                        app.goals.family.actions.len();
    let progress = format!("Progress: {}/{}", app.total_completed(), total_actions);
    let streak = format!("🔥 {} day streak", app.current_streak);

    // Get phase-specific colors
    let primary_color = match app.ritual_phase {
        crate::models::RitualPhase::Morning => Color::Yellow,
        crate::models::RitualPhase::Evening => Color::Blue,
        crate::models::RitualPhase::None => Color::Cyan,
    };

    let accent_color = match app.ritual_phase {
        crate::models::RitualPhase::Morning => Color::Green,
        crate::models::RitualPhase::Evening => Color::Magenta,
        crate::models::RitualPhase::None => Color::Gray,
    };

    // Build header with phase greeting
    let mut spans = vec![
        Span::styled(
            app.ritual_phase.greeting(),
            Style::default()
                .fg(primary_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::raw(&date_str),
        Span::raw(&day_str),
        Span::raw(" | "),
        Span::styled(&progress, Style::default().fg(accent_color)),
        Span::raw(" | "),
        Span::styled(&streak, Style::default().fg(Color::Yellow)),
    ];

    // Add phase indicator if not in None phase
    if app.ritual_phase != crate::models::RitualPhase::None {
        let phase_indicator = match app.ritual_phase {
            crate::models::RitualPhase::Morning => " ☀️",
            crate::models::RitualPhase::Evening => " 🌙",
            _ => "",
        };
        spans.push(Span::styled(phase_indicator, Style::default()));
    }

    let header_text = vec![Line::from(spans)];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Center);

    f.render_widget(header, area);
}

fn render_main_content(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_outcomes_pane(f, chunks[0], app);
    render_actions_pane(f, chunks[1], app);
}

fn render_outcomes_pane(f: &mut Frame, area: Rect, app: &App) {
    let outcomes = app.goals.outcomes();

    let items: Vec<ListItem> = outcomes
        .iter()
        .enumerate()
        .map(|(i, outcome)| {
            let completed = app.outcome_completed(outcome);
            let total = outcome.actions.len();
            let icon = if completed == total { "✓" } else { " " };
            let progress = format!("[{}/{}]", completed, total);

            let style = if app.active_pane == Pane::Outcomes && i == app.outcome_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = format!(
                "{} {} {} - {}",
                icon,
                outcome.outcome_type.as_str(),
                progress,
                outcome
                    .goal
                    .as_ref()
                    .unwrap_or(&String::from("No goal set"))
            );

            ListItem::new(content).style(style)
        })
        .collect();

    let border_style = if app.active_pane == Pane::Outcomes {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let outcomes_list = List::new(items).block(
        Block::default()
            .title(" Outcomes ")
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    f.render_widget(outcomes_list, area);
}

fn render_actions_pane(f: &mut Frame, area: Rect, app: &App) {
    let selected_outcome = app.get_selected_outcome();

    // Check if we're editing the current action
    let is_editing = matches!(app.input_mode, InputMode::Editing { .. });

    let items: Vec<ListItem> = selected_outcome
        .actions
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let checkbox = if action.completed { "[x]" } else { "[ ]" };

            // Check if this action is being edited
            let editing_this_action =
                is_editing && app.active_pane == Pane::Actions && i == app.action_index;

            let style = if editing_this_action {
                // Blue background for edit mode
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else if app.active_pane == Pane::Actions && i == app.action_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else if action.completed {
                Style::default().fg(Color::Green)
            } else if action.text.is_empty() {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            let text = if editing_this_action {
                // Show the edit buffer with cursor
                if let InputMode::Editing { ref buffer, .. } = app.input_mode {
                    format!("{}│", buffer)
                } else {
                    action.text.clone()
                }
            } else if action.text.is_empty() {
                "(empty)".to_string()
            } else {
                action.text.clone()
            };

            ListItem::new(format!("{} {}", checkbox, text)).style(style)
        })
        .collect();

    let border_style = if app.active_pane == Pane::Actions {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let title = format!(" {} Actions ", selected_outcome.outcome_type.as_str());

    let actions_list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    f.render_widget(actions_list, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "FocusFive Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from("  Tab       - Switch between Outcomes and Actions panes"),
        Line::from("  j/↓       - Move down in list"),
        Line::from("  k/↑       - Move up in list"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from("  Space     - Toggle checkbox (in Actions pane)"),
        Line::from("  e/Enter   - Edit action text (in Actions pane)"),
        Line::from("  a         - Add new action (max 5 per outcome, in Actions pane)"),
        Line::from("  d         - Delete action (double-press to confirm, min 1 per outcome)"),
        Line::from("  v         - Edit 5-year vision (in Outcomes pane)"),
        Line::from("  g         - Edit goal for outcome (in Outcomes pane)"),
        Line::from("  y         - Copy actions from yesterday"),
        Line::from("  t         - Apply action template (in Actions pane)"),
        Line::from("  T         - Save current actions as template (in Actions pane)"),
        Line::from("  m         - Switch to Morning ritual phase"),
        Line::from("  n         - Switch to Evening (night) ritual phase"),
        Line::from("  s         - Save changes"),
        Line::from("  q         - Save and quit"),
        Line::from("  ?         - Show/hide this help"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Edit Mode:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from("  Type      - Add text"),
        Line::from("  Backspace - Delete character"),
        Line::from("  Enter     - Save changes"),
        Line::from("  Ctrl+S    - Save (alternative)"),
        Line::from("  Ctrl+Enter/Cmd+Enter - Save (alternative)"),
        Line::from("  Esc       - Cancel editing"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Vision Edit Mode:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from("  Type       - Add text"),
        Line::from("  Enter      - New line"),
        Line::from(format!("  {}+Enter - Save vision", get_modifier_key())),
        Line::from("  Ctrl+S     - Save vision (alternative)"),
        Line::from("  F2         - Save vision (universal)"),
        Line::from("  Esc        - Cancel editing"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Morning Phase (Intention Setting):",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from("  y          - Quick-fill incomplete actions from yesterday"),
        Line::from("  1-9        - Apply templates (dynamic based on template count)"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Evening Phase (Reflection & Review):",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from("  1-9, a-f   - Quick complete actions (dynamic based on total count)"),
        Line::from("  r          - Add reflection for current outcome"),
        Line::from("  d          - Generate daily summary"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Variable Actions (NEW!):",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from("  • Each outcome can have 1-5 actions (flexible from fixed 3)"),
        Line::from("  • Use 'a' to add actions, 'd' to remove them"),
        Line::from("  • Progress shows dynamic counts like [2/4] or [1/5]"),
        Line::from("  • All features adapt to your action count automatically"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Tips:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from("  • Complete at least one task daily to maintain your streak"),
        Line::from("  • Focus on consistency over perfection"),
        Line::from("  • Review your goals weekly and adjust as needed"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press any key to close this help",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Left);

    // Center the help box
    let help_area = centered_rect(80, 80, area);
    f.render_widget(Clear, help_area); // Clear the background
    f.render_widget(help, help_area);
}

fn render_info(f: &mut Frame, area: Rect, info_msg: &str) {
    let info_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            info_msg,
            Style::default().fg(Color::Green),
        )]),
        Line::from(""),
    ];

    let info = Paragraph::new(info_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .alignment(Alignment::Center);

    // Create a small notification area at the top
    let info_area = Rect {
        x: area.x + area.width / 3,
        y: area.y + 2,
        width: area.width / 3,
        height: 5,
    };
    f.render_widget(Clear, info_area); // Clear the background
    f.render_widget(info, info_area);
}

fn render_error(f: &mut Frame, area: Rect, error_msg: &str) {
    let error_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Error",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(error_msg),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press any key to dismiss",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let error = Paragraph::new(error_text)
        .block(
            Block::default()
                .title(" Error ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        )
        .alignment(Alignment::Center);

    // Center the error box
    let error_area = centered_rect(60, 30, area);
    f.render_widget(Clear, error_area); // Clear the background
    f.render_widget(error, error_area);
}

/// Helper function to create a centered rect
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

fn render_yesterday_selector(f: &mut Frame, area: Rect, app: &App) {
    if let InputMode::CopyingFromYesterday {
        ref yesterday_goals,
        ref selections,
        selection_index,
    } = &app.input_mode
    {
        // Create a modal for yesterday's actions (80% width, 60% height)
        let modal_area = centered_rect(80, 60, area);

        // Clear the background
        f.render_widget(Clear, modal_area);

        // Build the list of actions with checkboxes
        let mut items = Vec::new();
        let mut action_index = 0;

        for (outcome_idx, outcome) in yesterday_goals.outcomes().iter().enumerate() {
            // Add outcome header
            let outcome_name = match outcome_idx {
                0 => "Work",
                1 => "Health",
                2 => "Family",
                _ => "",
            };
            items.push(Line::from(vec![Span::styled(
                format!("  {} ", outcome_name),
                Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )]));

            // Add actions for this outcome
            for action in &outcome.actions {
                if !action.text.is_empty() {
                    let checkbox = if selections[action_index] {
                        "[x]"
                    } else {
                        "[ ]"
                    };
                    let completed_marker = if action.completed { " ✓" } else { "" };

                    let style = if action_index == *selection_index {
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    items.push(Line::from(vec![Span::styled(
                        format!("  {} {}{}", checkbox, action.text, completed_marker),
                        style,
                    )]));
                }
                action_index += 1;
            }

            // Add spacing between outcomes
            if outcome_idx < 2 {
                items.push(Line::from(""));
            }
        }

        // Create the paragraph with all items
        let content = Paragraph::new(items).block(
            Block::default()
                .title(" Copy from Yesterday ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        // Render the content
        f.render_widget(content, modal_area);

        // Render help text at the bottom
        let help_text = vec![Line::from(vec![
            Span::styled("j/↓", Style::default().fg(Color::Green)),
            Span::raw(": Down | "),
            Span::styled("k/↑", Style::default().fg(Color::Green)),
            Span::raw(": Up | "),
            Span::styled("Space", Style::default().fg(Color::Green)),
            Span::raw(": Toggle | "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": Copy Selected | "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": Cancel"),
        ])];

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        // Position help text at bottom of modal
        let help_area = Rect {
            x: modal_area.x + 2,
            y: modal_area.y + modal_area.height - 2,
            width: modal_area.width - 4,
            height: 1,
        };
        f.render_widget(help, help_area);
    }
}

fn render_template_selector(f: &mut Frame, area: Rect, app: &App) {
    if let InputMode::TemplateSelection {
        ref templates,
        ref template_names,
        selection_index,
        outcome_type,
    } = &app.input_mode
    {
        // Create a modal for template selection (60% width, 50% height)
        let modal_area = centered_rect(60, 50, area);

        // Clear the background
        f.render_widget(Clear, modal_area);

        // Build the list of templates
        let mut items = Vec::new();

        if template_names.is_empty() {
            items.push(Line::from(vec![Span::styled(
                "No templates available",
                Style::default().fg(Color::DarkGray),
            )]));
        } else {
            for (idx, template_name) in template_names.iter().enumerate() {
                let style = if idx == *selection_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                // Get template actions for preview
                let preview = if let Some(actions) = templates.get_template(template_name) {
                    let action_count = actions.len();
                    format!(
                        " ({} action{})",
                        action_count,
                        if action_count == 1 { "" } else { "s" }
                    )
                } else {
                    String::new()
                };

                items.push(Line::from(vec![Span::styled(
                    format!("  {}{}", template_name, preview),
                    style,
                )]));
            }
        }

        // Create the paragraph with all items
        let content = Paragraph::new(items).block(
            Block::default()
                .title(format!(
                    " Select Template for {} ",
                    match outcome_type {
                        crate::models::OutcomeType::Work => "Work",
                        crate::models::OutcomeType::Health => "Health",
                        crate::models::OutcomeType::Family => "Family",
                    }
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        // Render the content
        f.render_widget(content, modal_area);

        // Render help text at the bottom
        let help_text = vec![Line::from(vec![
            Span::styled("j/↓", Style::default().fg(Color::Green)),
            Span::raw(": Down | "),
            Span::styled("k/↑", Style::default().fg(Color::Green)),
            Span::raw(": Up | "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": Apply Template | "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": Cancel"),
        ])];

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        // Position help text at bottom of modal
        let help_area = Rect {
            x: modal_area.x + 2,
            y: modal_area.y + modal_area.height - 2,
            width: modal_area.width - 4,
            height: 1,
        };
        f.render_widget(help, help_area);
    }
}

fn render_template_saving(f: &mut Frame, area: Rect, app: &App) {
    if let InputMode::TemplateSaving {
        ref buffer,
        outcome_type,
        ..
    } = &app.input_mode
    {
        // Create a small modal for template name input (50% width, 20% height)
        let modal_area = centered_rect(50, 20, area);

        // Clear the background
        f.render_widget(Clear, modal_area);

        // Create the input field
        let input = Paragraph::new(buffer.as_str())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title(format!(
                        " Save {} Actions as Template ",
                        match outcome_type {
                            crate::models::OutcomeType::Work => "Work",
                            crate::models::OutcomeType::Health => "Health",
                            crate::models::OutcomeType::Family => "Family",
                        }
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)),
            );

        // Render the input field
        f.render_widget(input, modal_area);

        // Render help text
        let help_text = vec![Line::from(vec![
            Span::raw("Enter template name | "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": Save | "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": Cancel"),
        ])];

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        // Position help text at bottom of modal
        let help_area = Rect {
            x: modal_area.x + 2,
            y: modal_area.y + modal_area.height - 2,
            width: modal_area.width - 4,
            height: 1,
        };
        f.render_widget(help, help_area);
    }
}

fn render_goal_editor(f: &mut Frame, area: Rect, app: &App) {
    if let InputMode::GoalEditing {
        outcome_type,
        ref buffer,
        ..
    } = &app.input_mode
    {
        // Create a modal for goal editing (50% width, 20% height)
        let modal_area = centered_rect(50, 20, area);

        let input = Paragraph::new(buffer.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .title(format!(
                        " Edit {} Goal (Max 100 chars) ",
                        match outcome_type {
                            OutcomeType::Work => "Work",
                            OutcomeType::Health => "Health",
                            OutcomeType::Family => "Family",
                        }
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            );

        f.render_widget(Clear, modal_area); // Clear the background
        f.render_widget(input, modal_area);

        // Show cursor at the end of the text
        f.set_cursor(
            modal_area.x + 1 + buffer.len() as u16,
            modal_area.y + 1,
        );
    }
}

fn render_vision_editor(f: &mut Frame, area: Rect, app: &App) {
    if let InputMode::VisionEditing {
        outcome_type,
        ref buffer,
        ..
    } = &app.input_mode
    {
        // Create a large modal for vision editing (90% width, 70% height)
        let modal_area = centered_rect(90, 70, area);

        // Clear the background
        f.render_widget(Clear, modal_area);

        // Split the modal into header, content, and footer
        let _chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(modal_area);

        // Build the title
        let title = format!(" Edit 5-Year {} Vision ", outcome_type.as_str());

        // Create the outer block
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        // Split buffer into lines for display
        let lines: Vec<String> = buffer.split('\n').map(|s| s.to_string()).collect();

        // Add cursor to the last line
        let mut display_lines = lines.clone();
        if let Some(last) = display_lines.last_mut() {
            last.push('│'); // Add cursor
        } else {
            display_lines.push("│".to_string()); // Empty buffer shows cursor
        }

        // Convert to Lines for rendering
        let text_lines: Vec<Line> = display_lines
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect();

        // Create the text widget
        let text = Paragraph::new(text_lines)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true });

        // Render the main text area
        f.render_widget(text, modal_area);

        // Render character count in top-right corner
        let char_count = format!("{}/{}", buffer.len(), crate::models::MAX_VISION_LENGTH);
        let char_count_style = if buffer.len() > crate::models::MAX_VISION_LENGTH - 100 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let char_counter = Paragraph::new(char_count)
            .style(char_count_style)
            .alignment(Alignment::Right);

        // Position the counter in the top-right of the modal
        let counter_area = Rect {
            x: modal_area.x + modal_area.width - 15,
            y: modal_area.y + 1,
            width: 13,
            height: 1,
        };
        f.render_widget(char_counter, counter_area);

        // Render help text at the bottom
        let help_text = vec![Line::from(vec![
            Span::styled(format!("{}+Enter", get_modifier_key()), Style::default().fg(Color::Green)),
            Span::raw(": Save | "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": Cancel | "),
            Span::styled("Enter", Style::default().fg(Color::DarkGray)),
            Span::raw(": New line"),
        ])];

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        // Position help text at bottom of modal
        let help_area = Rect {
            x: modal_area.x + 2,
            y: modal_area.y + modal_area.height - 2,
            width: modal_area.width - 4,
            height: 1,
        };
        f.render_widget(help, help_area);
    }
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let help_text = if matches!(app.input_mode, InputMode::CopyingFromYesterday { .. }) {
        Line::from(vec![
            Span::raw("j/k: Navigate | "),
            Span::raw("Space: Toggle | "),
            Span::raw("Enter: Copy | "),
            Span::raw("Esc: Cancel"),
        ])
    } else if matches!(app.input_mode, InputMode::TemplateSelection { .. }) {
        Line::from(vec![
            Span::raw("j/k: Navigate | "),
            Span::raw("Enter: Apply | "),
            Span::raw("Esc: Cancel"),
        ])
    } else if matches!(app.input_mode, InputMode::TemplateSaving { .. }) {
        Line::from(vec![
            Span::raw("Type name | "),
            Span::raw("Enter: Save | "),
            Span::raw("Esc: Cancel"),
        ])
    } else if matches!(app.input_mode, InputMode::GoalEditing { .. }) {
        Line::from(vec![
            Span::raw("Enter/F2/Ctrl+S: Save | "),
            Span::raw("Esc: Cancel"),
        ])
    } else if matches!(app.input_mode, InputMode::VisionEditing { .. } | InputMode::Reflecting { .. }) {
        Line::from(vec![
            Span::raw(format!("{}+Enter/Ctrl+S/F2: Save | ", get_modifier_key())),
            Span::raw("Esc: Cancel | "),
            Span::raw("Enter: New line"),
        ])
    } else if matches!(app.input_mode, InputMode::Editing { .. }) {
        Line::from(vec![
            Span::raw("Enter: Save | "),
            Span::raw("Esc: Cancel | "),
            Span::raw("Backspace: Delete char"),
        ])
    } else {
        // Phase-specific help text
        match app.ritual_phase {
            crate::models::RitualPhase::Morning => {
                let template_count = app.templates.get_template_names().len();
                let template_text = if template_count > 0 {
                    if template_count <= 9 {
                        format!("1-{}: Apply templates | ", template_count)
                    } else {
                        "1-9: Apply templates | ".to_string()
                    }
                } else {
                    "No templates | ".to_string()
                };
                
                Line::from(vec![
                    Span::styled("Morning Mode: ", Style::default().fg(Color::Yellow)),
                    Span::raw("y: Quick-fill yesterday | "),
                    Span::raw(template_text),
                    Span::raw("Tab: Switch | "),
                    Span::raw("e: Edit | "),
                    Span::raw("s: Save | "),
                Span::raw("q: Quit | "),
                    Span::raw("?: Help"),
                ])
            }
            crate::models::RitualPhase::Evening => {
                let total_actions = app.goals.work.actions.len() + 
                                  app.goals.health.actions.len() + 
                                  app.goals.family.actions.len();
                                  
                let quick_complete_text = match total_actions {
                    0..=9 => format!("1-{}: Quick complete | ", total_actions),
                    10..=15 => "1-9, a-f: Quick complete | ".to_string(),
                    _ => "1-9, a-f: Quick complete | ".to_string(), // Max 15 supported
                };
                
                Line::from(vec![
                    Span::styled("Evening Mode: ", Style::default().fg(Color::Blue)),
                    Span::raw(quick_complete_text),
                    Span::raw("r: Reflect | "),
                    Span::raw("d: Summary | "),
                    Span::raw("Tab: Switch | "),
                    Span::raw("s: Save | "),
                Span::raw("q: Quit | "),
                    Span::raw("?: Help"),
                ])
            }
            _ => Line::from(vec![
                Span::raw("Tab: Switch | "),
                Span::raw("j/k: Navigate | "),
                Span::raw("Space: Toggle | "),
                Span::raw("e: Edit | "),
                Span::raw("a: Add | "),
                Span::raw("d: Delete | "),
                Span::raw("t/T: Templates | "),
                Span::raw("y: Yesterday | "),
                Span::raw("m/n: Phase | "),
                Span::raw("v: Vision | "),
                Span::raw("g: Goal | "),
                Span::raw("s: Save | "),
                Span::raw("q: Quit | "),
                Span::raw("?: Help"),
            ]),
        }
    };

    let footer = Paragraph::new(vec![help_text])
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}

/// Render progress gauge for evening phase
fn render_progress_gauge(f: &mut Frame, area: Rect, stats: &crate::models::CompletionStats) {
    use ratatui::widgets::Gauge;

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!(
                    " Today's Progress - Day {} ",
                    stats.streak_days.unwrap_or(0)
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .gauge_style(Style::default().fg(match stats.percentage {
            80..=100 => Color::Green,
            50..=79 => Color::Yellow,
            _ => Color::Red,
        }))
        .percent(stats.percentage)
        .label(format!(
            "{}/{} tasks completed ({}%)",
            stats.completed, stats.total, stats.percentage
        ));

    f.render_widget(gauge, area);
}

/// Render reflection modal
fn render_reflection_modal(f: &mut Frame, area: Rect, app: &App) {
    if let InputMode::Reflecting {
        outcome_type,
        ref buffer,
        ..
    } = &app.input_mode
    {
        // Create modal (70% width, 50% height)
        let modal_area = centered_rect(70, 50, area);

        // Clear the background
        f.render_widget(Clear, modal_area);

        // Create the reflection input area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Text area
                Constraint::Length(2), // Help text
            ])
            .split(modal_area);

        // Build the text area
        let reflection_text = Paragraph::new(buffer.as_str())
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .block(
                Block::default()
                    .title(format!(" {} Reflection ", outcome_type.as_str()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)),
            );

        f.render_widget(reflection_text, chunks[0]);

        // Render help text
        let help_text = vec![Line::from(vec![
            Span::styled(format!("{}+Enter", get_modifier_key()), Style::default().fg(Color::Green)),
            Span::raw(": Save | "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": Cancel | "),
            Span::styled("Enter", Style::default().fg(Color::DarkGray)),
            Span::raw(": New line"),
        ])];

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(help, chunks[1]);
    }
}
