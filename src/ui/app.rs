use crate::ui::{theme::FocusFiveTheme, layout::{create_layout}, popup::{TextEditor, EditorResult}, stats::Statistics, error::ErrorDisplay, help};
use ratatui::{
    style::{Style, Modifier},
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph},
    text::{Line, Span},
    Frame,
    layout::{Rect, Constraint, Direction, Layout},
};
use crate::models::{DailyGoals, Config, OutcomeType};
use crossterm::event::KeyCode;

#[derive(PartialEq)]
pub enum FocusPanel {
    Outcomes,
    Actions,
}

pub struct App {
    pub goals: DailyGoals,
    pub config: Config,
    pub theme: FocusFiveTheme,
    pub selected_outcome: OutcomeType,
    pub selected_action: usize,
    pub focus_panel: FocusPanel,
    pub text_editor: TextEditor,
    pub statistics: Statistics,
    pub error_display: ErrorDisplay,
}

impl App {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let today = chrono::Local::now().date_naive();
        let goals = crate::data::load_or_create_goals(today, &config)?;
        let theme = FocusFiveTheme::default();
        let statistics = Statistics::from_current_goals(&goals, &config);
        Ok(Self {
            goals,
            config: config.clone(),
            statistics,
            theme,
            selected_outcome: OutcomeType::Work,
            selected_action: 0,
            focus_panel: FocusPanel::Outcomes,
            text_editor: TextEditor::new(""),
            error_display: ErrorDisplay::new(),
        })
    }

    pub fn handle_key(&mut self, key: KeyCode) -> anyhow::Result<bool> {
        // If editor is active, route input to it
        if self.text_editor.is_active {
            match self.text_editor.handle_input(key) {
                EditorResult::Save => {
                    // Save the edited text to the selected action
                    let action_index = self.selected_action;
                    let new_text = self.text_editor.text.clone();

                    // Deactivate editor first to avoid borrow issues
                    self.text_editor.deactivate();

                    // Store old text for potential revert
                    let old_text = {
                        let outcome = self.get_selected_outcome();
                        outcome.actions[action_index].text.clone()
                    };

                    // Now update the action text
                    {
                        let outcome = self.get_selected_outcome_mut();
                        outcome.actions[action_index].text = new_text.clone();
                    }

                    // Save to file
                    match crate::data::write_goals_file(&self.goals, &self.config) {
                        Ok(_) => {
                            self.error_display.show_info(format!("✓ Action text updated"));
                        }
                        Err(e) => {
                            self.error_display.show_error(format!("Failed to save: {}", e));
                            // Revert the change
                            let outcome = self.get_selected_outcome_mut();
                            outcome.actions[action_index].text = old_text;
                            return Err(e);
                        }
                    }
                }
                EditorResult::Cancel => {
                    // Just deactivate without saving
                    self.text_editor.deactivate();
                }
                EditorResult::Continue => {}
            }
            return Ok(false);
        }

        // Normal key handling when editor is not active
        match key {
            KeyCode::Char('q') => return Ok(true), // Exit
            KeyCode::Tab => self.switch_panel(),
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Enter | KeyCode::Char(' ') => self.toggle_current()?,
            KeyCode::Char('e') => self.open_editor(),
            _ => {}
        }
        Ok(false)
    }

    fn switch_panel(&mut self) {
        self.focus_panel = match self.focus_panel {
            FocusPanel::Outcomes => FocusPanel::Actions,
            FocusPanel::Actions => FocusPanel::Outcomes,
        };

        // When switching to actions, reset selected_action to be within range for selected outcome
        if self.focus_panel == FocusPanel::Actions {
            self.selected_action = 0;
        }
    }

    fn move_up(&mut self) {
        match self.focus_panel {
            FocusPanel::Outcomes => {
                self.selected_outcome = match self.selected_outcome {
                    OutcomeType::Work => OutcomeType::Family,
                    OutcomeType::Health => OutcomeType::Work,
                    OutcomeType::Family => OutcomeType::Health,
                };
            }
            FocusPanel::Actions => {
                if self.selected_action > 0 {
                    self.selected_action -= 1;
                } else {
                    self.selected_action = 2; // Wrap to bottom
                }
            }
        }
    }

    fn move_down(&mut self) {
        match self.focus_panel {
            FocusPanel::Outcomes => {
                self.selected_outcome = match self.selected_outcome {
                    OutcomeType::Work => OutcomeType::Health,
                    OutcomeType::Health => OutcomeType::Family,
                    OutcomeType::Family => OutcomeType::Work,
                };
            }
            FocusPanel::Actions => {
                if self.selected_action < 2 {
                    self.selected_action += 1;
                } else {
                    self.selected_action = 0; // Wrap to top
                }
            }
        }
    }

    fn toggle_current(&mut self) -> anyhow::Result<()> {
        if self.focus_panel == FocusPanel::Actions {
            let action_index = self.selected_action;

            // Get the current completion status
            let was_completed = {
                let outcome = self.get_selected_outcome();
                outcome.actions[action_index].completed
            };

            // Toggle the completion status
            {
                let outcome = self.get_selected_outcome_mut();
                outcome.actions[action_index].completed = !was_completed;
            }

            // Auto-save
            match crate::data::write_goals_file(&self.goals, &self.config) {
                Ok(_) => {
                    let msg = if !was_completed {
                        format!("✓ Action marked as completed")
                    } else {
                        format!("○ Action marked as pending")
                    };
                    self.error_display.show_info(msg);
                }
                Err(e) => {
                    self.error_display.show_error(format!("Failed to save: {}", e));
                    // Revert the change
                    let outcome = self.get_selected_outcome_mut();
                    outcome.actions[action_index].completed = was_completed;
                    return Err(e);
                }
            }

            // Update statistics after toggling
            self.statistics = Statistics::from_current_goals(&self.goals, &self.config);
        }
        Ok(())
    }

    fn get_selected_outcome_mut(&mut self) -> &mut crate::models::Outcome {
        match self.selected_outcome {
            OutcomeType::Work => &mut self.goals.work,
            OutcomeType::Health => &mut self.goals.health,
            OutcomeType::Family => &mut self.goals.family,
        }
    }

    fn get_selected_outcome(&self) -> &crate::models::Outcome {
        match self.selected_outcome {
            OutcomeType::Work => &self.goals.work,
            OutcomeType::Health => &self.goals.health,
            OutcomeType::Family => &self.goals.family,
        }
    }

    fn open_editor(&mut self) {
        // Only allow editing when focused on Actions panel
        if self.focus_panel == FocusPanel::Actions {
            let action_text = {
                let outcome = self.get_selected_outcome();
                outcome.actions[self.selected_action].text.clone()
            };
            self.text_editor.activate(&action_text);
        }
    }

    pub fn render(&mut self, f: &mut Frame) {
        // Clear background
        f.render_widget(
            Block::default().style(Style::default().bg(self.theme.background)),
            f.area()
        );

        let layout = create_layout(f.area());

        self.render_header(f, layout.header);
        self.render_outcomes(f, layout.outcomes);
        self.render_actions(f, layout.actions);
        self.render_stats(f, layout.stats);
        self.render_footer(f, layout.footer);

        // Render editor popup on top if active
        if self.text_editor.is_active {
            self.text_editor.render(f, &self.theme);
        }

        // Render error display on top if active
        if self.error_display.is_active() {
            self.error_display.render(f, f.area(), &self.theme);
        }
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                "FOCUSFIVE",
                Style::default()
                    .fg(self.theme.header)
                    .add_modifier(Modifier::BOLD)
            ),
            Span::raw(" - "),
            Span::styled(
                self.goals.date.format("%B %d, %Y").to_string(),
                Style::default().fg(self.theme.text_primary)
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(self.theme.border))
                .style(Style::default().bg(self.theme.panel_bg))
        );
        f.render_widget(header, area);
    }

    fn render_outcomes(&mut self, f: &mut Frame, area: Rect) {
        let mut outcomes: Vec<ListItem> = Vec::new();

        for (_idx, (outcome_type, label, color)) in [
            (OutcomeType::Work, "Work", self.theme.work_color),
            (OutcomeType::Health, "Health", self.theme.health_color),
            (OutcomeType::Family, "Family", self.theme.family_color),
        ].iter().enumerate() {
            let outcome = match outcome_type {
                OutcomeType::Work => &self.goals.work,
                OutcomeType::Health => &self.goals.health,
                OutcomeType::Family => &self.goals.family,
            };

            let completed = outcome.actions.iter().filter(|a| a.completed).count();

            let is_selected = self.focus_panel == FocusPanel::Outcomes &&
                             self.selected_outcome == *outcome_type;

            let style = if is_selected {
                Style::default()
                    .bg(self.theme.border)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            outcomes.push(
                ListItem::new(Line::from(vec![
                    Span::styled("■ ", Style::default().fg(*color)),
                    Span::styled(*label, Style::default().fg(self.theme.text_primary)),
                    Span::raw(" "),
                    Span::styled(
                        format!("[{}/3]", completed),
                        Style::default().fg(self.theme.text_secondary)
                    ),
                ]))
                .style(style)
            );
        }

        let border_color = if self.focus_panel == FocusPanel::Outcomes {
            self.theme.header
        } else {
            self.theme.border
        };

        let outcomes_list = List::new(outcomes)
            .block(
                Block::default()
                    .title(" OUTCOMES ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color))
                    .style(Style::default().bg(self.theme.panel_bg))
            );

        f.render_widget(outcomes_list, area);
    }

    fn render_actions(&mut self, f: &mut Frame, area: Rect) {
        let selected_outcome = self.get_selected_outcome();
        let mut actions_list = Vec::new();

        let outcome_color = match self.selected_outcome {
            OutcomeType::Work => self.theme.work_color,
            OutcomeType::Health => self.theme.health_color,
            OutcomeType::Family => self.theme.family_color,
        };

        let outcome_prefix = match self.selected_outcome {
            OutcomeType::Work => "W",
            OutcomeType::Health => "H",
            OutcomeType::Family => "F",
        };

        for (idx, action) in selected_outcome.actions.iter().enumerate() {
            let checkbox = if action.completed { "[x]" } else { "[ ]" };
            let color = if action.completed { self.theme.completed } else { self.theme.text_secondary };

            let is_selected = self.focus_panel == FocusPanel::Actions &&
                             self.selected_action == idx;

            let style = if is_selected {
                Style::default()
                    .bg(self.theme.border)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            actions_list.push(
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", outcome_prefix), Style::default().fg(outcome_color)),
                    Span::styled(checkbox, Style::default().fg(color)),
                    Span::raw(" "),
                    Span::styled(&action.text, Style::default().fg(self.theme.text_primary)),
                ]))
                .style(style)
            );
        }

        let border_color = if self.focus_panel == FocusPanel::Actions {
            self.theme.header
        } else {
            self.theme.border
        };

        let actions = List::new(actions_list)
            .block(
                Block::default()
                    .title(format!(" ACTIONS - {} ", match self.selected_outcome {
                        OutcomeType::Work => "Work",
                        OutcomeType::Health => "Health",
                        OutcomeType::Family => "Family",
                    }))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color))
                    .style(Style::default().bg(self.theme.panel_bg))
            );

        f.render_widget(actions, area);
    }

    fn render_stats(&self, f: &mut Frame, area: Rect) {
        use crate::ui::charts::{create_daily_gauge, create_outcome_gauges, render_trend_sparkline, create_weekly_chart};

        // Create a layout for the stats panel
        let _stats_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Title area
                Constraint::Length(5),   // Daily gauge
                Constraint::Length(7),   // Outcome gauges
                Constraint::Min(10),     // Weekly chart
                Constraint::Length(5),   // Monthly sparkline
            ])
            .split(area);

        // Render title block
        let title_block = Block::default()
            .title(" STATISTICS ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.theme.border))
            .style(Style::default().bg(self.theme.panel_bg));
        f.render_widget(title_block, area);

        // Inner area to avoid drawing over borders
        let inner = area.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 });
        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),   // Daily gauge
                Constraint::Length(4),   // Outcome gauges
                Constraint::Min(8),      // Weekly chart
                Constraint::Length(4),   // Monthly sparkline
            ])
            .split(inner);

        // Daily completion gauge
        let daily_gauge = create_daily_gauge(self.statistics.daily_completion, "TODAY'S PROGRESS", &self.theme);
        f.render_widget(daily_gauge, inner_layout[0]);

        // Outcome gauges (Work, Health, Family) - horizontal layout
        let outcome_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(inner_layout[1]);

        let (work_gauge, health_gauge, family_gauge) = create_outcome_gauges(&self.statistics, &self.theme);
        f.render_widget(work_gauge, outcome_layout[0]);
        f.render_widget(health_gauge, outcome_layout[1]);
        f.render_widget(family_gauge, outcome_layout[2]);

        // Weekly bar chart
        if inner_layout[2].height > 5 {  // Only render if there's enough space
            let weekly_chart = create_weekly_chart(&self.statistics, &self.theme);
            f.render_widget(weekly_chart, inner_layout[2]);
        }

        // Monthly trend sparkline
        if !self.statistics.monthly_trend.is_empty() && inner_layout[3].height > 2 {
            render_trend_sparkline(
                &self.statistics.monthly_trend,
                "30-DAY TREND",
                &self.theme,
                f,
                inner_layout[3]
            );
        }
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        // Add a bordered frame around the help text
        let footer_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.theme.border))
            .style(Style::default().bg(self.theme.panel_bg));

        f.render_widget(footer_block.clone(), area);

        // Render help text inside the border
        let inner_area = area.inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 });
        help::render_help(f, inner_area, &self.theme);
    }
}