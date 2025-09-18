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
    } else if matches!(app.input_mode, InputMode::ObjectiveSelection { .. }) {
        render_objective_selector(f, chunks[1], app);
    } else if matches!(app.input_mode, InputMode::ObjectiveCreation { .. }) {
        render_objective_creation(f, chunks[1], app);
    } else if matches!(app.input_mode, InputMode::Reflecting { .. }) {
        render_reflection_modal(f, chunks[1], app);
    } else if matches!(app.input_mode, InputMode::IndicatorManagement { .. }) {
        render_indicator_manager(f, chunks[1], app);
    } else if matches!(app.input_mode, InputMode::IndicatorCreation { .. }) {
        render_indicator_creator(f, chunks[1], app);
    } else if matches!(app.input_mode, InputMode::UpdatingIndicator(_)) {
        render_update_overlay(f, chunks[1], app);
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
    let total_actions = app.goals.work.actions.len()
        + app.goals.health.actions.len()
        + app.goals.family.actions.len();
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

fn render_indicator_detail(f: &mut Frame, area: Rect, indicator: &crate::models::Indicator) {
    use crate::widgets::IndicatorProgress;
    use ratatui::widgets::{Block, Borders, Paragraph};
    
    // Create progress data from indicator
    let current_value = indicator.current_value;
    let target_value = indicator.target_value;
    let history: Vec<f64> = indicator.history.iter()
        .map(|entry| entry.value)
        .collect();
    
    let progress = IndicatorProgress::new(current_value, target_value, history.clone());
    
    // Split area for different visualizations
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(3),  // Title and value
            ratatui::layout::Constraint::Length(3),  // Progress gauge
            ratatui::layout::Constraint::Length(3),  // Trend and bar
            ratatui::layout::Constraint::Min(3),     // Sparkline
        ])
        .split(area);
    
    // Title and current value
    let title_text = format!(
        "{}: {:.1}/{:.1} {}",
        indicator.name,
        current_value,
        target_value,
        &indicator.unit
    );
    let title = Paragraph::new(title_text)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, chunks[0]);
    
    // Progress gauge
    let gauge = progress.render_gauge()
        .label(format!("{:.0}%", progress.get_percentage()))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(gauge, chunks[1]);
    
    // Trend and progress bar
    let trend_text = format!(
        "{} {} {}",
        progress.render_trend(),
        progress.render_bar(),
        match progress.trend {
            crate::widgets::TrendDirection::Up => "Improving",
            crate::widgets::TrendDirection::Down => "Declining",
            crate::widgets::TrendDirection::Stable => "Stable",
        }
    );
    let trend_widget = Paragraph::new(trend_text)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(trend_widget, chunks[2]);
    
    // Sparkline (if enough history)
    if history.len() > 1 {
        let sparkline_data = progress.get_sparkline_data();
        let sparkline = ratatui::widgets::Sparkline::default()
            .data(&sparkline_data)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan))
            .block(Block::default()
                .borders(Borders::TOP)
                .title("History"));
        f.render_widget(sparkline, chunks[3]);
    }
}

fn render_actions_pane(f: &mut Frame, area: Rect, app: &App) {
    let selected_outcome = app.get_selected_outcome();

    // Check if we're editing the current action
    let is_editing = matches!(app.input_mode, InputMode::Editing { .. });
    
    // Build display items with expansion
    let mut display_items = Vec::new();
    let mut _selectable_indices = Vec::new();
    let mut _display_index = 0;
    
    for (idx, action) in selected_outcome.actions.iter().enumerate() {
        // Determine if this action is being edited
        let editing_this_action = is_editing && app.active_pane == Pane::Actions && idx == app.action_index;
        
        // Add expansion symbol if not editing
        let symbol = if !editing_this_action && app.ui_state.is_expanded(&action.id) { 
            "▼ " 
        } else if !editing_this_action { 
            "▶ " 
        } else {
            ""
        };
        
        // Use status-based checkbox with visual indicators
        let checkbox = match action.status {
            crate::models::ActionStatus::Planned => "[ ]",
            crate::models::ActionStatus::InProgress => "[→]",
            crate::models::ActionStatus::Done => "[✓]",
            crate::models::ActionStatus::Skipped => "[~]",
            crate::models::ActionStatus::Blocked => "[✗]",
        };

        // Style for main action line
        let style = if editing_this_action {
            Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD)
        } else if app.active_pane == Pane::Actions && idx == app.action_index {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else if action.text.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            match action.status {
                crate::models::ActionStatus::Done => Style::default().fg(Color::Green),
                crate::models::ActionStatus::InProgress => Style::default().fg(Color::Yellow),
                crate::models::ActionStatus::Blocked => {
                    Style::default().fg(Color::Red).add_modifier(Modifier::DIM)
                }
                crate::models::ActionStatus::Skipped => Style::default().fg(Color::DarkGray),
                crate::models::ActionStatus::Planned => Style::default(),
            }
        };

        // Text to display for action
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
        
        // Main action line
        let action_line = format!("{}{} {}", symbol, checkbox, text);
        display_items.push(ListItem::new(action_line).style(style));
        _selectable_indices.push((idx, None, None)); // Action level
        _display_index += 1;
        
        // Add objectives and indicators if expanded and not editing
        if !editing_this_action && app.ui_state.is_expanded(&action.id) {
            let all_objective_ids = action.get_all_objective_ids();
            
            if !all_objective_ids.is_empty() {
                // Display each linked objective
                for (obj_idx, objective_id) in all_objective_ids.iter().enumerate() {
                    // Determine prefix for tree display
                    let prefix = if all_objective_ids.len() == 1 {
                        "  └─"
                    } else if obj_idx == all_objective_ids.len() - 1 {
                        "  └─"
                    } else {
                        "  ├─"
                    };
                    
                    // Find the objective by ID
                    if let Some(objective) = app.objectives.objectives.iter().find(|obj| obj.id == *objective_id) {
                        // Objective line with icon
                        let obj_line = format!("{} 📎 Objective: {}", prefix, objective.title);
                        display_items.push(ListItem::new(obj_line).style(Style::default().fg(Color::Cyan)));
                        _selectable_indices.push((idx, Some(obj_idx), None)); // Objective level
                        _display_index += 1;
                        
                        // Add indicators for this objective (indented more for multiple objectives)
                        let indicator_indent = if all_objective_ids.len() > 1 { "        " } else { "      " };
                        if !objective.indicators.is_empty() {
                        for (ind_idx, indicator_id) in objective.indicators.iter().enumerate() {
                            // Find the indicator by ID
                            if let Some(indicator_def) = app.indicators.indicators.iter().find(|ind| ind.id == *indicator_id) {
                                // Fetch actual indicator value from observations
                                let current_value = app.get_latest_indicator_value(indicator_id).unwrap_or(0.0);
                                let target_value = indicator_def.target.unwrap_or(100.0);
                                let history: Vec<f64> = vec![];  // Empty history for now
                                
                                let progress = crate::widgets::IndicatorProgress::new(
                                    current_value,
                                    target_value,
                                    history
                                );
                                
                                let percentage = progress.get_percentage();
                                let trend = progress.render_trend();
                                let bar = progress.render_bar();
                                
                                // Check if this indicator is selected
                                let mut indicator_index = 0;
                                let mut is_selected_indicator = false;
                                
                                // Calculate the global indicator index for this action
                                for (o_idx, oid) in all_objective_ids.iter().enumerate() {
                                    if o_idx < obj_idx {
                                        // Count indicators in previous objectives
                                        if let Some(prev_obj) = app.objectives.objectives.iter().find(|obj| obj.id == *oid) {
                                            indicator_index += prev_obj.indicators.len();
                                        }
                                    } else if o_idx == obj_idx {
                                        // Add current indicator index
                                        indicator_index += ind_idx;
                                        break;
                                    }
                                }
                                
                                // Check if this indicator is selected
                                if app.active_pane == Pane::Actions 
                                    && idx == app.action_index 
                                    && app.ui_state.selected_indicator_index == Some(indicator_index) {
                                    is_selected_indicator = true;
                                }
                                
                                // Create indicator line with real progress visualization
                                let indicator_line = format!("{}{} {} [{}] {}% {}", 
                                    indicator_indent,
                                    if ind_idx == objective.indicators.len() - 1 { "└─" } else { "├─" },
                                    indicator_def.name,
                                    bar,
                                    percentage,
                                    trend
                                );
                                
                                // Apply highlighting if selected
                                let indicator_style = if is_selected_indicator {
                                    Style::default().bg(Color::Magenta).fg(Color::White).add_modifier(Modifier::BOLD)
                                } else {
                                    Style::default().fg(Color::Gray)
                                };
                                
                                display_items.push(ListItem::new(indicator_line).style(indicator_style));
                                _selectable_indices.push((idx, Some(obj_idx), Some(ind_idx))); // Indicator level
                                _display_index += 1;
                            }
                        }
                        
                        // Add overall progress line
                        let overall_progress = app.calculate_objective_progress(&objective);
                        let progress_color = if overall_progress >= 100.0 {
                            Color::Green
                        } else if overall_progress >= 70.0 {
                            Color::Yellow
                        } else {
                            Color::Red
                        };
                        let overall_line = format!("{}Overall Progress: {:.0}%", indicator_indent, overall_progress);
                        display_items.push(ListItem::new(overall_line).style(Style::default().fg(progress_color).add_modifier(Modifier::BOLD)));
                        _display_index += 1;
                    } else {
                        // No indicators defined
                        let no_ind_line = format!("{}(No indicators defined)", indicator_indent);
                        display_items.push(ListItem::new(no_ind_line).style(Style::default().fg(Color::DarkGray)));
                        _display_index += 1;
                    }
                } else {
                    // Objective not found
                    let not_found_line = format!("{} ⚠️ Objective '{}' not found", prefix, objective_id);
                    display_items.push(ListItem::new(not_found_line).style(Style::default().fg(Color::Yellow)));
                    _display_index += 1;
                }
                }
            } else {
                // Action has no objective linked
                let no_obj_line = "  └─ (No objective linked)";
                display_items.push(ListItem::new(no_obj_line).style(Style::default().fg(Color::DarkGray)));
                _display_index += 1;
            }
        }
    }

    let border_style = if app.ui_state.selected_indicator_index.is_some() {
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
    } else if app.active_pane == Pane::Actions {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let title = if app.ui_state.selected_indicator_index.is_some() {
        format!(" {} Actions [INDICATOR MODE - j/k navigate, i update, ESC exit] ", selected_outcome.outcome_type.as_str())
    } else {
        format!(" {} Actions ", selected_outcome.outcome_type.as_str())
    };

    let actions_list = List::new(display_items).block(
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
        Line::from("  Space     - Cycle status: Planned → InProgress → Done → Skipped → Blocked (in Actions pane)"),
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
            "Indicators & Objectives:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from("  o          - Link action to objective (in Actions pane)"),
        Line::from("  Enter      - Expand/collapse action to show indicators"),
        Line::from("  u/U        - Enter indicator mode (on action with indicators)"),
        Line::from("  i          - Update selected indicator (when in indicator mode)"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Indicator Update Mode:",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )]),
        Line::from("  0-9        - Type custom value directly"),
        Line::from("  a          - Small increment (+1 count, +15 min, +10%)"),
        Line::from("  s          - Medium increment (+5 count, +30 min, +25%)"),
        Line::from("  d          - Large increment (+10 count, +60 min, +50%)"),
        Line::from("  +/-        - Fine increment/decrement"),
        Line::from("  c          - Clear value to start fresh"),
        Line::from("  Enter      - Save update"),
        Line::from("  Esc        - Cancel update"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Variable Actions:",
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

fn render_objective_selector(f: &mut Frame, area: Rect, app: &App) {
    if let InputMode::ObjectiveSelection {
        domain,
        selection_index,
    } = &app.input_mode
    {
        // Create a modal for objective selection (70% width, 60% height)
        let modal_area = centered_rect(70, 60, area);

        // Clear the background
        f.render_widget(Clear, modal_area);
        
        // Get the current action's linked objectives
        let current_action = match app.outcome_index {
            0 => &app.goals.work.actions[app.action_index],
            1 => &app.goals.health.actions[app.action_index],
            2 => &app.goals.family.actions[app.action_index],
            _ => return,
        };
        let linked_objectives = current_action.get_all_objective_ids();

        // Get objectives for the current domain
        let domain_objectives: Vec<&crate::models::Objective> = app
            .objectives
            .objectives
            .iter()
            .filter(|obj| obj.domain == *domain)
            .collect();

        // Build the list of objectives
        let mut items = Vec::new();
        
        // Add a status line showing how many objectives are linked
        let linked_count = domain_objectives.iter()
            .filter(|obj| linked_objectives.contains(&obj.id))
            .count();
        
        if linked_count > 0 {
            items.push(Line::from(vec![Span::styled(
                format!("  {} objective{} linked to this action", 
                        linked_count, 
                        if linked_count == 1 { "" } else { "s" }),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC),
            )]));
            items.push(Line::from(""));
        }

        if domain_objectives.is_empty() {
            items.push(Line::from(vec![Span::styled(
                "No objectives available for this domain",
                Style::default().fg(Color::DarkGray),
            )]));
            items.push(Line::from(""));
        } else {
            for (idx, objective) in domain_objectives.iter().enumerate() {
                let is_linked = linked_objectives.contains(&objective.id);
                let is_selected = idx == *selection_index;
                
                let style = if is_selected {
                    if is_linked {
                        Style::default()
                            .bg(Color::DarkGray)
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD)
                    }
                } else if is_linked {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                // Format objective with status indicator
                let status_char = match objective.status {
                    crate::models::ObjectiveStatus::Active => "●",
                    crate::models::ObjectiveStatus::Paused => "⏸",
                    crate::models::ObjectiveStatus::Completed => "✓",
                    crate::models::ObjectiveStatus::Dropped => "✗",
                };
                
                // Show checkmark with better visual feedback
                let linked_indicator = if is_linked { 
                    "☑ " // Using a checkbox character for better visibility
                } else { 
                    "☐ " // Empty checkbox
                };

                // Show title with optional description preview
                let description_preview = if let Some(ref desc) = objective.description {
                    if desc.len() > 50 {
                        format!(" - {}...", &desc[..47])
                    } else {
                        format!(" - {}", desc)
                    }
                } else {
                    String::new()
                };

                items.push(Line::from(vec![Span::styled(
                    format!("{}{} {}{}", linked_indicator, status_char, objective.title, description_preview),
                    style,
                )]));
                
                // Add a hint for the selected item
                if is_selected {
                    let hint = if is_linked {
                        "    Press Enter to unlink"
                    } else {
                        "    Press Enter to link"
                    };
                    items.push(Line::from(vec![Span::styled(
                        hint,
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                    )]));
                }
            }
        }

        // Add "Create New Objective" option at the end
        items.push(Line::from(""));
        let create_new_style = if *selection_index == domain_objectives.len() {
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        };
        items.push(Line::from(vec![Span::styled(
            "  + Create New Objective",
            create_new_style,
        )]));

        // Create the paragraph with all items
        let content = Paragraph::new(items).block(
            Block::default()
                .title(format!(
                    " Select Objectives for {} Action ",
                    match domain {
                        crate::models::OutcomeType::Work => "Work",
                        crate::models::OutcomeType::Health => "Health",
                        crate::models::OutcomeType::Family => "Family",
                    }
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        );

        // Render the content
        f.render_widget(content, modal_area);

        // Render help text at the bottom
        let help_text = vec![Line::from(vec![
            Span::styled("j/↓", Style::default().fg(Color::Green)),
            Span::raw(": Down | "),
            Span::styled("k/↑", Style::default().fg(Color::Green)),
            Span::raw(": Up | "),
            Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(": Toggle | "),
            Span::styled("i", Style::default().fg(Color::Green)),
            Span::raw(": Indicators | "),
            Span::styled("c", Style::default().fg(Color::Yellow)),
            Span::raw(": Clear All | "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": Done"),
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

fn render_objective_creation(f: &mut Frame, area: Rect, app: &App) {
    if let InputMode::ObjectiveCreation {
        domain,
        ref buffer,
    } = &app.input_mode
    {
        // Create a small modal for objective title input (50% width, 20% height)
        let modal_area = centered_rect(50, 20, area);

        // Clear the background
        f.render_widget(Clear, modal_area);

        // Create the input field
        let input = Paragraph::new(buffer.as_str())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title(format!(
                        " Create New {} Objective ",
                        match domain {
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
            Span::raw("Enter objective title | "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": Create | "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": Cancel"),
        ])];

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .wrap(ratatui::widgets::Wrap { trim: true });

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

fn render_indicator_manager(f: &mut Frame, area: Rect, app: &App) {
    if let InputMode::IndicatorManagement {
        ref objective_title,
        ref indicators,
        selection_index,
        ref editing_field,
        ..
    } = &app.input_mode
    {
        // Create a modal for indicator management (80% width, 70% height)
        let modal_area = centered_rect(80, 70, area);

        // Clear the background
        f.render_widget(Clear, modal_area);

        // Build the list of indicators
        let mut items = Vec::new();

        // Add header
        items.push(Line::from(vec![Span::styled(
            format!("  Indicators for: {} ", objective_title),
            Style::default()
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                .fg(Color::Magenta),
        )]));
        items.push(Line::from(""));

        if indicators.is_empty() {
            items.push(Line::from(vec![Span::styled(
                "  No indicators defined yet",
                Style::default().fg(Color::DarkGray),
            )]));
            items.push(Line::from(""));
        } else {
            // Show existing indicators
            for (idx, indicator) in indicators.iter().enumerate() {
                let is_selected = idx == *selection_index;
                let is_editing = is_selected && editing_field.is_some();
                
                let style = if is_editing {
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                // Format indicator details
                let status_icon = if indicator.active { "●" } else { "○" };
                let kind_str = match indicator.kind {
                    crate::models::IndicatorKind::Leading => "L",
                    crate::models::IndicatorKind::Lagging => "La",
                };
                let unit_str = match &indicator.unit {
                    crate::models::IndicatorUnit::Count => "count",
                    crate::models::IndicatorUnit::Minutes => "min",
                    crate::models::IndicatorUnit::Dollars => "$",
                    crate::models::IndicatorUnit::Percent => "%",
                    crate::models::IndicatorUnit::Custom(s) => s.as_str(),
                };
                let target_str = indicator
                    .target
                    .map(|t| format!(" → {:.1}", t))
                    .unwrap_or_default();
                let direction_str = match indicator.direction {
                    crate::models::IndicatorDirection::HigherIsBetter => "↑",
                    crate::models::IndicatorDirection::LowerIsBetter => "↓",
                    crate::models::IndicatorDirection::WithinRange => "↔",
                };

                // Handle editing mode display
                let name_display = if is_editing {
                    if let Some(crate::app::IndicatorEditField::Name(ref buf)) = editing_field {
                        format!("{}│", buf)
                    } else if let Some(crate::app::IndicatorEditField::Target(ref buf)) = editing_field {
                        format!("{} [{}] {} {}{} {} (editing target: {}│)", 
                            status_icon, kind_str, indicator.name, unit_str, target_str, direction_str, buf)
                    } else if let Some(crate::app::IndicatorEditField::Notes(ref buf)) = editing_field {
                        format!("{} [{}] {} {}{} {} (editing notes: {}│)", 
                            status_icon, kind_str, indicator.name, unit_str, target_str, direction_str, buf)
                    } else {
                        format!("{} [{}] {} {}{} {}", 
                            status_icon, kind_str, indicator.name, unit_str, target_str, direction_str)
                    }
                } else {
                    format!("{} [{}] {} {}{} {}", 
                        status_icon, kind_str, indicator.name, unit_str, target_str, direction_str)
                };

                items.push(Line::from(vec![Span::styled(
                    format!("  {}", name_display),
                    style,
                )]));

                // Show notes if present and not editing
                if !is_editing {
                    if let Some(notes) = indicator.notes.as_ref() {
                        let notes_preview = if notes.len() > 60 {
                            format!("    Notes: {}...", &notes[..57])
                        } else {
                            format!("    Notes: {}", notes)
                        };
                        items.push(Line::from(vec![Span::styled(
                            notes_preview,
                            Style::default().fg(Color::DarkGray),
                        )]));
                    }
                }
            }
        }

        // Add "Create New" option
        items.push(Line::from(""));
        let create_new_style = if *selection_index == indicators.len() {
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };
        items.push(Line::from(vec![Span::styled(
            "  + Create New Indicator",
            create_new_style,
        )]));

        // Create the paragraph with all items
        let content = Paragraph::new(items).block(
            Block::default()
                .title(" Indicator Management ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        // Render the content
        f.render_widget(content, modal_area);

        // Render help text at the bottom
        let help_text = if editing_field.is_some() {
            vec![Line::from(vec![
                Span::raw("Type to edit | "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(": Save | "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw(": Cancel"),
            ])]
        } else {
            vec![Line::from(vec![
                Span::styled("j/↓", Style::default().fg(Color::Green)),
                Span::raw(": Down | "),
                Span::styled("k/↑", Style::default().fg(Color::Green)),
                Span::raw(": Up | "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(": Edit Name | "),
                Span::styled("t", Style::default().fg(Color::Green)),
                Span::raw(": Target | "),
                Span::styled("n", Style::default().fg(Color::Green)),
                Span::raw(": Notes | "),
                Span::styled("Space", Style::default().fg(Color::Green)),
                Span::raw(": Toggle Active | "),
                Span::styled("d", Style::default().fg(Color::Red)),
                Span::raw(": Delete | "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw(": Back"),
            ])]
        };

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

fn render_indicator_creator(f: &mut Frame, area: Rect, app: &App) {
    if let InputMode::IndicatorCreation {
        ref objective_title,
        field_index,
        ref name_buffer,
        ref kind,
        ref unit,
        ref unit_custom_buffer,
        ref target_buffer,
        ref direction,
        ref notes_buffer,
        ..
    } = &app.input_mode
    {
        // Create a modal for indicator creation (70% width, 50% height)
        let modal_area = centered_rect(70, 50, area);

        // Clear the background
        f.render_widget(Clear, modal_area);

        // Build the form fields
        let mut items = Vec::new();

        // Title
        items.push(Line::from(vec![Span::styled(
            format!("  Creating Indicator for: {} ", objective_title),
            Style::default()
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                .fg(Color::Magenta),
        )]));
        items.push(Line::from(""));

        // Field 0: Name
        let name_style = if *field_index == 0 {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(Line::from(vec![
            Span::styled("  Name: ", Style::default()),
            Span::styled(
                if name_buffer.is_empty() {
                    "_".to_string()
                } else {
                    format!("{}│", name_buffer)
                },
                name_style,
            ),
        ]));

        // Field 1: Kind
        let kind_style = if *field_index == 1 {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let kind_str = match kind {
            crate::models::IndicatorKind::Leading => "Leading",
            crate::models::IndicatorKind::Lagging => "Lagging",
        };
        items.push(Line::from(vec![
            Span::styled("  Kind: ", Style::default()),
            Span::styled(kind_str, kind_style),
            Span::styled(" (L=Leading, A=Lagging)", Style::default().fg(Color::DarkGray)),
        ]));

        // Field 2: Unit
        let unit_style = if *field_index == 2 {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let unit_str = match unit {
            crate::models::IndicatorUnit::Count => "Count",
            crate::models::IndicatorUnit::Minutes => "Minutes",
            crate::models::IndicatorUnit::Dollars => "Dollars",
            crate::models::IndicatorUnit::Percent => "Percent",
            crate::models::IndicatorUnit::Custom(_) => {
                if unit_custom_buffer.is_empty() {
                    "Custom: _"
                } else {
                    unit_custom_buffer.as_str()
                }
            }
        };
        items.push(Line::from(vec![
            Span::styled("  Unit: ", Style::default()),
            Span::styled(unit_str, unit_style),
            Span::styled(" (C=Count, M=Minutes, D=Dollars, P=Percent, U=Custom)", Style::default().fg(Color::DarkGray)),
        ]));

        // Field 3: Target
        let target_style = if *field_index == 3 {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(Line::from(vec![
            Span::styled("  Target: ", Style::default()),
            Span::styled(
                if target_buffer.is_empty() {
                    "(optional)".to_string()
                } else {
                    format!("{}│", target_buffer)
                },
                target_style,
            ),
        ]));

        // Field 4: Direction
        let direction_style = if *field_index == 4 {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let direction_str = match direction {
            crate::models::IndicatorDirection::HigherIsBetter => "Higher is Better ↑",
            crate::models::IndicatorDirection::LowerIsBetter => "Lower is Better ↓",
            crate::models::IndicatorDirection::WithinRange => "Within Range ↔",
        };
        items.push(Line::from(vec![
            Span::styled("  Direction: ", Style::default()),
            Span::styled(direction_str, direction_style),
            Span::styled(" (H=Higher, L=Lower, R=Range)", Style::default().fg(Color::DarkGray)),
        ]));

        // Field 5: Notes
        let notes_style = if *field_index == 5 {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(Line::from(vec![
            Span::styled("  Notes: ", Style::default()),
            Span::styled(
                if notes_buffer.is_empty() {
                    "(optional)".to_string()
                } else {
                    format!("{}│", notes_buffer)
                },
                notes_style,
            ),
        ]));

        // Create the paragraph with all items
        let content = Paragraph::new(items).block(
            Block::default()
                .title(" Create New Indicator ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        // Render the content
        f.render_widget(content, modal_area);

        // Render help text at the bottom
        let help_text = vec![Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Green)),
            Span::raw(": Next Field | "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": Create Indicator | "),
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
        f.set_cursor(modal_area.x + 1 + buffer.len() as u16, modal_area.y + 1);
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
            Span::styled(
                format!("{}+Enter", get_modifier_key()),
                Style::default().fg(Color::Green),
            ),
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
    } else if matches!(app.input_mode, InputMode::ObjectiveSelection { .. }) {
        Line::from(vec![
            Span::raw("j/k: Navigate | "),
            Span::raw("Enter: Select | "),
            Span::raw("n: None | "),
            Span::raw("Esc: Cancel"),
        ])
    } else if matches!(app.input_mode, InputMode::GoalEditing { .. }) {
        Line::from(vec![
            Span::raw("Enter/F2/Ctrl+S: Save | "),
            Span::raw("Esc: Cancel"),
        ])
    } else if matches!(
        app.input_mode,
        InputMode::VisionEditing { .. } | InputMode::Reflecting { .. }
    ) {
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
                let total_actions = app.goals.work.actions.len()
                    + app.goals.health.actions.len()
                    + app.goals.family.actions.len();

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
            _ => {
                // Check if in indicator selection mode
                if app.ui_state.selected_indicator_index.is_some() {
                    Line::from(vec![
                        Span::styled("INDICATOR MODE: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                        Span::raw("j/k: Navigate indicators | "),
                        Span::raw("u: Update value | "),
                        Span::raw("Tab/ESC: Exit mode | "),
                        Span::raw("?: Help"),
                    ])
                } else {
                    Line::from(vec![
                        Span::raw("Tab: Switch | "),
                        Span::raw("j/k: Navigate | "),
                        Span::raw("Space: Toggle | "),
                        Span::raw("e: Edit | "),
                        Span::raw("a: Add | "),
                        Span::raw("d: Delete | "),
                        Span::raw("o: Objectives | "),
                        Span::raw("t/T: Templates | "),
                        Span::raw("y: Yesterday | "),
                        Span::raw("m/n: Phase | "),
                        Span::raw("v: Vision | "),
                        Span::raw("g: Goal | "),
                        Span::raw("s: Save | "),
                        Span::raw("q: Quit | "),
                        Span::raw("?: Help"),
                    ])
                }
            },
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

/// Render indicator update overlay
fn render_update_overlay(f: &mut Frame, area: Rect, app: &App) {
    if let InputMode::UpdatingIndicator(id) = &app.input_mode {
        // Find the indicator
        let indicator = app.indicators.indicators.iter()
            .find(|ind| ind.id == *id);
        
        if let Some(indicator) = indicator {
            // Create centered popup (60% width, 40% height)
            let popup_area = centered_rect(60, 40, area);
            f.render_widget(Clear, popup_area);
            
            let popup = Block::bordered()
                .title(format!(" Update: {} ", indicator.name))
                .border_style(Style::default().fg(Color::Yellow));
            
            let inner = popup.inner(popup_area);
            f.render_widget(popup, popup_area);
            
            // Layout based on indicator type
            let chunks = Layout::vertical([
                Constraint::Length(3),   // Current/Target display
                Constraint::Length(3),   // Progress visualization
                Constraint::Length(4),   // Quick actions
                Constraint::Length(3),   // Custom input
                Constraint::Length(2),   // Help text
            ]).split(inner);
            
            // Get current value from observations
            let current_value = app.get_latest_indicator_value(id).unwrap_or(0.0);
            
            // Current and target values
            let value_display = format!("Current: {:.1} {} | Target: {:.1} {}",
                current_value,
                match &indicator.unit {
                    crate::models::IndicatorUnit::Count => "count",
                    crate::models::IndicatorUnit::Minutes => "min",
                    crate::models::IndicatorUnit::Dollars => "$",
                    crate::models::IndicatorUnit::Percent => "%",
                    crate::models::IndicatorUnit::Custom(s) => s.as_str(),
                },
                indicator.target.unwrap_or(100.0),
                match &indicator.unit {
                    crate::models::IndicatorUnit::Count => "count",
                    crate::models::IndicatorUnit::Minutes => "min",
                    crate::models::IndicatorUnit::Dollars => "$",
                    crate::models::IndicatorUnit::Percent => "%",
                    crate::models::IndicatorUnit::Custom(s) => s.as_str(),
                }
            );
            f.render_widget(Paragraph::new(value_display).centered(), chunks[0]);
            
            // Progress visualization
            use crate::widgets::IndicatorProgress;
            let progress = IndicatorProgress::new(
                current_value,
                indicator.target.unwrap_or(100.0),
                vec![] // Empty history for now
            );
            let progress_text = format!("[{}] {:.0}%", 
                progress.render_bar(),
                progress.get_percentage()
            );
            f.render_widget(
                Paragraph::new(progress_text)
                    .style(Style::default().fg(Color::Green))
                    .centered(),
                chunks[1]
            );
            
            // Quick actions based on type
            let quick_actions = match indicator.unit {
                crate::models::IndicatorUnit::Count => {
                    format!("[1] +1  [3] +3  [5] +5")
                },
                crate::models::IndicatorUnit::Minutes => {
                    format!("[1] +30min  [2] +1hr  [3] +2hrs")
                },
                crate::models::IndicatorUnit::Percent => {
                    format!("[2] 25%  [5] 50%  [7] 75%  [9] 100%")
                },
                _ => {
                    format!("[+/-] Adjust value")
                }
            };
            
            f.render_widget(
                Paragraph::new(quick_actions).centered(),
                chunks[2]
            );
            
            // Custom input field with cursor
            let input_text = format!("{}│", app.ui_state.update_buffer);
            let input = Paragraph::new(input_text)
                .block(Block::bordered().title("Custom value"))
                .style(Style::default().fg(Color::White));
            f.render_widget(input, chunks[3]);
            
            // Help text
            let help = match indicator.unit {
                crate::models::IndicatorUnit::Count => "[Enter number] [Enter] Save [Esc] Cancel",
                crate::models::IndicatorUnit::Minutes => "[Enter minutes] [Enter] Save [Esc] Cancel",
                crate::models::IndicatorUnit::Percent => "[Enter 0-100] [Enter] Save [Esc] Cancel",
                _ => "[Enter value] [Enter] Save [Esc] Cancel",
            };
            f.render_widget(
                Paragraph::new(help)
                    .style(Style::default().fg(Color::DarkGray))
                    .centered(),
                chunks[4]
            );
        }
    }
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
            Span::styled(
                format!("{}+Enter", get_modifier_key()),
                Style::default().fg(Color::Green),
            ),
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
