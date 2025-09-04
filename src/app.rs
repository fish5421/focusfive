use crate::data::{get_yesterday_goals, load_or_create_templates, save_objectives, save_templates};
use crate::models::{
    ActionTemplates, Config, DailyGoals, FiveYearVision, OutcomeType, RitualPhase,
};
use anyhow::Result;
use chrono::{Local, Timelike};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pane {
    Outcomes,
    Actions,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing {
        buffer: String,
        original: String,
    },
    GoalEditing {
        outcome_type: OutcomeType,
        buffer: String,
        original: String,
    },
    VisionEditing {
        outcome_type: OutcomeType,
        buffer: String,
        original: String,
    },
    CopyingFromYesterday {
        yesterday_goals: Box<DailyGoals>,
        selections: Vec<bool>, // Dynamic size for variable actions
        selection_index: usize,
    },
    TemplateSelection {
        templates: ActionTemplates,
        template_names: Vec<String>,
        selection_index: usize,
        outcome_type: OutcomeType,
    },
    TemplateSaving {
        templates: ActionTemplates,
        buffer: String,
        outcome_type: OutcomeType,
    },
    ObjectiveSelection {
        domain: OutcomeType,
        selection_index: usize,
    },
    ObjectiveCreation {
        domain: OutcomeType,
        buffer: String,
    },
    Reflecting {
        outcome_type: OutcomeType,
        buffer: String,
        original: String,
    },
    IndicatorManagement {
        objective_id: String,
        objective_title: String,
        indicators: Vec<crate::models::IndicatorDef>,
        selection_index: usize,
        editing_field: Option<IndicatorEditField>,
    },
    IndicatorCreation {
        objective_id: String,
        objective_title: String,
        field_index: usize,
        name_buffer: String,
        kind: crate::models::IndicatorKind,
        unit: crate::models::IndicatorUnit,
        unit_custom_buffer: String,
        target_buffer: String,
        direction: crate::models::IndicatorDirection,
        notes_buffer: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum IndicatorEditField {
    Name(String),
    Target(String),
    Notes(String),
}

pub struct App {
    pub goals: DailyGoals,
    pub config: Config,
    pub active_pane: Pane,
    pub outcome_index: usize,
    pub action_index: usize,
    pub should_quit: bool,
    pub needs_save: bool,
    pub show_help: bool,
    pub error_message: Option<String>,
    pub info_message: Option<String>,
    pub current_streak: u32,
    pub input_mode: InputMode,
    pub vision: FiveYearVision,
    pub vision_needs_save: bool,
    pub templates: ActionTemplates,
    pub templates_needs_save: bool,
    pub ritual_phase: RitualPhase,
    pub yesterday_context: Option<DailyGoals>, // Morning: yesterday's goals
    pub show_morning_prompt: bool,             // Morning: show helpful prompts
    pub completion_stats: Option<crate::models::CompletionStats>, // Evening: completion statistics
    pub daily_summary: String,                 // Evening: generated summary
    pub confirm_delete: bool,                  // Confirmation flag for deleting actions
    pub day_meta: crate::models::DayMeta,      // Rich metadata for actions
    pub meta_needs_save: bool,                 // Flag for saving metadata
    pub objectives: crate::models::ObjectivesData, // Long-term objectives
    pub objectives_needs_save: bool,           // Flag for saving objectives
    pub indicators: crate::models::IndicatorsData, // Key performance indicators
    pub indicators_needs_save: bool,           // Flag for saving indicators
}

impl App {
    /// Check if the key combination is a save command (Ctrl+Enter on Windows/Linux, Cmd+Enter on Mac)
    fn is_save_key_combo(key: &KeyEvent) -> bool {
        // Check for Ctrl+Enter or Cmd+Enter
        (key.code == KeyCode::Enter
            && (key.modifiers.contains(KeyModifiers::CONTROL)
                || key.modifiers.contains(KeyModifiers::SUPER)))
        // Also accept Ctrl+S as a universal save shortcut
        || (key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL))
    }

    pub fn new(goals: DailyGoals, config: Config, vision: FiveYearVision) -> Self {
        // Calculate initial streak
        let current_streak = crate::data::calculate_streak(&config).unwrap_or(0);

        // Load templates (create new if loading fails)
        let templates =
            load_or_create_templates(&config).unwrap_or_else(|_| ActionTemplates::new());

        // Determine ritual phase based on current time
        let current_hour = Local::now().hour();
        let ritual_phase = RitualPhase::from_hour(current_hour);

        // Load yesterday's context if in morning phase
        let yesterday_context = if matches!(ritual_phase, RitualPhase::Morning) {
            crate::data::get_yesterday_goals(goals.date, &config)
                .ok()
                .flatten()
        } else {
            None
        };

        // Calculate initial completion stats if in evening phase
        let completion_stats = if matches!(ritual_phase, RitualPhase::Evening) {
            Some(goals.completion_stats())
        } else {
            None
        };

        // Load or create day metadata aligned with current goals
        let day_meta = crate::data::load_or_create_day_meta(goals.date, &goals, &config)
            .unwrap_or_else(|_| crate::models::DayMeta::from_goals(&goals));

        // Load or create objectives
        let objectives = crate::data::load_or_create_objectives(&config)
            .unwrap_or_else(|_| crate::models::ObjectivesData::default());

        // Load or create indicators
        let indicators = crate::data::load_or_create_indicators(&config)
            .unwrap_or_else(|_| crate::models::IndicatorsData::default());

        Self {
            goals,
            config,
            active_pane: Pane::Outcomes,
            outcome_index: 0,
            action_index: 0,
            should_quit: false,
            needs_save: false,
            show_help: false,
            error_message: None,
            info_message: None,
            current_streak,
            input_mode: InputMode::Normal,
            vision,
            vision_needs_save: false,
            templates,
            templates_needs_save: false,
            ritual_phase,
            yesterday_context,
            show_morning_prompt: matches!(ritual_phase, RitualPhase::Morning),
            completion_stats,
            daily_summary: String::new(),
            confirm_delete: false,
            day_meta,
            meta_needs_save: false,
            objectives,
            objectives_needs_save: false,
            indicators,
            indicators_needs_save: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Clear messages on any key press
        self.error_message = None;
        self.info_message = None;

        // If help is showing, any key closes it
        if self.show_help {
            self.show_help = false;
            return Ok(());
        }

        // Handle input based on current mode
        match self.input_mode.clone() {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::Editing { buffer, original } => self.handle_edit_mode(key, buffer, original),
            InputMode::GoalEditing {
                outcome_type,
                buffer,
                original,
            } => self.handle_goal_edit_mode(key, outcome_type, buffer, original),
            InputMode::VisionEditing {
                outcome_type,
                buffer,
                original,
            } => self.handle_vision_edit_mode(key, outcome_type, buffer, original),
            InputMode::CopyingFromYesterday {
                yesterday_goals,
                selections,
                selection_index,
            } => self.handle_copy_from_yesterday_mode(
                key,
                yesterday_goals.as_ref(),
                selections,
                selection_index,
            ),
            InputMode::TemplateSelection {
                templates,
                template_names,
                selection_index,
                outcome_type,
            } => self.handle_template_selection_mode(
                key,
                templates,
                template_names,
                selection_index,
                outcome_type,
            ),
            InputMode::TemplateSaving {
                templates,
                buffer,
                outcome_type,
            } => self.handle_template_saving_mode(key, templates, buffer, outcome_type),
            InputMode::ObjectiveSelection {
                domain,
                selection_index,
            } => self.handle_objective_selection_mode(key, domain, selection_index),
            InputMode::ObjectiveCreation {
                domain,
                buffer,
            } => self.handle_objective_creation_mode(key, domain, buffer),
            InputMode::Reflecting {
                outcome_type,
                buffer,
                original,
            } => self.handle_reflecting_mode(key, outcome_type, buffer, original),
            InputMode::IndicatorManagement {
                objective_id,
                objective_title,
                indicators,
                selection_index,
                editing_field,
            } => self.handle_indicator_management_mode(
                key,
                objective_id,
                objective_title,
                indicators,
                selection_index,
                editing_field,
            ),
            InputMode::IndicatorCreation {
                objective_id,
                objective_title,
                field_index,
                name_buffer,
                kind,
                unit,
                unit_custom_buffer,
                target_buffer,
                direction,
                notes_buffer,
            } => self.handle_indicator_creation_mode(
                key,
                objective_id,
                objective_title,
                field_index,
                name_buffer,
                kind,
                unit,
                unit_custom_buffer,
                target_buffer,
                direction,
                notes_buffer,
            ),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<()> {
        // Handle phase-specific keys first
        match self.ritual_phase {
            RitualPhase::Morning => {
                // Morning-specific shortcuts
                if self.handle_morning_keys(key)? {
                    return Ok(());
                }
            }
            RitualPhase::Evening => {
                // Evening-specific shortcuts
                if self.handle_evening_keys(key)? {
                    return Ok(());
                }
            }
            _ => {}
        }

        // Handle normal keys
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('s') => {
                // Manual save
                self.needs_save = true;
                self.set_info("Saving...");
            }
            KeyCode::Char('?') => {
                self.show_help = true;
            }
            KeyCode::Tab => {
                self.active_pane = match self.active_pane {
                    Pane::Outcomes => Pane::Actions,
                    Pane::Actions => Pane::Outcomes,
                };
            }
            KeyCode::Char('j') | KeyCode::Down => match self.active_pane {
                Pane::Outcomes => {
                    if self.outcome_index < 2 {
                        self.outcome_index += 1;
                    }
                }
                Pane::Actions => {
                    let current_outcome = &self.goals.outcomes()[self.outcome_index];
                    if self.action_index < current_outcome.actions.len().saturating_sub(1) {
                        self.action_index += 1;
                    }
                }
            },
            KeyCode::Char('k') | KeyCode::Up => match self.active_pane {
                Pane::Outcomes => {
                    if self.outcome_index > 0 {
                        self.outcome_index -= 1;
                    }
                }
                Pane::Actions => {
                    if self.action_index > 0 {
                        self.action_index -= 1;
                    }
                }
            },
            KeyCode::Char(' ') => {
                if self.active_pane == Pane::Actions {
                    let outcome = match self.outcome_index {
                        0 => &mut self.goals.work,
                        1 => &mut self.goals.health,
                        2 => &mut self.goals.family,
                        _ => return Ok(()),
                    };
                    if self.action_index < outcome.actions.len() {
                        outcome.actions[self.action_index].cycle_status();
                    }
                    self.needs_save = true;
                }
            }
            KeyCode::Char('e') | KeyCode::Enter => {
                // Enter edit mode for the selected action
                if self.active_pane == Pane::Actions {
                    let outcome = match self.outcome_index {
                        0 => &self.goals.work,
                        1 => &self.goals.health,
                        2 => &self.goals.family,
                        _ => return Ok(()),
                    };
                    let action_text = if self.action_index < outcome.actions.len() {
                        outcome.actions[self.action_index].text.clone()
                    } else {
                        String::new()
                    };
                    self.input_mode = InputMode::Editing {
                        buffer: action_text.clone(),
                        original: action_text,
                    };
                }
            }
            KeyCode::Char('a') if self.active_pane == Pane::Actions => {
                // Add a new action (max 5)
                let outcome = match self.outcome_index {
                    0 => &mut self.goals.work,
                    1 => &mut self.goals.health,
                    2 => &mut self.goals.family,
                    _ => return Ok(()),
                };

                if let Err(e) = outcome.add_action() {
                    self.set_error(e.to_string());
                } else {
                    self.needs_save = true;
                    // Move to the new action
                    self.action_index = outcome.actions.len() - 1;
                    // Recalculate completion stats if in evening phase
                    if matches!(self.ritual_phase, RitualPhase::Evening) {
                        self.completion_stats = Some(self.goals.completion_stats());
                    }
                }
            }
            KeyCode::Char('d') if self.active_pane == Pane::Actions => {
                // Delete the selected action (with confirmation)
                if self.confirm_delete {
                    let outcome = match self.outcome_index {
                        0 => &mut self.goals.work,
                        1 => &mut self.goals.health,
                        2 => &mut self.goals.family,
                        _ => return Ok(()),
                    };

                    if let Err(e) = outcome.remove_action(self.action_index) {
                        self.set_error(e.to_string());
                    } else {
                        self.needs_save = true;
                        // Adjust action index if needed
                        if self.action_index >= outcome.actions.len() && self.action_index > 0 {
                            self.action_index = outcome.actions.len() - 1;
                        }
                        // Recalculate completion stats if in evening phase
                        if matches!(self.ritual_phase, RitualPhase::Evening) {
                            self.completion_stats = Some(self.goals.completion_stats());
                        }
                    }
                    self.confirm_delete = false;
                } else {
                    self.confirm_delete = true;
                    self.set_info("Press 'd' again to delete this action");
                }
            }
            KeyCode::Char('D') => {
                // Clear the selected action (old behavior)
                if self.active_pane == Pane::Actions {
                    let outcome = match self.outcome_index {
                        0 => &mut self.goals.work,
                        1 => &mut self.goals.health,
                        2 => &mut self.goals.family,
                        _ => return Ok(()),
                    };
                    if self.action_index < outcome.actions.len() {
                        outcome.actions[self.action_index].text.clear();
                        outcome.actions[self.action_index].completed = false;
                    }
                    self.needs_save = true;
                }
            }
            KeyCode::Char('v') => {
                // Open vision editor for selected outcome
                if self.active_pane == Pane::Outcomes {
                    let outcome_type = match self.outcome_index {
                        0 => OutcomeType::Work,
                        1 => OutcomeType::Health,
                        2 => OutcomeType::Family,
                        _ => return Ok(()),
                    };
                    let current_vision = self.vision.get_vision(&outcome_type).to_string();
                    self.input_mode = InputMode::VisionEditing {
                        outcome_type,
                        buffer: current_vision.clone(),
                        original: current_vision,
                    };
                }
            }
            KeyCode::Char('g') => {
                // Open goal editor for selected outcome
                if self.active_pane == Pane::Outcomes {
                    let outcome_type = match self.outcome_index {
                        0 => OutcomeType::Work,
                        1 => OutcomeType::Health,
                        2 => OutcomeType::Family,
                        _ => return Ok(()),
                    };
                    let current_goal = match self.outcome_index {
                        0 => self.goals.work.goal.clone().unwrap_or_default(),
                        1 => self.goals.health.goal.clone().unwrap_or_default(),
                        2 => self.goals.family.goal.clone().unwrap_or_default(),
                        _ => String::new(),
                    };
                    self.input_mode = InputMode::GoalEditing {
                        outcome_type,
                        buffer: current_goal.clone(),
                        original: current_goal,
                    };
                }
            }
            KeyCode::Char('y') => {
                // Copy from yesterday
                match get_yesterday_goals(self.goals.date, &self.config) {
                    Ok(Some(yesterday_goals)) => {
                        // Pre-select uncompleted actions (dynamic size)
                        let total_actions = yesterday_goals.work.actions.len()
                            + yesterday_goals.health.actions.len()
                            + yesterday_goals.family.actions.len();
                        let mut selections = vec![false; total_actions];
                        let mut index = 0;
                        for outcome in yesterday_goals.outcomes() {
                            for action in &outcome.actions {
                                if index < total_actions
                                    && !action.text.is_empty()
                                    && !action.completed
                                {
                                    selections[index] = true;
                                }
                                index += 1;
                            }
                        }

                        self.input_mode = InputMode::CopyingFromYesterday {
                            yesterday_goals: Box::new(yesterday_goals),
                            selections,
                            selection_index: 0,
                        };
                    }
                    Ok(None) => {
                        self.set_error("No goals found for yesterday".to_string());
                    }
                    Err(e) => {
                        self.set_error(format!("Failed to load yesterday's goals: {}", e));
                    }
                }
            }
            KeyCode::Char('t') => {
                // Select template to apply
                if self.active_pane == Pane::Actions {
                    let outcome_type = self.get_current_outcome_type();
                    let template_names = self.templates.get_template_names();

                    if template_names.is_empty() {
                        self.set_error("No templates available. Press 'T' to save current actions as a template.".to_string());
                    } else {
                        self.input_mode = InputMode::TemplateSelection {
                            templates: self.templates.clone(),
                            template_names,
                            selection_index: 0,
                            outcome_type,
                        };
                    }
                }
            }
            KeyCode::Char('o') => {
                // Open objective selector
                if self.active_pane == Pane::Actions {
                    let outcome_type = self.get_current_outcome_type();
                    self.input_mode = InputMode::ObjectiveSelection {
                        domain: outcome_type,
                        selection_index: 0,
                    };
                }
            }
            KeyCode::Char('T') => {
                // Save current actions as template
                if self.active_pane == Pane::Actions {
                    let outcome_type = self.get_current_outcome_type();
                    let current_outcome = self.get_current_outcome();

                    // Check if there are any non-empty actions to save
                    let has_actions = current_outcome.actions.iter().any(|a| !a.text.is_empty());

                    if has_actions {
                        self.input_mode = InputMode::TemplateSaving {
                            templates: self.templates.clone(),
                            buffer: String::new(),
                            outcome_type,
                        };
                    } else {
                        self.set_error("No actions to save as template".to_string());
                    }
                }
            }
            KeyCode::Char('m') => {
                // Switch to morning phase
                self.ritual_phase = RitualPhase::Morning;
            }
            KeyCode::Char('n') => {
                // Switch to evening (night) phase
                self.ritual_phase = RitualPhase::Evening;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_edit_mode(
        &mut self,
        key: KeyEvent,
        mut buffer: String,
        original: String,
    ) -> Result<()> {
        // Check for save key combo first (Ctrl+Enter or Cmd+Enter)
        if Self::is_save_key_combo(&key) || key.code == KeyCode::Enter {
            // Save the edited text
            let outcome = match self.outcome_index {
                0 => &mut self.goals.work,
                1 => &mut self.goals.health,
                2 => &mut self.goals.family,
                _ => return Ok(()),
            };
            outcome.actions[self.action_index].text = buffer;
            self.needs_save = true;
            self.input_mode = InputMode::Normal;
            return Ok(());
        }

        match key.code {
            KeyCode::Char(c) => {
                // Add character to buffer (respect 500 char limit)
                if buffer.len() < 500 {
                    buffer.push(c);
                    self.input_mode = InputMode::Editing { buffer, original };
                }
            }
            KeyCode::Backspace => {
                // Remove last character
                buffer.pop();
                self.input_mode = InputMode::Editing { buffer, original };
            }
            KeyCode::Esc => {
                // Cancel editing, restore original
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_goal_edit_mode(
        &mut self,
        key: KeyEvent,
        outcome_type: OutcomeType,
        mut buffer: String,
        original: String,
    ) -> Result<()> {
        // Check for save key combinations
        if Self::is_save_key_combo(&key) || key.code == KeyCode::Enter || key.code == KeyCode::F(2)
        {
            // Save the goal
            match outcome_type {
                OutcomeType::Work => self.goals.work.goal = Some(buffer),
                OutcomeType::Health => self.goals.health.goal = Some(buffer),
                OutcomeType::Family => self.goals.family.goal = Some(buffer),
            }
            self.needs_save = true;
            self.input_mode = InputMode::Normal;
            self.set_info("Goal saved");
            return Ok(());
        }

        match key.code {
            KeyCode::Char(c) => {
                // Add character to buffer (respect 100 char limit for goals)
                if buffer.len() < crate::models::MAX_GOAL_LENGTH {
                    buffer.push(c);
                    self.input_mode = InputMode::GoalEditing {
                        outcome_type,
                        buffer,
                        original,
                    };
                }
            }
            KeyCode::Backspace => {
                // Remove last character
                buffer.pop();
                self.input_mode = InputMode::GoalEditing {
                    outcome_type,
                    buffer,
                    original,
                };
            }
            KeyCode::Esc => {
                // Cancel editing, restore original
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_vision_edit_mode(
        &mut self,
        key: KeyEvent,
        outcome_type: OutcomeType,
        mut buffer: String,
        original: String,
    ) -> Result<()> {
        // Check for save key combinations
        if Self::is_save_key_combo(&key) || key.code == KeyCode::F(2) {
            // Save the vision
            self.vision.set_vision(&outcome_type, buffer);
            self.vision_needs_save = true;
            self.input_mode = InputMode::Normal;
            self.set_info("Vision saved");
            return Ok(());
        }

        match key.code {
            KeyCode::Char(c) => {
                // Add character to buffer (respect 1000 char limit)
                if buffer.len() < crate::models::MAX_VISION_LENGTH {
                    buffer.push(c);
                    self.input_mode = InputMode::VisionEditing {
                        outcome_type,
                        buffer,
                        original,
                    };
                }
            }
            KeyCode::Enter => {
                // Add newline
                if buffer.len() < crate::models::MAX_VISION_LENGTH {
                    buffer.push('\n');
                    self.input_mode = InputMode::VisionEditing {
                        outcome_type,
                        buffer,
                        original,
                    };
                }
            }
            KeyCode::Backspace => {
                // Remove last character
                buffer.pop();
                self.input_mode = InputMode::VisionEditing {
                    outcome_type,
                    buffer,
                    original,
                };
            }
            KeyCode::Esc => {
                // Cancel editing, restore original
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_copy_from_yesterday_mode(
        &mut self,
        key: KeyEvent,
        yesterday_goals: &DailyGoals,
        mut selections: Vec<bool>,
        mut selection_index: usize,
    ) -> Result<()> {
        // Calculate max index based on actual selections size
        let max_index = selections.len().saturating_sub(1);

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                // Move down in the list
                if selection_index < max_index {
                    selection_index += 1;
                    self.input_mode = InputMode::CopyingFromYesterday {
                        yesterday_goals: Box::new(yesterday_goals.clone()),
                        selections,
                        selection_index,
                    };
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                // Move up in the list
                if selection_index > 0 {
                    selection_index -= 1;
                    self.input_mode = InputMode::CopyingFromYesterday {
                        yesterday_goals: Box::new(yesterday_goals.clone()),
                        selections,
                        selection_index,
                    };
                }
            }
            KeyCode::Char(' ') => {
                // Toggle selection (check bounds first)
                if selection_index < selections.len() {
                    selections[selection_index] = !selections[selection_index];
                    self.input_mode = InputMode::CopyingFromYesterday {
                        yesterday_goals: Box::new(yesterday_goals.clone()),
                        selections,
                        selection_index,
                    };
                }
            }
            KeyCode::Enter => {
                // Copy selected actions to today
                let mut action_index = 0;
                let mut changes_made = false;

                for (outcome_idx, outcome) in yesterday_goals.outcomes().iter().enumerate() {
                    for (action_idx, action) in outcome.actions.iter().enumerate() {
                        if action_index < selections.len()
                            && selections[action_index]
                            && !action.text.is_empty()
                        {
                            // Copy this action to today's goals
                            let target_outcome = match outcome_idx {
                                0 => &mut self.goals.work,
                                1 => &mut self.goals.health,
                                2 => &mut self.goals.family,
                                _ => continue,
                            };

                            // Ensure we have room for this action
                            if action_idx < target_outcome.actions.len()
                                && target_outcome.actions[action_idx].text.is_empty()
                            {
                                target_outcome.actions[action_idx].text = action.text.clone();
                                target_outcome.actions[action_idx].origin =
                                    crate::models::ActionOrigin::CarryOver;
                                target_outcome.actions[action_idx]
                                    .set_status(crate::models::ActionStatus::Planned);
                                changes_made = true;
                            }
                        }
                        action_index += 1;
                    }
                }

                if changes_made {
                    self.needs_save = true;
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                // Cancel without copying
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn get_selected_outcome(&self) -> &crate::models::Outcome {
        match self.outcome_index {
            0 => &self.goals.work,
            1 => &self.goals.health,
            2 => &self.goals.family,
            _ => &self.goals.work,
        }
    }

    pub fn get_selected_outcome_type(&self) -> OutcomeType {
        match self.outcome_index {
            0 => OutcomeType::Work,
            1 => OutcomeType::Health,
            2 => OutcomeType::Family,
            _ => OutcomeType::Work,
        }
    }

    pub fn total_completed(&self) -> usize {
        self.goals
            .outcomes()
            .iter()
            .flat_map(|o| &o.actions)
            .filter(|a| a.completed)
            .count()
    }

    pub fn outcome_completed(&self, outcome: &crate::models::Outcome) -> usize {
        outcome.actions.iter().filter(|a| a.completed).count()
    }

    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
    }

    pub fn set_info(&mut self, message: &str) {
        self.info_message = Some(message.to_string());
        // Clear any error message when showing info
        self.error_message = None;
    }

    pub fn update_streak(&mut self) {
        self.current_streak =
            crate::data::calculate_streak(&self.config).unwrap_or(self.current_streak);
    }

    fn get_current_outcome_type(&self) -> OutcomeType {
        match self.outcome_index {
            0 => OutcomeType::Work,
            1 => OutcomeType::Health,
            2 => OutcomeType::Family,
            _ => OutcomeType::Work,
        }
    }

    fn get_current_outcome(&self) -> &crate::models::Outcome {
        match self.outcome_index {
            0 => &self.goals.work,
            1 => &self.goals.health,
            2 => &self.goals.family,
            _ => &self.goals.work,
        }
    }

    fn get_current_outcome_mut(&mut self) -> &mut crate::models::Outcome {
        match self.outcome_index {
            0 => &mut self.goals.work,
            1 => &mut self.goals.health,
            2 => &mut self.goals.family,
            _ => &mut self.goals.work,
        }
    }

    pub fn link_current_action_to_objective(&mut self, objective_id: Option<String>) -> Result<()> {
        // Get current action data before mutable borrow to avoid borrow checker issues
        let current_outcome_actions = match self.outcome_index {
            0 => self.goals.work.actions.clone(),
            1 => self.goals.health.actions.clone(),
            2 => self.goals.family.actions.clone(),
            _ => return Ok(()),
        };

        // Get the metadata for the current outcome
        let action_meta_list = match self.outcome_index {
            0 => &mut self.day_meta.work,
            1 => &mut self.day_meta.health,
            2 => &mut self.day_meta.family,
            _ => return Ok(()),
        };

        // Ensure we have metadata for the current action index
        while action_meta_list.len() <= self.action_index {
            if self.action_index < current_outcome_actions.len() {
                let action = &current_outcome_actions[self.action_index];
                let meta = crate::models::ActionMeta {
                    id: action.id.clone(),
                    status: action.status,
                    origin: action.origin.clone(),
                    estimated_min: None,
                    actual_min: None,
                    priority: None,
                    tags: vec![],
                    objective_id: None,
                };
                action_meta_list.push(meta);
            } else {
                break;
            }
        }

        // Update the objective_id for the current action
        if let Some(action_meta) = action_meta_list.get_mut(self.action_index) {
            action_meta.objective_id = objective_id;
            self.meta_needs_save = true;
        }

        Ok(())
    }

    fn handle_template_selection_mode(
        &mut self,
        key: KeyEvent,
        templates: ActionTemplates,
        template_names: Vec<String>,
        mut selection_index: usize,
        outcome_type: OutcomeType,
    ) -> Result<()> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if selection_index < template_names.len().saturating_sub(1) {
                    selection_index += 1;
                    self.input_mode = InputMode::TemplateSelection {
                        templates,
                        template_names,
                        selection_index,
                        outcome_type,
                    };
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if selection_index > 0 {
                    selection_index -= 1;
                    self.input_mode = InputMode::TemplateSelection {
                        templates,
                        template_names,
                        selection_index,
                        outcome_type,
                    };
                }
            }
            KeyCode::Enter => {
                // Apply the selected template
                if let Some(template_name) = template_names.get(selection_index) {
                    if let Some(actions) = templates.get_template(template_name) {
                        let current_outcome = self.get_current_outcome_mut();

                        // First, ensure we have enough action slots for the template
                        while current_outcome.actions.len() < actions.len() && current_outcome.actions.len() < 5 {
                            if let Err(e) = current_outcome.add_action() {
                                eprintln!("Warning: Could not add action slot: {}", e);
                                break;
                            }
                        }

                        // Apply template actions to empty slots (up to available slots)
                        for (i, action_text) in actions.iter().enumerate() {
                            if i < current_outcome.actions.len() {
                                // Only overwrite empty actions to avoid losing user data
                                if current_outcome.actions[i].text.is_empty() {
                                    current_outcome.actions[i].text = action_text.clone();
                                    current_outcome.actions[i].origin =
                                        crate::models::ActionOrigin::Template;
                                    current_outcome.actions[i]
                                        .set_status(crate::models::ActionStatus::Planned);
                                }
                            }
                        }

                        self.needs_save = true;
                    }
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_objective_selection_mode(
        &mut self,
        key: KeyEvent,
        domain: OutcomeType,
        mut selection_index: usize,
    ) -> Result<()> {
        // Get objectives for the current domain
        let domain_objectives: Vec<&crate::models::Objective> = self
            .objectives
            .objectives
            .iter()
            .filter(|obj| obj.domain == domain)
            .collect();

        // Total options = objectives + "Create New" option
        let total_options = domain_objectives.len() + 1;

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if selection_index < total_options.saturating_sub(1) {
                    selection_index += 1;
                    self.input_mode = InputMode::ObjectiveSelection {
                        domain,
                        selection_index,
                    };
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if selection_index > 0 {
                    selection_index -= 1;
                    self.input_mode = InputMode::ObjectiveSelection {
                        domain,
                        selection_index,
                    };
                }
            }
            KeyCode::Enter => {
                if selection_index < domain_objectives.len() {
                    // Selected an existing objective - get data before mutable operations
                    let selected_objective = domain_objectives[selection_index];
                    let objective_id = selected_objective.id.clone();
                    let objective_title = selected_objective.title.clone();
                    
                    self.link_current_action_to_objective(Some(objective_id))?;
                    self.set_info(&format!("Linked to objective: {}", objective_title));
                    self.input_mode = InputMode::Normal;
                } else {
                    // Selected "Create New Objective" - enter creation mode
                    self.input_mode = InputMode::ObjectiveCreation {
                        domain,
                        buffer: String::new(),
                    };
                }
            }
            KeyCode::Char('i') => {
                // Open indicator management for selected objective
                if selection_index < domain_objectives.len() {
                    let selected_objective = domain_objectives[selection_index];
                    let objective_id = selected_objective.id.clone();
                    let objective_title = selected_objective.title.clone();
                    
                    // Get indicators for this objective
                    let objective_indicators: Vec<crate::models::IndicatorDef> = self
                        .indicators
                        .indicators
                        .iter()
                        .filter(|ind| ind.objective_id.as_ref() == Some(&objective_id))
                        .cloned()
                        .collect();
                    
                    self.input_mode = InputMode::IndicatorManagement {
                        objective_id,
                        objective_title,
                        indicators: objective_indicators,
                        selection_index: 0,
                        editing_field: None,
                    };
                } else {
                    self.set_error("Create an objective first before managing indicators".to_string());
                }
            }
            KeyCode::Char('n') => {
                // Unlink from objective (set to None)
                self.link_current_action_to_objective(None)?;
                self.set_info("Unlinked from objective");
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_objective_creation_mode(
        &mut self,
        key: KeyEvent,
        domain: OutcomeType,
        mut buffer: String,
    ) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                if !buffer.trim().is_empty() {
                    // Create new objective
                    let new_objective = crate::models::Objective::new(domain, buffer.trim().to_string());
                    let objective_id = new_objective.id.clone();
                    let objective_title = new_objective.title.clone();
                    
                    // Add to objectives list
                    self.objectives.objectives.push(new_objective);
                    
                    // Save objectives to file
                    if let Err(e) = save_objectives(&self.objectives, &self.config) {
                        self.set_error(format!("Failed to save objective: {}", e));
                    } else {
                        // Link current action to the new objective
                        if let Err(e) = self.link_current_action_to_objective(Some(objective_id)) {
                            self.set_error(format!("Failed to link action to objective: {}", e));
                        } else {
                            self.set_info(&format!("Created and linked to objective: {}", objective_title));
                        }
                    }
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                buffer.push(c);
                self.input_mode = InputMode::ObjectiveCreation {
                    domain,
                    buffer,
                };
            }
            KeyCode::Backspace => {
                buffer.pop();
                self.input_mode = InputMode::ObjectiveCreation {
                    domain,
                    buffer,
                };
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_template_saving_mode(
        &mut self,
        key: KeyEvent,
        mut templates: ActionTemplates,
        mut buffer: String,
        outcome_type: OutcomeType,
    ) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                if !buffer.is_empty() {
                    // Get current actions
                    let current_outcome = self.get_current_outcome();
                    let actions: Vec<String> = current_outcome
                        .actions
                        .iter()
                        .filter(|a| !a.text.is_empty())
                        .map(|a| a.text.clone())
                        .collect();

                    if !actions.is_empty() {
                        // Save template
                        templates.add_template(buffer.clone(), actions);
                        self.templates = templates.clone();
                        self.templates_needs_save = true;

                        // Save to file
                        if let Err(e) = save_templates(&self.templates, &self.config) {
                            self.set_error(format!("Failed to save template: {}", e));
                        }
                    }
                }
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                buffer.push(c);
                self.input_mode = InputMode::TemplateSaving {
                    templates,
                    buffer,
                    outcome_type,
                };
            }
            KeyCode::Backspace => {
                buffer.pop();
                self.input_mode = InputMode::TemplateSaving {
                    templates,
                    buffer,
                    outcome_type,
                };
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle morning phase specific keys
    fn handle_morning_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            // Quick-fill from yesterday with 'y' key
            KeyCode::Char('y') if self.yesterday_context.is_some() => {
                let yesterday = self.yesterday_context.clone().unwrap();
                // Apply all incomplete tasks from yesterday
                self.apply_yesterday_incomplete(&yesterday);
                self.needs_save = true;
                self.set_error("Applied incomplete tasks from yesterday".to_string());
                Ok(true)
            }
            // Quick template application with number keys (dynamic range)
            KeyCode::Char(c @ '1'..='9') => {
                let index = (c as u8 - b'1') as usize;
                let template_names = self.templates.get_template_names();
                if index < template_names.len() {
                    let template_name = &template_names[index];
                    let outcome_type = self.get_current_outcome_type();
                    if let Some(actions) = self.templates.get_template(template_name) {
                        self.apply_template_actions(&outcome_type, actions.clone());
                        self.needs_save = true;
                        self.set_error(format!("Applied template: {}", template_name));
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Handle evening phase specific keys
    fn handle_evening_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            // Quick completion toggle with number keys (1-9)
            KeyCode::Char(c @ '1'..='9') => {
                let index = (c as u8 - b'1') as usize;
                self.toggle_action_by_global_index(index);
                self.needs_save = true;
                // Recalculate stats
                self.completion_stats = Some(self.goals.completion_stats());
                Ok(true)
            }
            // Quick completion toggle with letters (a-f for indices 9-14)
            KeyCode::Char(c @ 'a'..='f') if matches!(self.input_mode, InputMode::Normal) => {
                let index = 9 + (c as u8 - b'a') as usize;
                self.toggle_action_by_global_index(index);
                self.needs_save = true;
                // Recalculate stats
                self.completion_stats = Some(self.goals.completion_stats());
                Ok(true)
            }
            // Add reflection
            KeyCode::Char('r') => {
                let outcome_type = self.get_current_outcome_type();
                let current_reflection = match outcome_type {
                    OutcomeType::Work => self.goals.work.reflection.clone(),
                    OutcomeType::Health => self.goals.health.reflection.clone(),
                    OutcomeType::Family => self.goals.family.reflection.clone(),
                }
                .unwrap_or_default();

                self.input_mode = InputMode::Reflecting {
                    outcome_type,
                    buffer: current_reflection.clone(),
                    original: current_reflection,
                };
                Ok(true)
            }
            // Generate daily summary
            KeyCode::Char('d') => {
                self.generate_daily_summary();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Handle reflection input mode
    fn handle_reflecting_mode(
        &mut self,
        key: KeyEvent,
        outcome_type: OutcomeType,
        mut buffer: String,
        original: String,
    ) -> Result<()> {
        // Check for save key combinations
        if Self::is_save_key_combo(&key) || key.code == KeyCode::F(2) {
            // Save the reflection
            match outcome_type {
                OutcomeType::Work => self.goals.work.reflection = Some(buffer),
                OutcomeType::Health => self.goals.health.reflection = Some(buffer),
                OutcomeType::Family => self.goals.family.reflection = Some(buffer),
            }
            self.needs_save = true;
            self.input_mode = InputMode::Normal;
            self.set_info("Reflection saved");
            return Ok(());
        }

        match key.code {
            KeyCode::Char(c) => {
                if buffer.len() < 500 {
                    buffer.push(c);
                    self.input_mode = InputMode::Reflecting {
                        outcome_type,
                        buffer,
                        original,
                    };
                }
            }
            KeyCode::Enter => {
                // Add newline
                if buffer.len() < 500 {
                    buffer.push('\n');
                    self.input_mode = InputMode::Reflecting {
                        outcome_type,
                        buffer,
                        original,
                    };
                }
            }
            KeyCode::Backspace => {
                buffer.pop();
                self.input_mode = InputMode::Reflecting {
                    outcome_type,
                    buffer,
                    original,
                };
            }
            KeyCode::Esc => {
                // Cancel and restore original
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle indicator management mode
    fn handle_indicator_management_mode(
        &mut self,
        key: KeyEvent,
        objective_id: String,
        objective_title: String,
        mut indicators: Vec<crate::models::IndicatorDef>,
        mut selection_index: usize,
        editing_field: Option<IndicatorEditField>,
    ) -> Result<()> {
        // If editing a field, handle text input
        if let Some(ref field) = editing_field {
            match key.code {
                KeyCode::Enter => {
                    // Save the edit
                    if selection_index < indicators.len() {
                        match field {
                            IndicatorEditField::Name(ref buffer) => {
                                indicators[selection_index].name = buffer.clone();
                            }
                            IndicatorEditField::Target(ref buffer) => {
                                indicators[selection_index].target = buffer.parse::<f64>().ok();
                            }
                            IndicatorEditField::Notes(ref buffer) => {
                                indicators[selection_index].notes = if buffer.is_empty() {
                                    None
                                } else {
                                    Some(buffer.clone())
                                };
                            }
                        }
                        
                        // Update the main indicators list
                        let indicator = &indicators[selection_index];
                        if let Some(main_ind) = self
                            .indicators
                            .indicators
                            .iter_mut()
                            .find(|ind| ind.id == indicator.id)
                        {
                            main_ind.name = indicator.name.clone();
                            main_ind.target = indicator.target;
                            main_ind.notes = indicator.notes.clone();
                            main_ind.modified = chrono::Utc::now();
                        }
                        self.indicators_needs_save = true;
                    }
                    
                    // Exit edit mode
                    self.input_mode = InputMode::IndicatorManagement {
                        objective_id,
                        objective_title,
                        indicators,
                        selection_index,
                        editing_field: None,
                    };
                }
                KeyCode::Char(c) => {
                    // Add character to buffer
                    let mut new_field = editing_field.clone();
                    match &mut new_field {
                        Some(IndicatorEditField::Name(ref mut buffer))
                        | Some(IndicatorEditField::Target(ref mut buffer))
                        | Some(IndicatorEditField::Notes(ref mut buffer)) => {
                            buffer.push(c);
                        }
                        _ => {}
                    }
                    self.input_mode = InputMode::IndicatorManagement {
                        objective_id,
                        objective_title,
                        indicators,
                        selection_index,
                        editing_field: new_field,
                    };
                }
                KeyCode::Backspace => {
                    // Remove character from buffer
                    let mut new_field = editing_field.clone();
                    match &mut new_field {
                        Some(IndicatorEditField::Name(ref mut buffer))
                        | Some(IndicatorEditField::Target(ref mut buffer))
                        | Some(IndicatorEditField::Notes(ref mut buffer)) => {
                            buffer.pop();
                        }
                        _ => {}
                    }
                    self.input_mode = InputMode::IndicatorManagement {
                        objective_id,
                        objective_title,
                        indicators,
                        selection_index,
                        editing_field: new_field,
                    };
                }
                KeyCode::Esc => {
                    // Cancel editing
                    self.input_mode = InputMode::IndicatorManagement {
                        objective_id,
                        objective_title,
                        indicators,
                        selection_index,
                        editing_field: None,
                    };
                }
                _ => {}
            }
            return Ok(());
        }
        
        // Normal navigation mode
        let total_options = indicators.len() + 1; // +1 for "Create New"
        
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if selection_index < total_options.saturating_sub(1) {
                    selection_index += 1;
                    self.input_mode = InputMode::IndicatorManagement {
                        objective_id,
                        objective_title,
                        indicators,
                        selection_index,
                        editing_field: None,
                    };
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if selection_index > 0 {
                    selection_index -= 1;
                    self.input_mode = InputMode::IndicatorManagement {
                        objective_id,
                        objective_title,
                        indicators,
                        selection_index,
                        editing_field: None,
                    };
                }
            }
            KeyCode::Enter => {
                if selection_index < indicators.len() {
                    // Edit name of existing indicator
                    let current_name = indicators[selection_index].name.clone();
                    self.input_mode = InputMode::IndicatorManagement {
                        objective_id,
                        objective_title,
                        indicators,
                        selection_index,
                        editing_field: Some(IndicatorEditField::Name(current_name)),
                    };
                } else {
                    // Create new indicator
                    self.input_mode = InputMode::IndicatorCreation {
                        objective_id,
                        objective_title,
                        field_index: 0,
                        name_buffer: String::new(),
                        kind: crate::models::IndicatorKind::Leading,
                        unit: crate::models::IndicatorUnit::Count,
                        unit_custom_buffer: String::new(),
                        target_buffer: String::new(),
                        direction: crate::models::IndicatorDirection::HigherIsBetter,
                        notes_buffer: String::new(),
                    };
                }
            }
            KeyCode::Char('t') if selection_index < indicators.len() => {
                // Edit target
                let current_target = indicators[selection_index]
                    .target
                    .map(|t| t.to_string())
                    .unwrap_or_default();
                self.input_mode = InputMode::IndicatorManagement {
                    objective_id,
                    objective_title,
                    indicators,
                    selection_index,
                    editing_field: Some(IndicatorEditField::Target(current_target)),
                };
            }
            KeyCode::Char('n') if selection_index < indicators.len() => {
                // Edit notes
                let current_notes = indicators[selection_index]
                    .notes
                    .clone()
                    .unwrap_or_default();
                self.input_mode = InputMode::IndicatorManagement {
                    objective_id,
                    objective_title,
                    indicators,
                    selection_index,
                    editing_field: Some(IndicatorEditField::Notes(current_notes)),
                };
            }
            KeyCode::Char('d') if selection_index < indicators.len() => {
                // Delete indicator
                let indicator_id = indicators[selection_index].id.clone();
                
                // Remove from main list
                self.indicators.indicators.retain(|ind| ind.id != indicator_id);
                self.indicators_needs_save = true;
                
                // Remove from local list
                indicators.remove(selection_index);
                
                // Adjust selection index if needed
                if selection_index >= indicators.len() && selection_index > 0 {
                    selection_index = indicators.len() - 1;
                }
                
                self.input_mode = InputMode::IndicatorManagement {
                    objective_id,
                    objective_title,
                    indicators,
                    selection_index,
                    editing_field: None,
                };
                
                self.set_info("Indicator deleted");
            }
            KeyCode::Char(' ') if selection_index < indicators.len() => {
                // Toggle active status
                indicators[selection_index].active = !indicators[selection_index].active;
                
                // Update main list
                let indicator = &indicators[selection_index];
                if let Some(main_ind) = self
                    .indicators
                    .indicators
                    .iter_mut()
                    .find(|ind| ind.id == indicator.id)
                {
                    main_ind.active = indicator.active;
                    main_ind.modified = chrono::Utc::now();
                }
                self.indicators_needs_save = true;
                
                self.input_mode = InputMode::IndicatorManagement {
                    objective_id,
                    objective_title,
                    indicators,
                    selection_index,
                    editing_field: None,
                };
            }
            KeyCode::Esc => {
                // Return to objective selection
                self.input_mode = InputMode::ObjectiveSelection {
                    domain: self.get_current_outcome_type(),
                    selection_index: 0,
                };
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle indicator creation mode
    fn handle_indicator_creation_mode(
        &mut self,
        key: KeyEvent,
        objective_id: String,
        objective_title: String,
        mut field_index: usize,
        mut name_buffer: String,
        mut kind: crate::models::IndicatorKind,
        mut unit: crate::models::IndicatorUnit,
        mut unit_custom_buffer: String,
        mut target_buffer: String,
        mut direction: crate::models::IndicatorDirection,
        mut notes_buffer: String,
    ) -> Result<()> {
        match key.code {
            KeyCode::Tab => {
                // Move to next field
                field_index = (field_index + 1) % 6; // 6 fields total
                self.input_mode = InputMode::IndicatorCreation {
                    objective_id,
                    objective_title,
                    field_index,
                    name_buffer,
                    kind,
                    unit,
                    unit_custom_buffer,
                    target_buffer,
                    direction,
                    notes_buffer,
                };
            }
            KeyCode::Enter => {
                // Save if name is provided
                if !name_buffer.trim().is_empty() {
                    let mut indicator = crate::models::IndicatorDef::new(
                        name_buffer.trim().to_string(),
                        kind,
                        unit.clone(),
                    );
                    indicator.objective_id = Some(objective_id.clone());
                    indicator.target = target_buffer.parse::<f64>().ok();
                    indicator.direction = direction;
                    indicator.notes = if notes_buffer.is_empty() {
                        None
                    } else {
                        Some(notes_buffer)
                    };
                    
                    // Add to indicators
                    self.indicators.indicators.push(indicator.clone());
                    self.indicators_needs_save = true;
                    
                    // Return to management mode with updated list
                    let objective_indicators: Vec<crate::models::IndicatorDef> = self
                        .indicators
                        .indicators
                        .iter()
                        .filter(|ind| ind.objective_id.as_ref() == Some(&objective_id))
                        .cloned()
                        .collect();
                    
                    self.input_mode = InputMode::IndicatorManagement {
                        objective_id,
                        objective_title,
                        indicators: objective_indicators,
                        selection_index: 0,
                        editing_field: None,
                    };
                    
                    self.set_info("Indicator created");
                } else {
                    self.set_error("Indicator name is required".to_string());
                }
            }
            KeyCode::Char(c) => {
                // Input based on current field
                match field_index {
                    0 => name_buffer.push(c), // Name
                    1 => {
                        // Kind (l for Leading, a for Lagging)
                        if c == 'l' || c == 'L' {
                            kind = crate::models::IndicatorKind::Leading;
                        } else if c == 'a' || c == 'A' {
                            kind = crate::models::IndicatorKind::Lagging;
                        }
                    }
                    2 => {
                        // Unit (c=Count, m=Minutes, d=Dollars, p=Percent, u=Custom)
                        match c {
                            'c' | 'C' => unit = crate::models::IndicatorUnit::Count,
                            'm' | 'M' => unit = crate::models::IndicatorUnit::Minutes,
                            'd' | 'D' => unit = crate::models::IndicatorUnit::Dollars,
                            'p' | 'P' => unit = crate::models::IndicatorUnit::Percent,
                            'u' | 'U' => {
                                unit = crate::models::IndicatorUnit::Custom(unit_custom_buffer.clone())
                            }
                            _ if matches!(unit, crate::models::IndicatorUnit::Custom(_)) => {
                                unit_custom_buffer.push(c);
                                unit = crate::models::IndicatorUnit::Custom(unit_custom_buffer.clone());
                            }
                            _ => {}
                        }
                    }
                    3 => {
                        // Target (numeric only)
                        if c.is_ascii_digit() || c == '.' {
                            target_buffer.push(c);
                        }
                    }
                    4 => {
                        // Direction (h=Higher, l=Lower, r=Range)
                        match c {
                            'h' | 'H' => direction = crate::models::IndicatorDirection::HigherIsBetter,
                            'l' | 'L' => direction = crate::models::IndicatorDirection::LowerIsBetter,
                            'r' | 'R' => direction = crate::models::IndicatorDirection::WithinRange,
                            _ => {}
                        }
                    }
                    5 => notes_buffer.push(c), // Notes
                    _ => {}
                }
                
                self.input_mode = InputMode::IndicatorCreation {
                    objective_id,
                    objective_title,
                    field_index,
                    name_buffer,
                    kind,
                    unit,
                    unit_custom_buffer,
                    target_buffer,
                    direction,
                    notes_buffer,
                };
            }
            KeyCode::Backspace => {
                // Remove character based on current field
                match field_index {
                    0 => { name_buffer.pop(); }
                    2 if matches!(unit, crate::models::IndicatorUnit::Custom(_)) => {
                        unit_custom_buffer.pop();
                        unit = crate::models::IndicatorUnit::Custom(unit_custom_buffer.clone());
                    }
                    3 => { target_buffer.pop(); }
                    5 => { notes_buffer.pop(); }
                    _ => {}
                }
                
                self.input_mode = InputMode::IndicatorCreation {
                    objective_id,
                    objective_title,
                    field_index,
                    name_buffer,
                    kind,
                    unit,
                    unit_custom_buffer,
                    target_buffer,
                    direction,
                    notes_buffer,
                };
            }
            KeyCode::Esc => {
                // Cancel and return to management mode
                let objective_indicators: Vec<crate::models::IndicatorDef> = self
                    .indicators
                    .indicators
                    .iter()
                    .filter(|ind| ind.objective_id.as_ref() == Some(&objective_id))
                    .cloned()
                    .collect();
                
                self.input_mode = InputMode::IndicatorManagement {
                    objective_id,
                    objective_title,
                    indicators: objective_indicators,
                    selection_index: 0,
                    editing_field: None,
                };
            }
            _ => {}
        }
        Ok(())
    }

    /// Apply incomplete tasks from yesterday
    pub fn apply_yesterday_incomplete(&mut self, yesterday: &DailyGoals) {
        // Work actions
        for (i, action) in yesterday.work.actions.iter().enumerate() {
            if !action.completed
                && !action.text.is_empty()
                && i < self.goals.work.actions.len()
                && self.goals.work.actions[i].text.is_empty()
            {
                self.goals.work.actions[i].text = action.text.clone();
                self.goals.work.actions[i].origin = crate::models::ActionOrigin::CarryOver;
                self.goals.work.actions[i].set_status(crate::models::ActionStatus::Planned);
            }
        }

        // Health actions
        for (i, action) in yesterday.health.actions.iter().enumerate() {
            if !action.completed
                && !action.text.is_empty()
                && i < self.goals.health.actions.len()
                && self.goals.health.actions[i].text.is_empty()
            {
                self.goals.health.actions[i].text = action.text.clone();
                self.goals.health.actions[i].origin = crate::models::ActionOrigin::CarryOver;
                self.goals.health.actions[i].set_status(crate::models::ActionStatus::Planned);
            }
        }

        // Family actions
        for (i, action) in yesterday.family.actions.iter().enumerate() {
            if !action.completed
                && !action.text.is_empty()
                && i < self.goals.family.actions.len()
                && self.goals.family.actions[i].text.is_empty()
            {
                self.goals.family.actions[i].text = action.text.clone();
                self.goals.family.actions[i].origin = crate::models::ActionOrigin::CarryOver;
                self.goals.family.actions[i].set_status(crate::models::ActionStatus::Planned);
            }
        }
    }

    /// Apply template actions to current outcome
    fn apply_template_actions(&mut self, outcome_type: &OutcomeType, actions: Vec<String>) {
        let outcome = match outcome_type {
            OutcomeType::Work => &mut self.goals.work,
            OutcomeType::Health => &mut self.goals.health,
            OutcomeType::Family => &mut self.goals.family,
        };

        // Apply to existing empty slots (up to current action count)
        for (i, action_text) in actions.iter().enumerate() {
            if i < outcome.actions.len() && outcome.actions[i].text.is_empty() {
                outcome.actions[i].text = action_text.clone();
                outcome.actions[i].origin = crate::models::ActionOrigin::Template;
                outcome.actions[i].set_status(crate::models::ActionStatus::Planned);
            }
        }
    }

    /// Toggle action by global index (handles variable action counts)
    /// Now cycles through status: Planned → InProgress → Done → Skipped → Blocked → Planned
    pub fn toggle_action_by_global_index(&mut self, global_index: usize) {
        let mut current_index = 0;

        // Work actions
        for action in self.goals.work.actions.iter_mut() {
            if current_index == global_index {
                action.cycle_status();
                return;
            }
            current_index += 1;
        }

        // Health actions
        for action in self.goals.health.actions.iter_mut() {
            if current_index == global_index {
                action.cycle_status();
                return;
            }
            current_index += 1;
        }

        // Family actions
        for action in self.goals.family.actions.iter_mut() {
            if current_index == global_index {
                action.cycle_status();
                return;
            }
            current_index += 1;
        }
    }

    // Keep old method for backward compatibility
    pub fn toggle_action_by_index(&mut self, index: usize) {
        self.toggle_action_by_global_index(index);
    }

    /// Generate daily summary for evening review
    pub fn generate_daily_summary(&mut self) {
        let stats = self.goals.completion_stats();
        let mut summary = String::new();

        summary.push_str(&format!(
            "📊 Day {} Summary\n",
            self.goals.day_number.unwrap_or(0)
        ));
        summary.push_str(&format!(
            "Completion: {}/{} ({}%)\n\n",
            stats.completed, stats.total, stats.percentage
        ));

        // Per-outcome breakdown
        for (name, done, total) in &stats.by_outcome {
            let emoji = match *done {
                3 => "✅",
                2 => "🔶",
                1 => "⚠️",
                _ => "❌",
            };
            summary.push_str(&format!("{} {}: {}/{}\n", emoji, name, done, total));
        }

        // Add reflections if any
        if self.goals.work.reflection.is_some()
            || self.goals.health.reflection.is_some()
            || self.goals.family.reflection.is_some()
        {
            summary.push_str("\n📝 Reflections:\n");
            if let Some(ref r) = self.goals.work.reflection {
                summary.push_str(&format!("Work: {}\n", r));
            }
            if let Some(ref r) = self.goals.health.reflection {
                summary.push_str(&format!("Health: {}\n", r));
            }
            if let Some(ref r) = self.goals.family.reflection {
                summary.push_str(&format!("Family: {}\n", r));
            }
        }

        self.daily_summary = summary;
        self.set_error("Daily summary generated! Press '?' to view".to_string());
    }
}
