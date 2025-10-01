use crate::models::{
    Config, DailyGoals, FiveYearVision, Indicator, IndicatorDirection, IndicatorKind,
    IndicatorType, IndicatorUnit, IndicatorsData, Objective, ObjectiveStatus, ObjectivesData,
    Observation, ObservationSource, OutcomeType,
};
use crate::ui::{
    dashboard_layout::DashboardLayout,
    error::ErrorDisplay,
    help,
    layout::create_layout,
    popup::{centered_rect, EditorResult, TextEditor},
    stats::Statistics,
    theme::{FinancialTheme, FocusFiveTheme},
};
use crate::ui_state::ExpandableActionState;
use crate::widgets::{
    alternative_signals::{AlternativeSignal, AlternativeSignalsWidget},
    LiveMetricsWidget, PerformanceChart, SentimentWidget,
};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Sparkline,
    },
    Frame,
};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(PartialEq)]
pub enum FocusPanel {
    Outcomes,
    Actions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardPanel {
    Market,
    Performance,
    Sentiment,
    Signals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorContext {
    Action {
        outcome_type: OutcomeType,
        index: usize,
    },
    Vision {
        outcome_type: OutcomeType,
    },
    ObjectiveTitle {
        outcome_type: OutcomeType,
        objective_id: Option<String>,
        link_action: Option<usize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectiveModalState {
    pub outcome_type: OutcomeType,
    pub action_index: usize,
    pub selection: usize,
}

#[derive(Debug, Clone)]
pub struct ObjectiveChoice {
    pub storage_index: usize,
    pub id: String,
    pub title: String,
    pub status: ObjectiveStatus,
}

#[derive(Debug, Clone)]
pub enum ModalState {
    ObjectivePicker(ObjectiveModalState),
    IndicatorUpdate(IndicatorUpdateState),
}

#[derive(Debug, Clone)]
pub struct IndicatorUpdateState {
    pub indicator_id: String,
    pub name: String,
    pub unit: IndicatorUnit,
    pub indicator_type: IndicatorType,
    pub direction: IndicatorDirection,
    pub target: Option<f64>,
    pub previous_value: Option<f64>,
    pub latest_value: Option<f64>,
    pub history: Vec<f64>,
    pub last_updated: Option<chrono::NaiveDate>,
    pub buffer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrendStatus {
    Improving,
    Declining,
    Stable,
}

pub struct App {
    pub goals: DailyGoals,
    pub config: Config,
    pub theme: FocusFiveTheme,
    pub financial_theme: FinancialTheme,
    pub selected_outcome: OutcomeType,
    pub selected_action: usize,
    pub focus_panel: FocusPanel,
    pub show_dashboard: bool,
    pub dashboard_focus: DashboardPanel,
    pub text_editor: TextEditor,
    pub editor_context: Option<EditorContext>,
    pub modal: Option<ModalState>,
    pub statistics: Statistics,
    pub error_display: ErrorDisplay,
    pub ui_state: ExpandableActionState,
    pub objectives: ObjectivesData,
    pub indicators: IndicatorsData,
    pub indicators_map: HashMap<String, Indicator>,
    pub vision: FiveYearVision,
    pub vision_needs_save: bool,
    pub dashboard_signal_index: usize,
    pub dashboard_signal_ids: Vec<String>,
    pub dashboard_performance_index: usize,
    pub dashboard_performance_ids: Vec<String>,
    pub dashboard_market_index: usize,
    pub dashboard_market_ids: Vec<String>,
    // NEW: Day navigation support
    pub current_date: chrono::NaiveDate,
    pub max_date: chrono::NaiveDate,
}

impl App {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let today = chrono::Local::now().date_naive();
        let goals = crate::data::load_or_create_goals(today, &config)?;
        let theme = FocusFiveTheme::default();
        let statistics = Statistics::from_current_goals(&goals, &config);
        let objectives = crate::data::load_or_create_objectives(&config)?;
        let indicators = crate::data::load_or_create_indicators(&config)?;
        let vision = crate::data::load_or_create_vision(&config)?;

        // Create indicators map for quick lookup
        let mut indicators_map = HashMap::new();
        for ind_def in &indicators.indicators {
            // Determine indicator type based on unit
            let indicator_type = match &ind_def.unit {
                crate::models::IndicatorUnit::Minutes => IndicatorType::Duration,
                crate::models::IndicatorUnit::Percent => IndicatorType::Percentage,
                crate::models::IndicatorUnit::Count => IndicatorType::Counter,
                crate::models::IndicatorUnit::Dollars => IndicatorType::Counter,
                crate::models::IndicatorUnit::Custom(s) if s == "boolean" => IndicatorType::Boolean,
                crate::models::IndicatorUnit::Custom(s) if s == "hours" => IndicatorType::Duration,
                crate::models::IndicatorUnit::Custom(s) if s == "percentage" => {
                    IndicatorType::Percentage
                }
                _ => IndicatorType::Counter,
            };

            // Convert unit to string representation
            let unit_str = match &ind_def.unit {
                crate::models::IndicatorUnit::Count => "count",
                crate::models::IndicatorUnit::Minutes => "minutes",
                crate::models::IndicatorUnit::Dollars => "dollars",
                crate::models::IndicatorUnit::Percent => "percentage",
                crate::models::IndicatorUnit::Custom(s) => s.as_str(),
            }
            .to_string();

            let indicator = Indicator {
                id: ind_def.id.clone(),
                name: ind_def.name.clone(),
                indicator_type,
                current_value: 0.0, // Start at 0, will be updated from observations
                target_value: ind_def.target.unwrap_or(100.0),
                unit: unit_str,
                history: vec![],
            };
            indicators_map.insert(ind_def.id.clone(), indicator);
        }

        Ok(Self {
            goals,
            config: config.clone(),
            statistics,
            theme,
            financial_theme: FinancialTheme::default(),
            selected_outcome: OutcomeType::Work,
            selected_action: 0,
            focus_panel: FocusPanel::Outcomes,
            show_dashboard: false,
            dashboard_focus: DashboardPanel::Market,
            text_editor: TextEditor::new("Edit Action"),
            editor_context: None,
            modal: None,
            error_display: ErrorDisplay::new(),
            ui_state: ExpandableActionState::new(),
            objectives,
            indicators,
            indicators_map,
            vision,
            vision_needs_save: false,
            dashboard_signal_index: 0,
            dashboard_signal_ids: Vec::new(),
            dashboard_performance_index: 0,
            dashboard_performance_ids: Vec::new(),
            dashboard_market_index: 0,
            dashboard_market_ids: Vec::new(),
            // NEW: Initialize day navigation fields
            current_date: today,
            max_date: today,
        })
    }

    // NEW: Day navigation methods
    pub fn navigate_to_previous_day(&mut self) -> anyhow::Result<()> {
        let previous_date = self.current_date - chrono::Duration::days(1);
        
        // Save current changes before navigating
        self.save_current_goals()?;
        
        // Load goals for previous day
        self.goals = crate::data::load_or_create_goals(previous_date, &self.config)?;
        self.current_date = previous_date;
        
        // Reset selection to avoid out-of-bounds
        self.selected_outcome = OutcomeType::Work;
        self.selected_action = 0;
        
        // Update statistics for new date
        self.statistics = Statistics::from_current_goals(&self.goals, &self.config);
        
        Ok(())
    }

    pub fn navigate_to_next_day(&mut self) -> anyhow::Result<()> {
        let next_date = self.current_date + chrono::Duration::days(1);
        
        // Restrict future navigation
        if next_date > self.max_date {
            return Ok(()); // Silently ignore future navigation attempts
        }
        
        // Save current changes before navigating
        self.save_current_goals()?;
        
        // Load goals for next day
        self.goals = crate::data::load_or_create_goals(next_date, &self.config)?;
        self.current_date = next_date;
        
        // Reset selection to avoid out-of-bounds
        self.selected_outcome = OutcomeType::Work;
        self.selected_action = 0;
        
        // Update statistics for new date
        self.statistics = Statistics::from_current_goals(&self.goals, &self.config);
        
        Ok(())
    }

    fn save_current_goals(&self) -> anyhow::Result<()> {
        crate::data::write_goals_file(&self.goals, &self.config)?;
        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyCode) -> anyhow::Result<bool> {
        // If editor is active, route input to it
        if self.text_editor.is_active {
            match self.text_editor.handle_input(key) {
                EditorResult::Save => {
                    let new_text = self.text_editor.text.clone();
                    self.text_editor.deactivate();

                    if let Some(context) = self.editor_context.take() {
                        match context {
                            EditorContext::Action {
                                outcome_type,
                                index,
                            } => {
                                let outcome_snapshot = self.get_outcome_by_type(outcome_type);
                                if index >= outcome_snapshot.actions.len() {
                                    self.error_display.show_error(
                                        "Action index out of range when saving".to_string(),
                                    );
                                    return Ok(false);
                                }

                                let previous_text = outcome_snapshot.actions[index].text.clone();

                                {
                                    let outcome = self.get_outcome_by_type_mut(outcome_type);
                                    outcome.actions[index].text = new_text.clone();
                                }

                                if let Err(e) =
                                    crate::data::write_goals_file(&self.goals, &self.config)
                                {
                                    self.error_display
                                        .show_error(format!("Failed to save: {}", e));
                                    let outcome = self.get_outcome_by_type_mut(outcome_type);
                                    outcome.actions[index].text = previous_text;
                                    return Err(e);
                                }

                                // Refresh statistics when actions change
                                self.statistics =
                                    Statistics::from_current_goals(&self.goals, &self.config);
                            }
                            EditorContext::Vision { outcome_type } => {
                                let backup = self.vision.clone();
                                self.vision.set_vision(&outcome_type, new_text.clone());

                                if let Err(e) = crate::data::save_vision(&self.vision, &self.config)
                                {
                                    self.error_display
                                        .show_error(format!("Failed to save vision: {}", e));
                                    self.vision = backup;
                                    self.vision_needs_save = true;
                                    return Err(e);
                                }

                                self.vision_needs_save = false;
                            }
                            EditorContext::ObjectiveTitle {
                                outcome_type,
                                objective_id,
                                link_action,
                            } => {
                                let title = new_text.trim();
                                if title.is_empty() {
                                    self.error_display
                                        .show_error("Objective title cannot be empty".to_string());
                                    return Ok(false);
                                }

                                let backup = self.objectives.clone();
                                let created_id = match objective_id {
                                    Some(ref existing_id) => {
                                        if let Some(objective) = self
                                            .objectives
                                            .objectives
                                            .iter_mut()
                                            .find(|o| &o.id == existing_id)
                                        {
                                            objective.title = title.to_string();
                                            objective.modified = chrono::Utc::now();
                                            None
                                        } else {
                                            self.error_display.show_error(
                                                "Objective not found for update".to_string(),
                                            );
                                            return Ok(false);
                                        }
                                    }
                                    None => {
                                        let objective =
                                            Objective::new(outcome_type, title.to_string());
                                        let new_id = objective.id.clone();
                                        self.objectives.objectives.push(objective);
                                        Some(new_id)
                                    }
                                };

                                if let Err(e) =
                                    crate::data::save_objectives(&self.objectives, &self.config)
                                {
                                    self.error_display
                                        .show_error(format!("Failed to save objectives: {}", e));
                                    self.objectives = backup;
                                    return Err(e);
                                }

                                if let Some(obj_id) = created_id.clone() {
                                    if let Some(action_idx) = link_action {
                                        if let Err(e) = self.link_action_to_objective(
                                            outcome_type,
                                            action_idx,
                                            &obj_id,
                                        ) {
                                            self.error_display.show_error(format!(
                                                "Failed to link objective: {}",
                                                e
                                            ));
                                            self.objectives = backup;
                                            let _ = crate::data::save_objectives(
                                                &self.objectives,
                                                &self.config,
                                            );
                                            return Err(e);
                                        }

                                        if let Some(index) =
                                            self.objective_index_in_domain(outcome_type, &obj_id)
                                        {
                                            if let Some(ModalState::ObjectivePicker(
                                                ref mut modal,
                                            )) = self.modal
                                            {
                                                modal.selection = index;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                EditorResult::Cancel => {
                    self.text_editor.deactivate();
                    self.editor_context = None;
                }
                EditorResult::Continue => {}
            }
            return Ok(false);
        }

        if self.modal.is_some() {
            if self.handle_modal_key(key)? {
                return Ok(false);
            }
        }

        // Dashboard toggle is available globally when editor is not active
        if key == KeyCode::Char('d') {
            self.toggle_dashboard_view();
            return Ok(false);
        }

        // When dashboard is visible, delegate key handling to dashboard controls
        if self.show_dashboard {
            return self.handle_dashboard_key(key);
        }

        // Normal key handling when editor is not active
        match key {
            KeyCode::Char('q') => return Ok(true), // Exit
            KeyCode::Tab => self.switch_panel(),
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Char(' ') => self.toggle_current()?,
            KeyCode::Enter | KeyCode::Char('e') => self.toggle_expansion(),
            KeyCode::Char('E') => self.open_editor(),
            KeyCode::Char('v') => self.open_vision_editor(),
            KeyCode::Char('o') => self.open_objective_picker(),
            KeyCode::Char('i') => self.open_selected_indicator_update()?,
            // NEW: Day navigation using Page Up/Down keys
            KeyCode::PageUp => {
                if let Err(e) = self.navigate_to_previous_day() {
                    self.error_display.show_error(format!("Navigation failed: {}", e));
                }
            }
            KeyCode::PageDown => {
                if let Err(e) = self.navigate_to_next_day() {
                    self.error_display.show_error(format!("Navigation failed: {}", e));
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn toggle_dashboard_view(&mut self) {
        self.show_dashboard = !self.show_dashboard;

        if self.show_dashboard {
            self.dashboard_focus = DashboardPanel::Market;
            self.dashboard_signal_index = 0;
            self.dashboard_performance_index = 0;
            self.dashboard_market_index = 0;
        } else {
            self.focus_panel = FocusPanel::Outcomes;
            self.dashboard_signal_ids.clear();
            self.dashboard_performance_ids.clear();
            self.dashboard_market_ids.clear();
        }
    }

    fn handle_dashboard_key(&mut self, key: KeyCode) -> anyhow::Result<bool> {
        match key {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Esc => {
                self.toggle_dashboard_view();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.dashboard_focus = match self.dashboard_focus {
                    DashboardPanel::Market => DashboardPanel::Signals,
                    DashboardPanel::Performance => DashboardPanel::Market,
                    DashboardPanel::Sentiment => DashboardPanel::Performance,
                    DashboardPanel::Signals => DashboardPanel::Sentiment,
                };
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.dashboard_focus = match self.dashboard_focus {
                    DashboardPanel::Market => DashboardPanel::Performance,
                    DashboardPanel::Performance => DashboardPanel::Sentiment,
                    DashboardPanel::Sentiment => DashboardPanel::Signals,
                    DashboardPanel::Signals => DashboardPanel::Market,
                };
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.dashboard_focus == DashboardPanel::Market
                    && !self.dashboard_market_ids.is_empty()
                {
                    if self.dashboard_market_index == 0 {
                        self.dashboard_market_index = self.dashboard_market_ids.len() - 1;
                    } else {
                        self.dashboard_market_index -= 1;
                    }
                } else if self.dashboard_focus == DashboardPanel::Signals
                    && !self.dashboard_signal_ids.is_empty()
                {
                    if self.dashboard_signal_index == 0 {
                        self.dashboard_signal_index = self.dashboard_signal_ids.len() - 1;
                    } else {
                        self.dashboard_signal_index -= 1;
                    }
                } else if self.dashboard_focus == DashboardPanel::Performance
                    && !self.dashboard_performance_ids.is_empty()
                {
                    if self.dashboard_performance_index == 0 {
                        self.dashboard_performance_index = self.dashboard_performance_ids.len() - 1;
                    } else {
                        self.dashboard_performance_index -= 1;
                    }
                } else {
                    self.dashboard_focus = match self.dashboard_focus {
                        DashboardPanel::Market => DashboardPanel::Market,
                        DashboardPanel::Performance => DashboardPanel::Performance,
                        DashboardPanel::Sentiment => DashboardPanel::Market,
                        DashboardPanel::Signals => DashboardPanel::Performance,
                    };
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.dashboard_focus == DashboardPanel::Market
                    && !self.dashboard_market_ids.is_empty()
                {
                    self.dashboard_market_index =
                        (self.dashboard_market_index + 1) % self.dashboard_market_ids.len();
                } else if self.dashboard_focus == DashboardPanel::Signals
                    && !self.dashboard_signal_ids.is_empty()
                {
                    self.dashboard_signal_index =
                        (self.dashboard_signal_index + 1) % self.dashboard_signal_ids.len();
                } else if self.dashboard_focus == DashboardPanel::Performance
                    && !self.dashboard_performance_ids.is_empty()
                {
                    self.dashboard_performance_index =
                        (self.dashboard_performance_index + 1) % self.dashboard_performance_ids.len();
                } else {
                    self.dashboard_focus = match self.dashboard_focus {
                        DashboardPanel::Market => DashboardPanel::Sentiment,
                        DashboardPanel::Performance => DashboardPanel::Signals,
                        DashboardPanel::Sentiment => DashboardPanel::Sentiment,
                        DashboardPanel::Signals => DashboardPanel::Signals,
                    };
                }
            }
            KeyCode::Enter | KeyCode::Char('i') => {
                if self.dashboard_focus == DashboardPanel::Signals {
                    if let Some(indicator_id) = self
                        .dashboard_signal_ids
                        .get(self.dashboard_signal_index)
                        .cloned()
                    {
                        self.open_indicator_update_modal(&indicator_id)?;
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_modal_key(&mut self, key: KeyCode) -> anyhow::Result<bool> {
        if let Some(ModalState::ObjectivePicker(mut state)) = self.modal.clone() {
            let choices = self.objective_choices(state.outcome_type);
            let total_items = choices.len() + 1; // +1 for "Create New"

            match key {
                KeyCode::Esc => {
                    self.modal = None;
                    return Ok(true);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if total_items == 0 {
                        state.selection = 0;
                    } else if state.selection == 0 {
                        state.selection = total_items.saturating_sub(1);
                    } else {
                        state.selection -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if total_items == 0 {
                        state.selection = 0;
                    } else {
                        state.selection = (state.selection + 1) % total_items.max(1);
                    }
                }
                KeyCode::Enter => {
                    if state.selection == choices.len() {
                        self.start_objective_creation(state.outcome_type, Some(state.action_index));
                    } else if let Some(choice) = choices.get(state.selection) {
                        let objective_id = choice.id.clone();
                        self.toggle_action_objective(
                            state.outcome_type,
                            state.action_index,
                            &objective_id,
                        )?;

                        let updated_len = self.objective_choices(state.outcome_type).len();
                        if state.selection >= updated_len {
                            state.selection = updated_len;
                        }
                    }
                }
                KeyCode::Char('n') => {
                    self.start_objective_creation(state.outcome_type, Some(state.action_index));
                }
                KeyCode::Char('r') => {
                    if let Some(choice) = choices.get(state.selection) {
                        self.start_objective_rename(state.outcome_type, choice.id.clone());
                    }
                }
                KeyCode::Char('d') => {
                    if let Some(choice) = choices.get(state.selection) {
                        self.delete_objective(choice.storage_index, &choice.id)?;
                        let updated_len = self.objective_choices(state.outcome_type).len();
                        if state.selection >= updated_len {
                            state.selection = updated_len;
                        }
                    }
                }
                _ => {}
            }

            if self.modal.is_some() {
                self.modal = Some(ModalState::ObjectivePicker(state));
            }

            return Ok(true);
        }

        if let Some(ModalState::IndicatorUpdate(mut state)) = self.modal.clone() {
            match key {
                KeyCode::Esc => {
                    self.modal = None;
                    return Ok(true);
                }
                KeyCode::Enter => {
                    self.apply_indicator_update(&state)?;
                    self.modal = None;
                    return Ok(true);
                }
                KeyCode::Backspace => {
                    state.buffer.pop();
                }
                KeyCode::Char(ch) => {
                    match ch {
                        '+' | '=' => {
                            let delta = Self::indicator_fine_delta(&state);
                            if delta > 0.0 {
                                state.buffer =
                                    Self::adjust_buffer_value(&state.buffer, &state.unit, delta);
                            }
                        }
                        '-' | '_' => {
                            let delta = Self::indicator_fine_delta(&state);
                            if delta > 0.0 {
                                state.buffer =
                                    Self::adjust_buffer_value(&state.buffer, &state.unit, -delta);
                            }
                        }
                        _ => {
                            let lower = ch.to_ascii_lowercase();
                            match lower {
                                'c' => state.buffer.clear(),
                                'a' => match state.indicator_type {
                                    IndicatorType::Percentage => {
                                        state.buffer =
                                            Self::format_value_for_unit(25.0, &state.unit);
                                    }
                                    IndicatorType::Boolean => {}
                                    _ => {
                                        let delta = Self::indicator_small_delta(&state);
                                        if delta > 0.0 {
                                            state.buffer = Self::adjust_buffer_value(
                                                &state.buffer,
                                                &state.unit,
                                                delta,
                                            );
                                        }
                                    }
                                },
                                's' => match state.indicator_type {
                                    IndicatorType::Percentage => {
                                        state.buffer =
                                            Self::format_value_for_unit(50.0, &state.unit);
                                    }
                                    IndicatorType::Boolean => {}
                                    _ => {
                                        let delta = Self::indicator_medium_delta(&state);
                                        if delta > 0.0 {
                                            state.buffer = Self::adjust_buffer_value(
                                                &state.buffer,
                                                &state.unit,
                                                delta,
                                            );
                                        }
                                    }
                                },
                                'd' => match state.indicator_type {
                                    IndicatorType::Percentage => {
                                        state.buffer =
                                            Self::format_value_for_unit(75.0, &state.unit);
                                    }
                                    IndicatorType::Boolean => {}
                                    _ => {
                                        let delta = Self::indicator_large_delta(&state);
                                        if delta > 0.0 {
                                            state.buffer = Self::adjust_buffer_value(
                                                &state.buffer,
                                                &state.unit,
                                                delta,
                                            );
                                        }
                                    }
                                },
                                'f' => {
                                    if state.indicator_type == IndicatorType::Percentage {
                                        state.buffer =
                                            Self::format_value_for_unit(100.0, &state.unit);
                                    }
                                }
                                'y' => {
                                    if state.indicator_type == IndicatorType::Boolean {
                                        state.buffer =
                                            Self::format_value_for_unit(1.0, &state.unit);
                                    }
                                }
                                'n' => {
                                    if state.indicator_type == IndicatorType::Boolean {
                                        state.buffer =
                                            Self::format_value_for_unit(0.0, &state.unit);
                                    }
                                }
                                _ => {
                                    if lower.is_ascii_digit() || lower == '.' {
                                        if lower == '.' && state.buffer.contains('.') {
                                            // ignore duplicate decimal
                                        } else {
                                            state.buffer.push(ch);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            if self.modal.is_some() {
                self.modal = Some(ModalState::IndicatorUpdate(state));
            }

            return Ok(true);
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
                let total = self.get_selected_outcome().actions.len();
                if total == 0 {
                    self.selected_action = 0;
                } else if self.selected_action == 0 {
                    self.selected_action = total - 1;
                } else {
                    self.selected_action -= 1;
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
                let total = self.get_selected_outcome().actions.len();
                if total == 0 {
                    self.selected_action = 0;
                } else {
                    self.selected_action = (self.selected_action + 1) % total;
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
                    // Silent save - no popup notification
                }
                Err(e) => {
                    self.error_display
                        .show_error(format!("Failed to save: {}", e));
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
        self.get_outcome_by_type_mut(self.selected_outcome)
    }

    fn get_selected_outcome(&self) -> &crate::models::Outcome {
        self.get_outcome_by_type(self.selected_outcome)
    }

    fn current_action_indicator_ids(&self) -> Vec<String> {
        let outcome = self.get_selected_outcome();
        if self.selected_action >= outcome.actions.len() {
            return Vec::new();
        }

        let action = &outcome.actions[self.selected_action];
        let mut ids = Vec::new();

        for objective_id in action.get_all_objective_ids() {
            if let Some(objective) = self
                .objectives
                .objectives
                .iter()
                .find(|obj| obj.id == objective_id)
            {
                for indicator_id in &objective.indicators {
                    if self
                        .indicators
                        .indicators
                        .iter()
                        .any(|indicator| indicator.id == *indicator_id)
                    {
                        ids.push(indicator_id.clone());
                    }
                }
            }
        }

        ids
    }

    fn open_selected_indicator_update(&mut self) -> anyhow::Result<()> {
        if self.focus_panel != FocusPanel::Actions {
            return Ok(());
        }

        // Open indicator update for the first indicator linked to the current action
        let indicator_ids = self.current_action_indicator_ids();
        if let Some(indicator_id) = indicator_ids.first() {
            self.open_indicator_update_modal(indicator_id)?;
        } else {
            self.error_display
                .show_info("No indicators linked to this action".to_string());
        }

        Ok(())
    }

    fn open_indicator_update_modal(&mut self, indicator_id: &str) -> anyhow::Result<()> {
        let indicator_def = if let Some(def) = self
            .indicators
            .indicators
            .iter()
            .find(|indicator| indicator.id == indicator_id)
        {
            def.clone()
        } else {
            self.error_display
                .show_error("Indicator definition not found".to_string());
            return Ok(());
        };

        let indicator_type = self
            .indicators_map
            .get(indicator_id)
            .map(|indicator| indicator.indicator_type)
            .unwrap_or_else(|| Self::infer_indicator_type(&indicator_def.unit));

        let (history, latest, previous, last_updated) =
            self.collect_indicator_history(indicator_id)?;

        let buffer_source = latest.or(previous).unwrap_or(0.0);
        let buffer = Self::format_value_for_unit(buffer_source, &indicator_def.unit);

        let state = IndicatorUpdateState {
            indicator_id: indicator_def.id.clone(),
            name: indicator_def.name.clone(),
            unit: indicator_def.unit.clone(),
            indicator_type,
            direction: indicator_def.direction.clone(),
            target: indicator_def.target,
            previous_value: previous,
            latest_value: latest,
            history,
            last_updated,
            buffer,
        };

        self.modal = Some(ModalState::IndicatorUpdate(state));
        Ok(())
    }

    fn collect_indicator_history(
        &self,
        indicator_id: &str,
    ) -> anyhow::Result<(
        Vec<f64>,
        Option<f64>,
        Option<f64>,
        Option<chrono::NaiveDate>,
    )> {
        let today = chrono::Local::now().date_naive();
        let mut observations = crate::data::read_observations_range(
            today - chrono::Duration::days(60),
            today,
            &self.config,
        )?;

        observations.sort_by(|a, b| a.when.cmp(&b.when).then(a.created.cmp(&b.created)));

        let mut history_values = Vec::new();
        let mut history_dates = Vec::new();
        for obs in observations
            .into_iter()
            .filter(|obs| obs.indicator_id == indicator_id)
        {
            history_values.push(obs.value);
            history_dates.push(obs.when);
        }

        let fallback_current = self
            .indicators_map
            .get(indicator_id)
            .map(|indicator| indicator.current_value);

        let mut last_updated = history_dates.last().copied();
        if history_values.is_empty() {
            if let Some(current) = fallback_current {
                history_values.push(current);
                last_updated = Some(today);
            }
        }

        let latest = history_values.last().copied().or(fallback_current);
        let previous = if history_values.len() >= 2 {
            history_values.get(history_values.len() - 2).copied()
        } else {
            None
        };

        let start = history_values.len().saturating_sub(7);
        let history_window = history_values[start..].to_vec();

        Ok((history_window, latest, previous, last_updated))
    }

    fn infer_indicator_type(unit: &IndicatorUnit) -> IndicatorType {
        match unit {
            IndicatorUnit::Minutes => IndicatorType::Duration,
            IndicatorUnit::Percent => IndicatorType::Percentage,
            IndicatorUnit::Custom(label) if label.eq_ignore_ascii_case("boolean") => {
                IndicatorType::Boolean
            }
            IndicatorUnit::Custom(label) if label.eq_ignore_ascii_case("hours") => {
                IndicatorType::Duration
            }
            IndicatorUnit::Custom(label) if label.eq_ignore_ascii_case("percentage") => {
                IndicatorType::Percentage
            }
            _ => IndicatorType::Counter,
        }
    }

    fn format_value_with_unit(value: f64, unit: &IndicatorUnit) -> String {
        let base = Self::format_value_for_unit(value, unit);
        match unit {
            IndicatorUnit::Percent => format!("{}%", base),
            _ => {
                let label = Self::unit_label(unit);
                if label.is_empty() {
                    base
                } else {
                    format!("{} {}", base, label)
                }
            }
        }
    }

    fn indicator_delta_threshold(indicator_type: IndicatorType) -> f64 {
        match indicator_type {
            IndicatorType::Counter => 0.5,
            IndicatorType::Duration => 0.1,
            IndicatorType::Percentage => 0.5,
            IndicatorType::Boolean => 0.5,
        }
    }

    fn indicator_delta_label(state: &IndicatorUpdateState) -> Option<String> {
        let target = state.target?;
        let latest = state.latest_value?;
        let threshold = Self::indicator_delta_threshold(state.indicator_type);
        let diff = latest - target;
        if diff.abs() < threshold {
            return Some("on target".to_string());
        }

        let magnitude = Self::format_value_with_unit(diff.abs(), &state.unit);
        if diff > 0.0 {
            Some(format!("(+{} ahead)", magnitude))
        } else {
            Some(format!("(-{} behind)", magnitude))
        }
    }

    fn indicator_trend_status(state: &IndicatorUpdateState) -> TrendStatus {
        match (state.latest_value, state.previous_value) {
            (Some(latest), Some(previous)) => {
                let diff = latest - previous;
                let threshold = Self::indicator_delta_threshold(state.indicator_type);
                if diff > threshold {
                    TrendStatus::Improving
                } else if diff < -threshold {
                    TrendStatus::Declining
                } else {
                    TrendStatus::Stable
                }
            }
            _ => TrendStatus::Stable,
        }
    }

    fn indicator_trend_display(status: TrendStatus) -> (&'static str, &'static str) {
        match status {
            TrendStatus::Improving => ("▲", "Improving"),
            TrendStatus::Declining => ("▼", "Declining"),
            TrendStatus::Stable => ("■", "Stable"),
        }
    }

    fn trend_color(&self, status: TrendStatus) -> Color {
        match status {
            TrendStatus::Improving => self.theme.completed,
            TrendStatus::Declining => self.theme.pending,
            TrendStatus::Stable => self.theme.partial,
        }
    }

    fn indicator_quick_actions_text(state: &IndicatorUpdateState) -> String {
        match state.indicator_type {
            IndicatorType::Counter => "+/- fine   a +1   s +3   d +5   c clear".to_string(),
            IndicatorType::Duration => {
                if Self::is_hours_unit(&state.unit) {
                    "+/- fine   a +0.5h   s +1h   d +2h   c reset".to_string()
                } else {
                    "+/- fine   a +30m   s +60m   d +120m   c reset".to_string()
                }
            }
            IndicatorType::Percentage => {
                "+/- fine   a 25%   s 50%   d 75%   f 100%   c clear".to_string()
            }
            IndicatorType::Boolean => "y complete   n incomplete".to_string(),
        }
    }

    fn is_hours_unit(unit: &IndicatorUnit) -> bool {
        match unit {
            IndicatorUnit::Custom(label) => label.to_lowercase().contains("hour"),
            _ => false,
        }
    }

    fn indicator_fine_delta(state: &IndicatorUpdateState) -> f64 {
        match state.indicator_type {
            IndicatorType::Counter => 1.0,
            IndicatorType::Duration => {
                if Self::is_hours_unit(&state.unit) {
                    0.25
                } else {
                    15.0
                }
            }
            IndicatorType::Percentage => 5.0,
            IndicatorType::Boolean => 0.0,
        }
    }

    fn indicator_small_delta(state: &IndicatorUpdateState) -> f64 {
        match state.indicator_type {
            IndicatorType::Counter => 1.0,
            IndicatorType::Duration => {
                if Self::is_hours_unit(&state.unit) {
                    0.5
                } else {
                    30.0
                }
            }
            IndicatorType::Percentage => 0.0,
            IndicatorType::Boolean => 0.0,
        }
    }

    fn indicator_medium_delta(state: &IndicatorUpdateState) -> f64 {
        match state.indicator_type {
            IndicatorType::Counter => 3.0,
            IndicatorType::Duration => {
                if Self::is_hours_unit(&state.unit) {
                    1.0
                } else {
                    60.0
                }
            }
            IndicatorType::Percentage => 0.0,
            IndicatorType::Boolean => 0.0,
        }
    }

    fn indicator_large_delta(state: &IndicatorUpdateState) -> f64 {
        match state.indicator_type {
            IndicatorType::Counter => 5.0,
            IndicatorType::Duration => {
                if Self::is_hours_unit(&state.unit) {
                    2.0
                } else {
                    120.0
                }
            }
            IndicatorType::Percentage => 0.0,
            IndicatorType::Boolean => 0.0,
        }
    }

    fn clamp_text(text: &str, width: usize) -> String {
        if width == 0 {
            return String::new();
        }
        let count = text.chars().count();
        if count <= width {
            return text.to_string();
        }
        if width == 1 {
            return "…".to_string();
        }
        let truncated: String = text.chars().take(width - 1).collect();
        format!("{}…", truncated)
    }

    fn render_indicator_update_modal(&self, f: &mut Frame, state: &IndicatorUpdateState) {
        let area = centered_rect(70, 65, f.area());
        f.render_widget(Clear, area);

        let border_set = border::Set {
            top_left: "/",
            top_right: "\\",
            bottom_left: "\\",
            bottom_right: "/",
            vertical_left: "|",
            vertical_right: "|",
            horizontal_top: "-",
            horizontal_bottom: "-",
        };

        let shell = Block::default()
            .borders(Borders::ALL)
            .border_set(border_set)
            .border_style(Style::default().fg(self.theme.header))
            .style(Style::default().bg(self.theme.panel_bg));
        f.render_widget(shell, area);

        if area.width < 6 || area.height < 10 {
            return;
        }

        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        let header_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(6),
            ])
            .split(inner);

        let title_line = Line::from(vec![Span::styled(
            " FocusFive · Update Indicator ",
            Style::default().fg(self.theme.header),
        )]);
        let title = Paragraph::new(title_line)
            .alignment(Alignment::Left)
            .style(Style::default().bg(self.theme.panel_bg));
        f.render_widget(title, header_layout[0]);

        let metrics_text = format!(
            " Target {} | Latest {} | Previous {} ",
            state
                .target
                .map(|value| Self::format_value_with_unit(value, &state.unit))
                .unwrap_or_else(|| "—".to_string()),
            state
                .latest_value
                .map(|value| Self::format_value_with_unit(value, &state.unit))
                .unwrap_or_else(|| "—".to_string()),
            state
                .previous_value
                .map(|value| Self::format_value_with_unit(value, &state.unit))
                .unwrap_or_else(|| "—".to_string()),
        );

        let inner_width = header_layout[1].width.saturating_sub(2) as usize;
        let clamped = Self::clamp_text(&metrics_text, inner_width);
        let padding = inner_width.saturating_sub(clamped.chars().count());
        let left_pad = padding / 2;
        let right_pad = padding.saturating_sub(left_pad);

        let mut spans = Vec::new();
        spans.push(Span::styled("\\", Style::default().fg(self.theme.header)));
        if left_pad > 0 {
            spans.push(Span::styled(
                "-".repeat(left_pad),
                Style::default().fg(self.theme.border),
            ));
        }
        spans.push(Span::styled(
            clamped,
            Style::default().fg(self.theme.text_secondary),
        ));
        if right_pad > 0 {
            spans.push(Span::styled(
                "-".repeat(right_pad),
                Style::default().fg(self.theme.border),
            ));
        }
        spans.push(Span::styled("/", Style::default().fg(self.theme.header)));
        let metrics =
            Paragraph::new(Line::from(spans)).style(Style::default().bg(self.theme.panel_bg));
        f.render_widget(metrics, header_layout[1]);

        let body_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(6),
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(header_layout[2]);

        if body_layout.len() < 6 {
            return;
        }

        let summary_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(body_layout[0]);

        let current_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border))
            .style(Style::default().bg(self.theme.background));
        f.render_widget(current_block, summary_chunks[0]);

        let current_inner = Rect {
            x: summary_chunks[0].x + 1,
            y: summary_chunks[0].y + 1,
            width: summary_chunks[0].width.saturating_sub(2),
            height: summary_chunks[0].height.saturating_sub(2),
        };

        let current_value = state
            .latest_value
            .map(|value| Self::format_value_with_unit(value, &state.unit))
            .unwrap_or_else(|| "—".to_string());

        let mut current_line = vec![
            Span::styled("Current ", Style::default().fg(self.theme.text_secondary)),
            Span::styled(
                current_value,
                Style::default()
                    .fg(self.theme.text_primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ];

        if let Some(delta) = Self::indicator_delta_label(state) {
            current_line.push(Span::raw(" "));
            current_line.push(Span::styled(delta, Style::default().fg(self.theme.partial)));
        }

        let current_paragraph = Paragraph::new(Line::from(current_line))
            .alignment(Alignment::Center)
            .style(Style::default().bg(self.theme.background));
        f.render_widget(current_paragraph, current_inner);

        let direction_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border))
            .style(Style::default().bg(self.theme.background));
        f.render_widget(direction_block, summary_chunks[1]);

        let direction_inner = Rect {
            x: summary_chunks[1].x + 1,
            y: summary_chunks[1].y + 1,
            width: summary_chunks[1].width.saturating_sub(2),
            height: summary_chunks[1].height.saturating_sub(2),
        };

        let direction_text = match state.direction {
            IndicatorDirection::HigherIsBetter => "Higher is better",
            IndicatorDirection::LowerIsBetter => "Lower is better",
            IndicatorDirection::WithinRange => "Target range",
        };

        let trend_status = Self::indicator_trend_status(state);
        let (trend_icon, trend_label) = Self::indicator_trend_display(trend_status);
        let trend_color = self.trend_color(trend_status);

        let direction_paragraph = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Direction ", Style::default().fg(self.theme.text_secondary)),
                Span::styled(direction_text, Style::default().fg(self.theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled(trend_icon, Style::default().fg(trend_color)),
                Span::raw(" "),
                Span::styled(trend_label, Style::default().fg(self.theme.text_secondary)),
            ]),
        ])
        .alignment(Alignment::Center)
        .style(Style::default().bg(self.theme.background));
        f.render_widget(direction_paragraph, direction_inner);

        let progress_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
            .split(body_layout[1]);

        let percent_value = state
            .target
            .filter(|target| *target > 0.0)
            .and_then(|target| state.latest_value.map(|current| (current / target) * 100.0));
        let gauge_percent = percent_value.unwrap_or(0.0).clamp(0.0, 100.0) as u16;
        let gauge_color = match gauge_percent {
            100.. => self.theme.completed,
            70..=99 => self.theme.partial,
            _ => self.theme.pending,
        };
        let gauge_label = percent_value
            .map(|value| format!("{:.0}%", value.clamp(0.0, 999.0)))
            .unwrap_or_else(|| "—".to_string());

        let gauge = Gauge::default()
            .percent(gauge_percent)
            .label(gauge_label)
            .gauge_style(Style::default().fg(gauge_color).bg(self.theme.background))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Goal Pace ")
                    .border_style(Style::default().fg(self.theme.border))
                    .style(Style::default().bg(self.theme.background)),
            );
        f.render_widget(gauge, progress_chunks[0]);

        let trend_line = Paragraph::new(Line::from(vec![
            Span::styled(trend_icon, Style::default().fg(trend_color)),
            Span::raw(" "),
            Span::styled(trend_label, Style::default().fg(self.theme.text_secondary)),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().bg(self.theme.panel_bg));
        f.render_widget(trend_line, progress_chunks[1]);

        let history_block = Block::default()
            .borders(Borders::ALL)
            .title(" 7-Day History ")
            .border_style(Style::default().fg(self.theme.border))
            .style(Style::default().bg(self.theme.background));
        f.render_widget(history_block, body_layout[2]);

        let history_inner = Rect {
            x: body_layout[2].x + 1,
            y: body_layout[2].y + 1,
            width: body_layout[2].width.saturating_sub(2),
            height: body_layout[2].height.saturating_sub(2),
        };

        if history_inner.height == 0 {
            return;
        }

        let history_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(history_inner);

        if history_layout.len() < 2 {
            return;
        }

        if !state.history.is_empty() {
            let data: Vec<u64> = state
                .history
                .iter()
                .map(|value| (value.max(0.0) * 100.0) as u64)
                .collect();
            let sparkline = Sparkline::default()
                .data(&data)
                .style(Style::default().fg(self.theme.partial));
            f.render_widget(sparkline, history_layout[0]);
        } else {
            let placeholder = Paragraph::new("No history yet")
                .alignment(Alignment::Center)
                .style(
                    Style::default()
                        .fg(self.theme.text_secondary)
                        .bg(self.theme.background),
                );
            f.render_widget(placeholder, history_layout[0]);
        }

        let start_value = state
            .history
            .first()
            .copied()
            .map(|value| Self::format_value_with_unit(value, &state.unit))
            .unwrap_or_else(|| "—".to_string());
        let end_value = state
            .history
            .last()
            .copied()
            .map(|value| Self::format_value_with_unit(value, &state.unit))
            .unwrap_or_else(|| "—".to_string());
        let last_update = state
            .last_updated
            .map(|date| date.to_string())
            .unwrap_or_else(|| "—".to_string());

        let footer = format!(
            "Start {}   End {}   Last update {}",
            start_value, end_value, last_update
        );
        let footer_paragraph = Paragraph::new(footer).alignment(Alignment::Center).style(
            Style::default()
                .fg(self.theme.text_secondary)
                .bg(self.theme.background),
        );
        f.render_widget(footer_paragraph, history_layout[1]);

        let quick_actions = Paragraph::new(Self::indicator_quick_actions_text(state))
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(self.theme.text_secondary)
                    .bg(self.theme.panel_bg),
            );
        f.render_widget(quick_actions, body_layout[3]);

        let unit_label = Self::unit_label(&state.unit);
        let input_display = format!("[ {} ] {}", state.buffer, unit_label);
        let input_paragraph = Paragraph::new(input_display)
            .alignment(Alignment::Center)
            .style(Style::default().fg(self.theme.text_primary))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Input Value ")
                    .border_style(Style::default().fg(self.theme.border))
                    .style(Style::default().bg(self.theme.background)),
            );
        f.render_widget(input_paragraph, body_layout[4]);

        let helper_footer = Paragraph::new(Line::from(vec![
            Span::styled("Enter", Style::default().fg(self.theme.header)),
            Span::raw(" Save  "),
            Span::styled("Backspace", Style::default().fg(self.theme.header)),
            Span::raw(" Delete  "),
            Span::styled("Esc", Style::default().fg(self.theme.header)),
            Span::raw(" Cancel"),
        ]))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(self.theme.text_secondary)
                .bg(self.theme.panel_bg),
        );
        f.render_widget(helper_footer, body_layout[5]);
    }

    fn parse_value_from_buffer(buffer: &str, unit: &IndicatorUnit) -> anyhow::Result<f64> {
        let value = buffer.trim().parse::<f64>()?;
        Ok(Self::clamp_value_for_unit(value, unit))
    }

    fn clamp_value_for_unit(value: f64, unit: &IndicatorUnit) -> f64 {
        match unit {
            IndicatorUnit::Percent => value.clamp(0.0, 100.0),
            _ => value.max(0.0),
        }
    }

    fn format_value_for_unit(value: f64, unit: &IndicatorUnit) -> String {
        match unit {
            IndicatorUnit::Percent => format!("{:.0}", value),
            IndicatorUnit::Count => format!("{:.0}", value),
            IndicatorUnit::Minutes | IndicatorUnit::Dollars | IndicatorUnit::Custom(_) => {
                if value.fract().abs() < f64::EPSILON {
                    format!("{:.0}", value)
                } else {
                    format!("{:.2}", value)
                }
            }
        }
    }

    fn unit_label(unit: &IndicatorUnit) -> String {
        match unit {
            IndicatorUnit::Count => "count".to_string(),
            IndicatorUnit::Minutes => "minutes".to_string(),
            IndicatorUnit::Dollars => "dollars".to_string(),
            IndicatorUnit::Percent => "%".to_string(),
            IndicatorUnit::Custom(label) => label.clone(),
        }
    }

    fn adjust_buffer_value(buffer: &str, unit: &IndicatorUnit, delta: f64) -> String {
        let current = buffer.trim().parse::<f64>().unwrap_or(0.0);
        let adjusted = Self::clamp_value_for_unit(current + delta, unit);
        Self::format_value_for_unit(adjusted, unit)
    }

    fn apply_indicator_update(&mut self, state: &IndicatorUpdateState) -> anyhow::Result<()> {
        if state.buffer.trim().is_empty() {
            self.error_display
                .show_error("Enter a value before saving".to_string());
            return Ok(());
        }

        let value = match Self::parse_value_from_buffer(&state.buffer, &state.unit) {
            Ok(value) => value,
            Err(err) => {
                self.error_display
                    .show_error(format!("Invalid indicator value: {}", err));
                return Ok(());
            }
        };

        let observation = Observation {
            id: Uuid::new_v4().to_string(),
            indicator_id: state.indicator_id.clone(),
            when: chrono::Local::now().date_naive(),
            value,
            unit: state.unit.clone(),
            source: ObservationSource::Manual,
            action_id: None,
            note: None,
            created: chrono::Utc::now(),
        };

        crate::data::append_observation(&observation, &self.config)?;

        if let Some(indicator) = self
            .indicators
            .indicators
            .iter_mut()
            .find(|def| def.id == state.indicator_id)
        {
            indicator.modified = chrono::Utc::now();
        }

        if let Err(err) = crate::data::save_indicators(&self.indicators, &self.config) {
            self.error_display
                .show_error(format!("Failed to update indicators: {}", err));
        }

        if let Some(indicator) = self.indicators_map.get_mut(&state.indicator_id) {
            indicator.current_value = value;
            indicator.history.push(crate::models::IndicatorEntry {
                timestamp: chrono::Utc::now(),
                value,
                note: None,
            });
        }

        self.error_display
            .show_info("Indicator value recorded".to_string());

        // Refresh dashboard cursor bounds
        if !self.dashboard_signal_ids.is_empty() {
            self.dashboard_signal_index = self
                .dashboard_signal_index
                .min(self.dashboard_signal_ids.len().saturating_sub(1));
        }

        Ok(())
    }

    fn get_outcome_by_type(&self, outcome_type: OutcomeType) -> &crate::models::Outcome {
        match outcome_type {
            OutcomeType::Work => &self.goals.work,
            OutcomeType::Health => &self.goals.health,
            OutcomeType::Family => &self.goals.family,
        }
    }

    fn get_outcome_by_type_mut(
        &mut self,
        outcome_type: OutcomeType,
    ) -> &mut crate::models::Outcome {
        match outcome_type {
            OutcomeType::Work => &mut self.goals.work,
            OutcomeType::Health => &mut self.goals.health,
            OutcomeType::Family => &mut self.goals.family,
        }
    }

    fn open_editor(&mut self) {
        // Only allow editing when focused on Actions panel
        if self.focus_panel == FocusPanel::Actions {
            let action_text = {
                let outcome = self.get_selected_outcome();
                outcome.actions[self.selected_action].text.clone()
            };
            self.text_editor.activate_with(
                "Edit Action",
                &action_text,
                crate::models::MAX_ACTION_LENGTH,
            );
            self.editor_context = Some(EditorContext::Action {
                outcome_type: self.selected_outcome,
                index: self.selected_action,
            });
        }
    }

    fn open_vision_editor(&mut self) {
        if self.focus_panel != FocusPanel::Outcomes {
            self.focus_panel = FocusPanel::Outcomes;
        }
        let outcome_type = self.selected_outcome;
        let vision_text = self.vision.get_vision(&outcome_type).to_string();
        self.text_editor.activate_with(
            "Edit 5-Year Vision",
            &vision_text,
            crate::models::MAX_VISION_LENGTH,
        );
        self.editor_context = Some(EditorContext::Vision { outcome_type });
    }

    fn open_objective_picker(&mut self) {
        if self.focus_panel != FocusPanel::Actions {
            self.focus_panel = FocusPanel::Actions;
        }

        let outcome_type = self.selected_outcome;
        let action_index = self.selected_action;

        self.modal = Some(ModalState::ObjectivePicker(ObjectiveModalState {
            outcome_type,
            action_index,
            selection: 0,
        }));
    }

    fn objective_choices(&self, outcome_type: OutcomeType) -> Vec<ObjectiveChoice> {
        self.objectives
            .objectives
            .iter()
            .enumerate()
            .filter(|(_, obj)| obj.domain == outcome_type)
            .map(|(index, obj)| ObjectiveChoice {
                storage_index: index,
                id: obj.id.clone(),
                title: obj.title.clone(),
                status: obj.status.clone(),
            })
            .collect()
    }

    fn objective_index_in_domain(
        &self,
        outcome_type: OutcomeType,
        objective_id: &str,
    ) -> Option<usize> {
        let mut index = 0;
        for obj in self
            .objectives
            .objectives
            .iter()
            .filter(|o| o.domain == outcome_type)
        {
            if obj.id == objective_id {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    fn start_objective_creation(&mut self, outcome_type: OutcomeType, link_action: Option<usize>) {
        self.text_editor
            .activate_with("Create Objective", "", crate::models::MAX_GOAL_LENGTH);
        self.editor_context = Some(EditorContext::ObjectiveTitle {
            outcome_type,
            objective_id: None,
            link_action,
        });
    }

    fn start_objective_rename(&mut self, outcome_type: OutcomeType, objective_id: String) {
        if let Some(objective) = self
            .objectives
            .objectives
            .iter()
            .find(|o| o.id == objective_id)
        {
            self.text_editor.activate_with(
                "Rename Objective",
                &objective.title,
                crate::models::MAX_GOAL_LENGTH,
            );
            self.editor_context = Some(EditorContext::ObjectiveTitle {
                outcome_type,
                objective_id: Some(objective.id.clone()),
                link_action: None,
            });
        }
    }

    fn delete_objective(&mut self, storage_index: usize, objective_id: &str) -> anyhow::Result<()> {
        if storage_index >= self.objectives.objectives.len() {
            return Ok(());
        }

        let backup_objectives = self.objectives.clone();
        let backup_goals = self.goals.clone();

        self.objectives.objectives.remove(storage_index);

        // Remove objective references from all actions
        for outcome in [
            &mut self.goals.work,
            &mut self.goals.health,
            &mut self.goals.family,
        ] {
            for action in &mut outcome.actions {
                action.remove_objective_id(objective_id);
            }
        }

        if let Err(e) = crate::data::save_objectives(&self.objectives, &self.config) {
            self.error_display
                .show_error(format!("Failed to save objectives: {}", e));
            self.objectives = backup_objectives;
            self.goals = backup_goals;
            return Err(e);
        }

        if let Err(e) = crate::data::write_goals_file(&self.goals, &self.config) {
            self.error_display
                .show_error(format!("Failed to update goals: {}", e));
            self.objectives = backup_objectives;
            self.goals = backup_goals;
            return Err(e);
        }

        Ok(())
    }

    fn toggle_action_objective(
        &mut self,
        outcome_type: OutcomeType,
        action_index: usize,
        objective_id: &str,
    ) -> anyhow::Result<()> {
        let outcome = self.get_outcome_by_type_mut(outcome_type);
        if action_index >= outcome.actions.len() {
            self.error_display
                .show_error("Invalid action selection".to_string());
            return Ok(());
        }

        let action = &mut outcome.actions[action_index];
        let already_linked = action
            .get_all_objective_ids()
            .iter()
            .any(|id| id == objective_id);

        if already_linked {
            action.remove_objective_id(objective_id);
        } else {
            action.add_objective_id(objective_id.to_string());
        }

        let backup_goals = self.goals.clone();
        if let Err(e) = crate::data::write_goals_file(&self.goals, &self.config) {
            self.goals = backup_goals;
            return Err(e);
        }
        Ok(())
    }

    fn link_action_to_objective(
        &mut self,
        outcome_type: OutcomeType,
        action_index: usize,
        objective_id: &str,
    ) -> anyhow::Result<()> {
        let outcome = self.get_outcome_by_type_mut(outcome_type);
        if action_index >= outcome.actions.len() {
            self.error_display
                .show_error("Invalid action selection".to_string());
            return Ok(());
        }

        let action = &mut outcome.actions[action_index];
        if !action
            .get_all_objective_ids()
            .iter()
            .any(|id| id == objective_id)
        {
            action.add_objective_id(objective_id.to_string());
            let backup_goals = self.goals.clone();
            if let Err(e) = crate::data::write_goals_file(&self.goals, &self.config) {
                self.goals = backup_goals;
                return Err(e);
            }
        }

        Ok(())
    }

    fn toggle_expansion(&mut self) {
        // Toggle expansion of current action when in Actions panel
        if self.focus_panel == FocusPanel::Actions {
            let action_id = {
                let outcome = self.get_selected_outcome();
                outcome.actions[self.selected_action].id.clone()
            };
            self.ui_state.toggle_expansion(action_id);
        }
    }

    pub fn render(&mut self, f: &mut Frame) {
        if self.show_dashboard {
            self.render_dashboard(f);
            return;
        }

        // Clear background
        f.render_widget(
            Block::default().style(Style::default().bg(self.theme.background)),
            f.area(),
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

        self.render_modal(f);
    }

    fn render_dashboard(&mut self, f: &mut Frame) {
        f.render_widget(
            Block::default().style(Style::default().bg(self.financial_theme.bg_primary)),
            f.area(),
        );

        let layout = DashboardLayout::new(f.area());

        self.render_dashboard_header(f, layout.header);
        self.render_dashboard_live_metrics(f, layout.live_metrics);
        self.render_dashboard_performance(f, layout.performance);
        self.render_dashboard_sentiment(f, layout.sentiment);
        self.render_dashboard_signals(f, layout.signals);
        self.render_dashboard_status_line(f, layout.status_line);
        self.render_dashboard_footer(f, layout.footer);

        if self.error_display.is_active() {
            self.error_display.render(f, f.area(), &self.theme);
        }

        self.render_modal(f);
    }

    fn render_modal(&self, f: &mut Frame) {
        match self.modal {
            Some(ModalState::ObjectivePicker(state)) => {
                let area = centered_rect(60, 60, f.area());
                f.render_widget(Clear, area);

                let choices = self.objective_choices(state.outcome_type);
                let outcome = self.get_outcome_by_type(state.outcome_type);
                let action_title = outcome
                    .actions
                    .get(state.action_index)
                    .map(|a| a.text.clone())
                    .unwrap_or_else(|| "(unknown action)".to_string());
                let linked_ids = outcome
                    .actions
                    .get(state.action_index)
                    .map(|a| a.get_all_objective_ids())
                    .unwrap_or_default();

                let mut items: Vec<ListItem> = choices
                    .iter()
                    .map(|choice| {
                        let linked = linked_ids.iter().any(|id| id == &choice.id);
                        let status_icon = match choice.status {
                            ObjectiveStatus::Active => "●",
                            ObjectiveStatus::Paused => "⏸",
                            ObjectiveStatus::Completed => "✓",
                            ObjectiveStatus::Dropped => "✗",
                        };

                        ListItem::new(Line::from(vec![
                            Span::styled(
                                if linked { "[x] " } else { "[ ] " },
                                Style::default().fg(self.theme.text_secondary),
                            ),
                            Span::styled(
                                format!("{} ", status_icon),
                                Style::default().fg(self.theme.header),
                            ),
                            Span::styled(
                                &choice.title,
                                Style::default().fg(self.theme.text_primary),
                            ),
                            Span::raw("  "),
                            Span::styled(
                                &choice.id[..8.min(choice.id.len())],
                                Style::default().fg(self.theme.text_secondary),
                            ),
                        ]))
                    })
                    .collect();

                items.push(ListItem::new(Line::from(vec![Span::styled(
                    "➕ Create New Objective",
                    Style::default()
                        .fg(self.theme.header)
                        .add_modifier(Modifier::BOLD),
                )])));

                let mut list_state = ListState::default();
                list_state.select(Some(state.selection.min(items.len().saturating_sub(1))));

                let block = Block::default()
                    .title(format!(
                        " Objectives for {:?} • Action: {} ",
                        state.outcome_type, action_title
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.header))
                    .style(Style::default().bg(self.theme.panel_bg));

                let list = List::new(items)
                    .block(block)
                    .highlight_style(
                        Style::default()
                            .fg(self.theme.header)
                            .bg(self.theme.border)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol("➤ ");

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(4), Constraint::Length(2)])
                    .split(area);

                f.render_stateful_widget(list, layout[0], &mut list_state);

                let help_text = Paragraph::new(Line::from(vec![
                    Span::styled("↑/↓", Style::default().fg(self.theme.header)),
                    Span::raw(" Navigate  "),
                    Span::styled("Enter", Style::default().fg(self.theme.header)),
                    Span::raw(" Link/Unlink  "),
                    Span::styled("n", Style::default().fg(self.theme.header)),
                    Span::raw(" New  "),
                    Span::styled("r", Style::default().fg(self.theme.header)),
                    Span::raw(" Rename  "),
                    Span::styled("d", Style::default().fg(self.theme.header)),
                    Span::raw(" Delete  "),
                    Span::styled("Esc", Style::default().fg(self.theme.header)),
                    Span::raw(" Close"),
                ]))
                .style(Style::default().fg(self.theme.text_secondary))
                .alignment(Alignment::Left);

                f.render_widget(help_text, layout[1]);
            }
            Some(ModalState::IndicatorUpdate(ref state)) => {
                self.render_indicator_update_modal(f, state);
            }
            _ => {}
        }
    }

    fn render_dashboard_header(&self, f: &mut Frame, area: Rect) {
        let now = chrono::Local::now();
        let streak_text = self
            .goals
            .day_number
            .map(|n| format!("Day {}", n))
            .unwrap_or_else(|| "Day --".to_string());

        let header_line = Line::from(vec![
            Span::styled(
                "FOCUSFIVE GOAL TRACKING SYSTEM",
                Style::default()
                    .fg(self.financial_theme.accent_yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  •  "),
            Span::styled(
                now.format("%B %d, %Y  %I:%M %p %Z").to_string(),
                Style::default().fg(self.financial_theme.text_secondary),
            ),
            Span::raw("  •  "),
            Span::styled(
                streak_text,
                Style::default()
                    .fg(self.financial_theme.info)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  •  "),
            Span::styled(
                format!("Focus: {:?}", self.dashboard_focus),
                Style::default().fg(self.financial_theme.text_secondary),
            ),
        ]);

        let header = Paragraph::new(header_line)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.financial_theme.text_dim))
                    .style(Style::default().bg(self.financial_theme.bg_secondary)),
            );

        f.render_widget(header, area);
    }

    fn render_dashboard_live_metrics(&mut self, f: &mut Frame, area: Rect) {
        let today = chrono::Local::now().naive_local().date();
        let observations = crate::data::read_observations_range(
            today - chrono::Duration::days(7),
            today,
            &self.config,
        )
        .unwrap_or_default();

        // Populate market IDs with active indicators
        self.dashboard_market_ids = self.indicators.indicators
            .iter()
            .filter(|ind| ind.active)
            .map(|ind| ind.id.clone())
            .collect();

        // Ensure index is within bounds
        if self.dashboard_market_ids.is_empty() {
            self.dashboard_market_index = 0;
        } else if self.dashboard_market_index >= self.dashboard_market_ids.len() {
            self.dashboard_market_index = self.dashboard_market_ids.len().saturating_sub(1);
        }

        // Use yellow title if this section is active, grey otherwise
        let title_color = if self.dashboard_focus == DashboardPanel::Market {
            self.financial_theme.accent_yellow
        } else {
            self.financial_theme.text_dim
        };

        let widget = LiveMetricsWidget::new(
            &self.indicators.indicators,
            &observations,
            &self.financial_theme,
        )
        .block(
            Block::default()
                .title(" LIVE METRICS ")
                .title_style(
                    Style::default()
                        .fg(title_color)
                        .add_modifier(Modifier::BOLD),
                )
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.financial_theme.text_dim))
                .style(Style::default().bg(self.financial_theme.bg_panel)),
        );

        f.render_widget(widget, area);
    }

    fn render_dashboard_performance(&mut self, f: &mut Frame, area: Rect) {
        let today = chrono::Local::now().naive_local().date();
        let observations = crate::data::read_observations_range(
            today - chrono::Duration::days(7),
            today,
            &self.config,
        )
        .unwrap_or_default();

        let active_indicators: Vec<_> = self
            .indicators
            .indicators
            .iter()
            .filter(|indicator| indicator.active)
            .collect();

        if active_indicators.is_empty() {
            self.dashboard_performance_ids.clear();
            self.dashboard_performance_index = 0;

            // Use yellow title if this section is active, grey otherwise
            let title_color = if self.dashboard_focus == DashboardPanel::Performance {
                self.financial_theme.accent_yellow
            } else {
                self.financial_theme.text_dim
            };

            let placeholder =
                Paragraph::new("PERFORMANCE ANALYTICS\nActivate indicators to see trend charts")
                    .style(
                        Style::default()
                            .fg(self.financial_theme.text_primary)
                            .bg(self.financial_theme.bg_panel),
                    )
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .title(" PERFORMANCE ANALYTICS ")
                            .title_style(
                                Style::default()
                                    .fg(title_color)
                                    .add_modifier(Modifier::BOLD),
                            )
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(self.financial_theme.text_dim))
                            .style(Style::default().bg(self.financial_theme.bg_panel)),
                    );

            f.render_widget(placeholder, area);
            return;
        }

        // Update available performance IDs
        self.dashboard_performance_ids = active_indicators
            .iter()
            .map(|indicator| indicator.id.clone())
            .collect();

        // Ensure index is within bounds
        if self.dashboard_performance_ids.is_empty() {
            self.dashboard_performance_index = 0;
        } else if self.dashboard_performance_index >= self.dashboard_performance_ids.len() {
            self.dashboard_performance_index = self.dashboard_performance_ids.len().saturating_sub(1);
        }

        // Use yellow title if this section is active, grey otherwise
        let title_color = if self.dashboard_focus == DashboardPanel::Performance {
            self.financial_theme.accent_yellow
        } else {
            self.financial_theme.text_dim
        };

        // Define viewport: show 2 charts at a time for better visibility
        let charts_per_page = 2;
        let start_index = if self.dashboard_performance_index >= charts_per_page {
            self.dashboard_performance_index - charts_per_page + 1
        } else {
            0
        }.min(active_indicators.len().saturating_sub(charts_per_page));

        let end_index = (start_index + charts_per_page).min(active_indicators.len());
        let visible_indicators = &active_indicators[start_index..end_index];

        // Create layout for visible charts
        let chart_count = visible_indicators.len();
        let constraints = vec![Constraint::Ratio(1, chart_count as u32); chart_count];
        let chart_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Render visible charts with selection highlighting
        for (i, (indicator, chart_area)) in visible_indicators
            .iter()
            .zip(chart_areas.iter())
            .enumerate()
        {
            let global_index = start_index + i;
            let is_selected = global_index == self.dashboard_performance_index
                && self.dashboard_focus == DashboardPanel::Performance;

            // Use brighter color for selected chart, normal title color for others
            let chart_title_color = if is_selected {
                self.financial_theme.accent_yellow
            } else {
                title_color
            };

            let chart = PerformanceChart::new(
                &observations,
                &indicator.id,
                &self.financial_theme,
                &indicator.name,
            )
            .title_color(chart_title_color);

            f.render_widget(chart, *chart_area);
        }
    }

    fn render_dashboard_sentiment(&self, f: &mut Frame, area: Rect) {
        let segments = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(area);

        let outcomes = [
            (OutcomeType::Work, self.goals.work.actions.as_slice()),
            (OutcomeType::Health, self.goals.health.actions.as_slice()),
            (OutcomeType::Family, self.goals.family.actions.as_slice()),
        ];

        // Use yellow title if this section is active, grey otherwise
        let title_color = if self.dashboard_focus == DashboardPanel::Sentiment {
            self.financial_theme.accent_yellow
        } else {
            self.financial_theme.text_dim
        };

        for ((outcome, actions), segment) in outcomes.into_iter().zip(segments.iter()) {
            let widget = SentimentWidget::new(outcome, actions, &self.financial_theme)
                .title_color(title_color);
            f.render_widget(widget, *segment);
        }
    }

    fn render_dashboard_signals(&mut self, f: &mut Frame, area: Rect) {
        let mut candidate_indicators: Vec<_> = self
            .indicators
            .indicators
            .iter()
            .filter(|indicator| {
                indicator.active && matches!(indicator.kind, IndicatorKind::Leading)
            })
            .collect();

        if candidate_indicators.is_empty() {
            candidate_indicators = self
                .indicators
                .indicators
                .iter()
                .filter(|indicator| indicator.active)
                .collect();
        }

        if candidate_indicators.is_empty() {
            self.dashboard_signal_ids.clear();
            self.dashboard_signal_index = 0;

            // Use yellow title if this section is active, grey otherwise
            let title_color = if self.dashboard_focus == DashboardPanel::Signals {
                self.financial_theme.accent_yellow
            } else {
                self.financial_theme.text_dim
            };

            let placeholder = Paragraph::new(
                "ALTERNATIVE SIGNALS\nActivate indicators to surface secondary metrics",
            )
            .style(
                Style::default()
                    .fg(self.financial_theme.text_primary)
                    .bg(self.financial_theme.bg_panel),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" ALTERNATIVE DATA SIGNALS ")
                    .title_style(
                        Style::default()
                            .fg(title_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.financial_theme.text_dim))
                    .style(Style::default().bg(self.financial_theme.bg_panel)),
            );

            f.render_widget(placeholder, area);
            return;
        }

        let today = chrono::Local::now().naive_local().date();
        let mut observations = crate::data::read_observations_range(
            today - chrono::Duration::days(30),
            today,
            &self.config,
        )
        .unwrap_or_default();
        observations.sort_by_key(|obs| obs.when);

        let mut latest_map: HashMap<String, (Option<f64>, Option<f64>)> = HashMap::new();
        for obs in &observations {
            let entry = latest_map
                .entry(obs.indicator_id.clone())
                .or_insert((None, None));
            entry.0 = entry.1;
            entry.1 = Some(obs.value);
        }

        let mut pending: Vec<(AlternativeSignal, f64)> = Vec::new();
        for indicator in candidate_indicators {
            let (previous, latest) = latest_map
                .get(&indicator.id)
                .cloned()
                .unwrap_or((None, None));

            let latest_value = latest.unwrap_or(0.0);
            let deviation = indicator
                .target
                .map(|target| (latest_value - target).abs() / target.max(1.0))
                .unwrap_or(0.0);
            let base_weight = if matches!(indicator.kind, IndicatorKind::Leading) {
                1.5
            } else {
                1.0
            };
            let weight_seed = base_weight + deviation;

            let signal = AlternativeSignal {
                indicator,
                latest_value,
                previous_value: previous,
                weight: 0.0,
            };

            pending.push((signal, weight_seed));
        }

        if pending.is_empty() {
            self.dashboard_signal_ids.clear();
            self.dashboard_signal_index = 0;

            // Use yellow title if this section is active, grey otherwise
            let title_color = if self.dashboard_focus == DashboardPanel::Signals {
                self.financial_theme.accent_yellow
            } else {
                self.financial_theme.text_dim
            };

            let placeholder = Paragraph::new(
                "ALTERNATIVE SIGNALS\nNo observation data available for active indicators",
            )
            .style(
                Style::default()
                    .fg(self.financial_theme.text_primary)
                    .bg(self.financial_theme.bg_panel),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" ALTERNATIVE DATA SIGNALS ")
                    .title_style(
                        Style::default()
                            .fg(title_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.financial_theme.text_dim))
                    .style(Style::default().bg(self.financial_theme.bg_panel)),
            );

            f.render_widget(placeholder, area);
            return;
        }

        let total_weight: f64 = pending.iter().map(|(_, seed)| *seed).sum();
        let mut signals: Vec<AlternativeSignal> = if total_weight <= f64::EPSILON {
            let uniform = 100.0 / pending.len() as f64;
            pending
                .into_iter()
                .map(|(mut signal, _)| {
                    signal.weight = uniform;
                    signal
                })
                .collect()
        } else {
            pending
                .into_iter()
                .map(|(mut signal, seed)| {
                    signal.weight = (seed / total_weight) * 100.0;
                    signal
                })
                .collect()
        };

        signals.sort_by(|a, b| {
            b.weight
                .partial_cmp(&a.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let display_signals = signals.into_iter().take(5).collect::<Vec<_>>();
        self.dashboard_signal_ids = display_signals
            .iter()
            .map(|signal| signal.indicator.id.clone())
            .collect();

        if self.dashboard_signal_ids.is_empty() {
            self.dashboard_signal_index = 0;
        } else if self.dashboard_signal_index >= self.dashboard_signal_ids.len() {
            self.dashboard_signal_index = self.dashboard_signal_ids.len().saturating_sub(1);
        }

        let selected = if self.dashboard_focus == DashboardPanel::Signals
            && !self.dashboard_signal_ids.is_empty()
        {
            Some(self.dashboard_signal_index)
        } else {
            None
        };

        // Use yellow title if this section is active, grey otherwise
        let title_color = if self.dashboard_focus == DashboardPanel::Signals {
            self.financial_theme.accent_yellow
        } else {
            self.financial_theme.text_dim
        };

        let widget = AlternativeSignalsWidget::new(display_signals, &self.financial_theme, selected)
            .title_color(title_color);
        f.render_widget(widget, area);
    }

    fn render_dashboard_status_line(&self, f: &mut Frame, area: Rect) {
        let status_content = if self.dashboard_focus == DashboardPanel::Market {
            // Get selected indicator name from Live Metrics
            if let Some(selected_idx) = if !self.dashboard_market_ids.is_empty() {
                Some(self.dashboard_market_index)
            } else {
                None
            } {
                if let Some(indicator) = self.indicators.indicators
                    .iter()
                    .filter(|ind| ind.active)
                    .nth(selected_idx)
                {
                    Line::from(vec![
                        Span::styled("Selected: ", Style::default().fg(self.financial_theme.text_secondary)),
                        Span::styled(&indicator.name, Style::default()
                            .fg(self.financial_theme.text_primary)
                            .add_modifier(ratatui::style::Modifier::BOLD)),
                    ])
                } else {
                    Line::from(Span::styled(
                        "Use ↑/↓ or j/k to navigate metrics",
                        Style::default().fg(self.financial_theme.text_secondary),
                    ))
                }
            } else {
                Line::from(Span::styled(
                    "Use ↑/↓ or j/k to navigate metrics",
                    Style::default().fg(self.financial_theme.text_secondary),
                ))
            }
        } else {
            Line::from(Span::styled(
                "Select Live Metrics panel to view indicator details",
                Style::default().fg(self.financial_theme.text_secondary),
            ))
        };

        let status_paragraph = ratatui::widgets::Paragraph::new(status_content)
            .block(ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title(" Status ")
                .title_style(Style::default()
                    .fg(self.financial_theme.text_dim)
                    .add_modifier(ratatui::style::Modifier::BOLD))
                .border_style(Style::default().fg(self.financial_theme.text_dim))
                .style(Style::default().bg(self.financial_theme.bg_panel)))
            .style(Style::default().bg(self.financial_theme.bg_panel));

        f.render_widget(status_paragraph, area);
    }

    fn render_dashboard_footer(&self, f: &mut Frame, area: Rect) {
        let accent = Style::default().fg(self.financial_theme.accent_yellow);
        let lines = vec![
            Line::from(vec![
                Span::styled("h / l", accent),
                Span::raw(" Cycle Panels  "),
                Span::styled("↑/↓ j/k", accent),
                Span::raw(" Navigate Signals  "),
                Span::styled("Enter", accent),
                Span::raw(" Inspect Indicator"),
            ]),
            Line::from(vec![
                Span::styled("d", accent),
                Span::raw(" Back to FocusFive  "),
                Span::styled("Esc", accent),
                Span::raw(" Close Dashboard  "),
                Span::styled("q", accent),
                Span::raw(" Quit"),
            ]),
        ];

        let outer = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.financial_theme.text_dim))
            .style(Style::default().bg(self.financial_theme.bg_secondary));
        f.render_widget(outer, area);

        let footer = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .style(Style::default().fg(self.financial_theme.text_secondary));
        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 0,
        });
        f.render_widget(footer, inner);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                "FOCUSFIVE",
                Style::default()
                    .fg(self.theme.header)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - "),
            Span::styled(
                self.goals.date.format("%B %d, %Y").to_string(),
                Style::default().fg(self.theme.text_primary),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(self.theme.border))
                .style(Style::default().bg(self.theme.panel_bg)),
        );
        f.render_widget(header, area);
    }

    fn render_outcomes(&mut self, f: &mut Frame, area: Rect) {
        let mut outcomes: Vec<ListItem> = Vec::new();

        for (_idx, (outcome_type, label, color)) in [
            (OutcomeType::Work, "Work", self.theme.work_color),
            (OutcomeType::Health, "Health", self.theme.health_color),
            (OutcomeType::Family, "Family", self.theme.family_color),
        ]
        .iter()
        .enumerate()
        {
            let outcome = match outcome_type {
                OutcomeType::Work => &self.goals.work,
                OutcomeType::Health => &self.goals.health,
                OutcomeType::Family => &self.goals.family,
            };

            let completed = outcome.actions.iter().filter(|a| a.completed).count();

            let is_selected =
                self.focus_panel == FocusPanel::Outcomes && self.selected_outcome == *outcome_type;

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
                        Style::default().fg(self.theme.text_secondary),
                    ),
                ]))
                .style(style),
            );
        }

        let border_color = if self.focus_panel == FocusPanel::Outcomes {
            self.theme.header
        } else {
            self.theme.border
        };

        let outcomes_list = List::new(outcomes).block(
            Block::default()
                .title(" OUTCOMES ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(self.theme.panel_bg)),
        );

        f.render_widget(outcomes_list, area);
    }

    fn render_actions(&mut self, f: &mut Frame, area: Rect) {
        let selected_outcome = self.get_selected_outcome();
        let mut actions_list = Vec::new();
        let mut current_line = 0;
        let mut _selected_display_line = None;
        let mut selected_indicator_counter = 0usize;

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
            let is_expanded = self.ui_state.is_expanded(&action.id);
            let expansion_symbol = if is_expanded { "▼ " } else { "▶ " };
            let checkbox = if action.completed { "[x]" } else { "[ ]" };
            let color = if action.completed {
                self.theme.completed
            } else {
                self.theme.text_secondary
            };

            let is_selected =
                self.focus_panel == FocusPanel::Actions && self.selected_action == idx;

            if is_selected {
                _selected_display_line = Some(current_line);
            }

            let style = if is_selected {
                Style::default()
                    .bg(self.theme.border)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Main action line with expansion symbol
            actions_list.push(
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} ", outcome_prefix),
                        Style::default().fg(outcome_color),
                    ),
                    Span::styled(
                        expansion_symbol,
                        Style::default().fg(self.theme.text_secondary),
                    ),
                    Span::styled(checkbox, Style::default().fg(color)),
                    Span::raw(" "),
                    Span::styled(&action.text, Style::default().fg(self.theme.text_primary)),
                ]))
                .style(style),
            );
            current_line += 1;

            // Add objective and indicators if expanded
            if is_expanded {
                if !action.objective_ids.is_empty() {
                    for obj_id in &action.objective_ids {
                        if let Some(objective) =
                            self.objectives.objectives.iter().find(|o| o.id == *obj_id)
                        {
                            // Objective line with icon
                            actions_list.push(ListItem::new(Line::from(vec![
                                Span::raw("  └─ 📎 Objective: "),
                                Span::styled(
                                    &objective.title,
                                    Style::default().fg(self.theme.text_primary),
                                ),
                            ])));
                            current_line += 1;

                            // Add indicators for this objective
                            let num_indicators = objective.indicators.len();
                            for (ind_idx, indicator_id) in objective.indicators.iter().enumerate() {
                                if let Some(indicator) = self.indicators_map.get(indicator_id) {
                                    let progress_bar = self.render_mini_progress(indicator);
                                    let value_display = self.format_indicator_value(indicator);
                                    let prefix = if ind_idx == num_indicators - 1 {
                                        "└─"
                                    } else {
                                        "├─"
                                    };

                                    let is_selected_action = self.focus_panel
                                        == FocusPanel::Actions
                                        && self.selected_action == idx;
                                    let is_selected_indicator = false;

                                    let mut item = ListItem::new(Line::from(vec![
                                        Span::raw(format!("      {} ", prefix)),
                                        Span::styled(
                                            &indicator.name,
                                            Style::default().fg(self.theme.text_primary),
                                        ),
                                        Span::raw(" "),
                                        Span::styled(
                                            progress_bar,
                                            Style::default().fg(Color::Cyan),
                                        ),
                                        Span::raw(" "),
                                        Span::styled(
                                            value_display,
                                            Style::default().fg(self.theme.text_secondary),
                                        ),
                                    ]));

                                    if is_selected_indicator {
                                        item = item.style(
                                            Style::default()
                                                .bg(self.theme.border)
                                                .fg(self.theme.text_primary)
                                                .add_modifier(Modifier::BOLD),
                                        );
                                    }

                                    actions_list.push(item);
                                    current_line += 1;

                                    if is_selected_action {
                                        selected_indicator_counter += 1;
                                    }
                                }
                            }

                            // Add overall progress
                            if !objective.indicators.is_empty() {
                                let overall_progress = self.calculate_objective_progress(objective);
                                actions_list.push(ListItem::new(Line::from(vec![
                                    Span::raw("      "),
                                    Span::styled(
                                        format!("Overall Progress: {:.0}%", overall_progress),
                                        Style::default()
                                            .fg(self.theme.text_secondary)
                                            .add_modifier(Modifier::ITALIC),
                                    ),
                                ])));
                                current_line += 1;
                            }
                        }
                    }
                } else {
                    // No objectives linked
                    actions_list.push(ListItem::new(Line::from(vec![
                        Span::raw("  └─ "),
                        Span::styled(
                            "(No objective linked)",
                            Style::default()
                                .fg(self.theme.text_secondary)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ])));
                    current_line += 1;
                }
            }
        }

        let border_color = if self.focus_panel == FocusPanel::Actions {
            self.theme.header
        } else {
            self.theme.border
        };

        let actions = List::new(actions_list).block(
            Block::default()
                .title(format!(
                    " ACTIONS - {} ",
                    match self.selected_outcome {
                        OutcomeType::Work => "Work",
                        OutcomeType::Health => "Health",
                        OutcomeType::Family => "Family",
                    }
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(self.theme.panel_bg)),
        );

        f.render_widget(actions, area);
    }

    fn render_mini_progress(&self, indicator: &Indicator) -> String {
        let progress = (indicator.current_value / indicator.target_value).min(1.0);
        let filled = (progress * 10.0) as usize;
        let empty = 10 - filled;

        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }

    fn format_indicator_value(&self, indicator: &Indicator) -> String {
        match indicator.indicator_type {
            IndicatorType::Counter => format!(
                "{}/{}",
                indicator.current_value as i32, indicator.target_value as i32
            ),
            IndicatorType::Duration => format!(
                "{:.1}/{:.1} hrs",
                indicator.current_value, indicator.target_value
            ),
            IndicatorType::Percentage => format!("{:.0}%", indicator.current_value),
            IndicatorType::Boolean => if indicator.current_value >= 1.0 {
                "✓ Complete"
            } else {
                "✗ Incomplete"
            }
            .to_string(),
        }
    }

    fn calculate_objective_progress(&self, objective: &crate::models::Objective) -> f64 {
        if objective.indicators.is_empty() {
            return 0.0;
        }

        let mut total_progress = 0.0;
        let mut valid_count = 0;

        for indicator_id in &objective.indicators {
            if let Some(indicator) = self.indicators_map.get(indicator_id) {
                let progress = (indicator.current_value / indicator.target_value).min(1.0);
                total_progress += progress;
                valid_count += 1;
            }
        }

        if valid_count > 0 {
            (total_progress / valid_count as f64) * 100.0
        } else {
            0.0
        }
    }

    fn render_stats(&self, f: &mut Frame, area: Rect) {
        use crate::ui::charts::{
            create_daily_gauge, create_outcome_gauges, render_trend_sparkline, WeeklyLineChart,
        };

        // Create a layout for the stats panel
        let _stats_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title area
                Constraint::Length(5), // Daily gauge
                Constraint::Length(7), // Outcome gauges
                Constraint::Min(10),   // Weekly chart
                Constraint::Length(5), // Monthly sparkline
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
        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });
        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Daily gauge
                Constraint::Length(4), // Outcome gauges
                Constraint::Min(8),    // Weekly chart
                Constraint::Length(4), // Monthly sparkline
            ])
            .split(inner);

        // Daily completion gauge
        let daily_gauge = create_daily_gauge(
            self.statistics.daily_completion,
            "TODAY'S PROGRESS",
            &self.theme,
        );
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

        let (work_gauge, health_gauge, family_gauge) =
            create_outcome_gauges(&self.statistics, &self.theme);
        f.render_widget(work_gauge, outcome_layout[0]);
        f.render_widget(health_gauge, outcome_layout[1]);
        f.render_widget(family_gauge, outcome_layout[2]);

        // Weekly line chart (7-day rolling window)
        if inner_layout[2].height > 5 {
            // Only render if there's enough space
            let weekly_chart = WeeklyLineChart::new(&self.statistics, self.goals.date, &self.theme);
            weekly_chart.render(f, inner_layout[2]);
        }

        // Monthly trend sparkline
        if !self.statistics.monthly_trend.is_empty() && inner_layout[3].height > 2 {
            render_trend_sparkline(
                &self.statistics.monthly_trend,
                "30-DAY TREND",
                &self.theme,
                f,
                inner_layout[3],
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
        let inner_area = area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 0,
        });
        help::render_help(f, inner_area, &self.theme);
    }

    pub fn render_live_metrics(&self, f: &mut Frame, area: Rect) {
        // Get current observations
        let today = chrono::Local::now().naive_local().date();
        let observations = crate::data::read_observations_range(
            today - chrono::Duration::days(7),
            today,
            &self.config,
        )
        .unwrap_or_default();

        let widget = LiveMetricsWidget::new(
            &self.indicators.indicators,
            &observations,
            &self.financial_theme,
        )
        .block(
            Block::default()
                .title(" LIVE METRICS ")
                .title_style(
                    Style::default()
                        .fg(self.financial_theme.accent_yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.financial_theme.text_dim))
                .style(Style::default().bg(self.financial_theme.bg_panel)),
        );

        f.render_widget(widget, area);
    }
}
