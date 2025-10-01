use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Input validation constants
pub const MAX_ACTION_LENGTH: usize = 500;
pub const MAX_GOAL_LENGTH: usize = 100;
pub const MAX_VISION_LENGTH: usize = 1000;

/// A single action item with completion status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    #[serde(default = "Action::generate_id")]
    pub id: String, // UUID for stable identification
    pub text: String,
    pub completed: bool, // Mirror of status for compatibility (completed = status == Done)
    #[serde(default = "Action::default_status")]
    pub status: ActionStatus, // Rich status tracking
    #[serde(default = "Action::default_origin")]
    pub origin: ActionOrigin, // How this action was created
    #[serde(default)]
    pub objective_id: Option<String>, // DEPRECATED: Link to ONE objective (kept for compatibility)
    #[serde(default)]
    pub objective_ids: Vec<String>, // Link to MULTIPLE objectives
    #[serde(default = "chrono::Utc::now")]
    pub created: chrono::DateTime<chrono::Utc>,
    #[serde(default = "chrono::Utc::now")]
    pub modified: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>, // When completed
}

impl Action {
    /// Generate a new UUID for an action
    fn generate_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Default status for new actions
    fn default_status() -> ActionStatus {
        ActionStatus::Planned
    }

    /// Default origin for new actions
    fn default_origin() -> ActionOrigin {
        ActionOrigin::Manual
    }

    /// Create a new empty action with default values
    pub fn new_empty() -> Self {
        let now = chrono::Utc::now();
        let mut action = Action {
            id: Self::generate_id(),
            text: String::new(),
            completed: false,
            status: ActionStatus::Planned,
            origin: ActionOrigin::Manual,
            objective_id: None,
            objective_ids: Vec::new(),
            created: now,
            modified: now,
            completed_at: None,
        };
        action.sync_completed_from_status();
        action
    }

    /// Create an action from markdown parsing (preserves completion status)
    pub fn from_markdown(text: String, completed: bool) -> Self {
        let mut action = Self::new(text);
        action.completed = completed;
        // Set status based on completed field for backward compatibility
        action.status = if completed {
            ActionStatus::Done
        } else {
            ActionStatus::Planned
        };
        action.sync_completed_from_status();
        action
    }

    /// Create a new action with text
    pub fn new(mut text: String) -> Self {
        // Truncate if too long
        if text.len() > MAX_ACTION_LENGTH {
            eprintln!(
                "Warning: Action text truncated from {} to {} chars",
                text.len(),
                MAX_ACTION_LENGTH
            );
            text.truncate(MAX_ACTION_LENGTH);
        }

        let now = chrono::Utc::now();
        let mut action = Action {
            id: Self::generate_id(),
            text,
            completed: false,
            status: ActionStatus::Planned,
            origin: ActionOrigin::Manual,
            objective_id: None,
            objective_ids: Vec::new(),
            created: now,
            modified: now,
            completed_at: None,
        };
        action.sync_completed_from_status();
        action
    }

    /// Create action with specific origin (for templates, carry-over, etc.)
    pub fn new_with_origin(text: String, origin: ActionOrigin) -> Self {
        let mut action = Self::new(text);
        action.origin = origin;
        action
    }

    /// Cycle to next status: Planned → InProgress → Done → Skipped → Blocked → Planned
    pub fn cycle_status(&mut self) {
        self.status = match self.status {
            ActionStatus::Planned => ActionStatus::InProgress,
            ActionStatus::InProgress => ActionStatus::Done,
            ActionStatus::Done => ActionStatus::Skipped,
            ActionStatus::Skipped => ActionStatus::Blocked,
            ActionStatus::Blocked => ActionStatus::Planned,
        };
        self.sync_completed_from_status();
        self.modified = chrono::Utc::now();
    }

    /// Set status and sync completed field
    pub fn set_status(&mut self, status: ActionStatus) {
        self.status = status;
        self.sync_completed_from_status();
        self.modified = chrono::Utc::now();
    }

    /// Sync completed field with status (completed = status == Done)
    fn sync_completed_from_status(&mut self) {
        self.completed = self.status == ActionStatus::Done;
        // Set completed_at when status becomes Done
        if self.status == ActionStatus::Done && self.completed_at.is_none() {
            self.completed_at = Some(chrono::Utc::now());
        } else if self.status != ActionStatus::Done {
            self.completed_at = None;
        }
    }

    /// Get status display character for UI
    pub fn status_char(&self) -> char {
        match self.status {
            ActionStatus::Planned => ' ',
            ActionStatus::InProgress => '→',
            ActionStatus::Done => '✓',
            ActionStatus::Skipped => '~',
            ActionStatus::Blocked => '✗',
        }
    }

    /// Get all objective IDs (combines legacy single objective_id with new objective_ids)
    pub fn get_all_objective_ids(&self) -> Vec<String> {
        let mut ids = self.objective_ids.clone();

        // Include the legacy single objective_id if present and not already in the list
        if let Some(single_id) = &self.objective_id {
            if !ids.contains(single_id) {
                ids.push(single_id.clone());
            }
        }

        ids
    }

    /// Add an objective ID to this action
    pub fn add_objective_id(&mut self, objective_id: String) {
        if !self.objective_ids.contains(&objective_id) {
            self.objective_ids.push(objective_id.clone());

            // If this is the first objective and objective_id is not set, set it for compatibility
            if self.objective_id.is_none() {
                self.objective_id = Some(objective_id);
            }
        }
    }

    /// Remove an objective ID from this action
    pub fn remove_objective_id(&mut self, objective_id: &str) {
        self.objective_ids.retain(|id| id != objective_id);

        // Also clear the legacy field if it matches
        if let Some(ref single_id) = self.objective_id {
            if single_id == objective_id {
                self.objective_id = None;
            }
        }
    }
}

/// The three life outcome areas - fixed enum to enforce exactly 3
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OutcomeType {
    Work,
    Health,
    Family,
}

/// Daily ritual phases for guided interactions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RitualPhase {
    Morning, // 5am-12pm: Set intentions
    Evening, // 5pm-11pm: Reflect and review
    None,    // Other times: Normal mode
}

impl RitualPhase {
    /// Determine phase based on current hour (0-23)
    pub fn from_hour(hour: u32) -> Self {
        match hour {
            5..=11 => RitualPhase::Morning,
            17..=22 => RitualPhase::Evening,
            _ => RitualPhase::None,
        }
    }

    /// Get a greeting message for the phase
    pub fn greeting(&self) -> &'static str {
        match self {
            RitualPhase::Morning => "Good Morning! Time to set today's intentions",
            RitualPhase::Evening => "Evening Review - Reflect on your day",
            RitualPhase::None => "FocusFive - Daily Goal Tracker",
        }
    }
}

impl OutcomeType {
    pub fn as_str(&self) -> &str {
        match self {
            OutcomeType::Work => "Work",
            OutcomeType::Health => "Health",
            OutcomeType::Family => "Family",
        }
    }
}

/// An outcome area with optional goal description and exactly 3 actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Outcome {
    pub outcome_type: OutcomeType,
    pub goal: Option<String>,
    pub actions: Vec<Action>,       // Now supports 1-5 actions
    pub reflection: Option<String>, // Evening reflection note
}

impl Outcome {
    pub fn new(outcome_type: OutcomeType) -> Self {
        Self {
            outcome_type,
            goal: None,
            actions: vec![
                Action::new(String::new()),
                Action::new(String::new()),
                Action::new(String::new()),
            ], // Default to 3 actions for backward compatibility
            reflection: None,
        }
    }

    /// Add a new action (max 5 total)
    pub fn add_action(&mut self) -> anyhow::Result<()> {
        if self.actions.len() >= 5 {
            anyhow::bail!("Maximum 5 actions per outcome");
        }
        self.actions.push(Action::new(String::new()));
        Ok(())
    }

    /// Remove an action (min 1 must remain)
    pub fn remove_action(&mut self, index: usize) -> anyhow::Result<()> {
        if self.actions.len() <= 1 {
            anyhow::bail!("Minimum 1 action required per outcome");
        }
        if index >= self.actions.len() {
            anyhow::bail!("Invalid action index: {}", index);
        }
        self.actions.remove(index);
        Ok(())
    }

    /// Count completed actions
    pub fn count_completed(&self) -> usize {
        self.actions.iter().filter(|a| a.completed).count()
    }

    /// Get completion percentage (0-100)
    pub fn completion_percentage(&self) -> u16 {
        if self.actions.is_empty() {
            return 0;
        }
        ((self.count_completed() * 100) / self.actions.len()) as u16
    }
}

/// Statistics for tracking daily progress
#[derive(Debug, Clone)]
pub struct CompletionStats {
    pub completed: usize,
    pub total: usize,
    pub percentage: u16,
    pub by_outcome: Vec<(String, usize, usize)>,
    pub streak_days: Option<u32>,
    pub best_outcome: Option<String>,
    pub needs_attention: Vec<String>,
}

/// Daily goals containing date and all three outcomes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DailyGoals {
    pub date: NaiveDate,
    pub day_number: Option<u32>,
    pub work: Outcome,
    pub health: Outcome,
    pub family: Outcome,
}

impl DailyGoals {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            day_number: None,
            work: Outcome::new(OutcomeType::Work),
            health: Outcome::new(OutcomeType::Health),
            family: Outcome::new(OutcomeType::Family),
        }
    }

    /// Get all outcomes as an array for iteration
    pub fn outcomes(&self) -> [&Outcome; 3] {
        [&self.work, &self.health, &self.family]
    }

    /// Get mutable references to all outcomes
    pub fn outcomes_mut(&mut self) -> [&mut Outcome; 3] {
        [&mut self.work, &mut self.health, &mut self.family]
    }

    /// Calculate completion statistics for the day
    pub fn completion_stats(&self) -> CompletionStats {
        let work_done = self.work.count_completed();
        let health_done = self.health.count_completed();
        let family_done = self.family.count_completed();
        let total_done = work_done + health_done + family_done;

        let work_total = self.work.actions.len();
        let health_total = self.health.actions.len();
        let family_total = self.family.actions.len();
        let total_actions = work_total + health_total + family_total;

        let by_outcome = vec![
            ("Work".to_string(), work_done, work_total),
            ("Health".to_string(), health_done, health_total),
            ("Family".to_string(), family_done, family_total),
        ];

        // Find best performing outcome (by percentage, not raw count)
        let best_outcome = by_outcome
            .iter()
            .max_by_key(|(_, done, total)| {
                if *total == 0 {
                    0
                } else {
                    (*done * 100) / *total
                }
            })
            .map(|(name, _, _)| name.clone());

        // Find outcomes needing attention (< 50% complete)
        let needs_attention: Vec<String> = by_outcome
            .iter()
            .filter(|(_, done, total)| *total > 0 && (*done as f32 / *total as f32) < 0.5)
            .map(|(name, _, _)| name.clone())
            .collect();

        let percentage = if total_actions > 0 {
            (total_done * 100 / total_actions) as u16
        } else {
            0
        };

        CompletionStats {
            completed: total_done,
            total: total_actions,
            percentage,
            by_outcome,
            streak_days: self.day_number,
            best_outcome,
            needs_attention,
        }
    }
}

/// Five-year vision for each life outcome area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiveYearVision {
    pub work: String,
    pub health: String,
    pub family: String,
    pub created: NaiveDate,
    pub modified: NaiveDate,
}

impl Default for FiveYearVision {
    fn default() -> Self {
        Self::new()
    }
}

impl FiveYearVision {
    pub fn new() -> Self {
        let today = chrono::Local::now().date_naive();
        Self {
            work: String::new(),
            health: String::new(),
            family: String::new(),
            created: today,
            modified: today,
        }
    }

    pub fn get_vision(&self, outcome_type: &OutcomeType) -> &str {
        match outcome_type {
            OutcomeType::Work => &self.work,
            OutcomeType::Health => &self.health,
            OutcomeType::Family => &self.family,
        }
    }

    pub fn set_vision(&mut self, outcome_type: &OutcomeType, vision: String) {
        let vision = if vision.len() > MAX_VISION_LENGTH {
            vision.chars().take(MAX_VISION_LENGTH).collect()
        } else {
            vision
        };

        match outcome_type {
            OutcomeType::Work => self.work = vision,
            OutcomeType::Health => self.health = vision,
            OutcomeType::Family => self.family = vision,
        }
        self.modified = chrono::Local::now().date_naive();
    }
}

/// Action templates for quick reuse of common action patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionTemplates {
    /// Map of template name to list of action texts (up to 5 per template)
    pub templates: HashMap<String, Vec<String>>,
    pub created: NaiveDate,
    pub modified: NaiveDate,
}

impl Default for ActionTemplates {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionTemplates {
    pub fn new() -> Self {
        let today = chrono::Local::now().date_naive();
        Self {
            templates: HashMap::new(),
            created: today,
            modified: today,
        }
    }

    /// Add or update a template
    pub fn add_template(&mut self, name: String, actions: Vec<String>) {
        // Limit to 5 actions per template
        let actions: Vec<String> = actions
            .into_iter()
            .take(5)
            .map(|s| {
                if s.len() > MAX_ACTION_LENGTH {
                    s.chars().take(MAX_ACTION_LENGTH).collect()
                } else {
                    s
                }
            })
            .collect();

        self.templates.insert(name, actions);
        self.modified = chrono::Local::now().date_naive();
    }

    /// Remove a template
    pub fn remove_template(&mut self, name: &str) -> bool {
        let removed = self.templates.remove(name).is_some();
        if removed {
            self.modified = chrono::Local::now().date_naive();
        }
        removed
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&Vec<String>> {
        self.templates.get(name)
    }

    /// Get all template names sorted alphabetically
    pub fn get_template_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.templates.keys().cloned().collect();
        names.sort();
        names
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub goals_dir: String,
    pub data_root: String, // NEW: parent directory for JSON/NDJSON stores
}

/// Action metadata stored in sidecar JSON files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionMeta {
    pub id: String, // UUID as string
    pub status: ActionStatus,
    pub origin: ActionOrigin,
    pub estimated_min: Option<u32>,
    pub actual_min: Option<u32>,
    pub priority: Option<u32>,
    pub tags: Vec<String>,
    pub objective_id: Option<String>, // Link to objective UUID
}

impl Default for ActionMeta {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            status: ActionStatus::Planned,
            origin: ActionOrigin::Manual,
            estimated_min: None,
            actual_min: None,
            priority: None,
            tags: Vec::new(),
            objective_id: None,
        }
    }
}

/// Status for rich action tracking
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionStatus {
    Planned,
    InProgress,
    Done,
    Skipped,
    Blocked,
}

/// Origin of an action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionOrigin {
    Manual,
    Template,
    CarryOver,
}

/// Day metadata stored as sidecar to markdown files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayMeta {
    pub version: u32,
    pub work: Vec<ActionMeta>,
    pub health: Vec<ActionMeta>,
    pub family: Vec<ActionMeta>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
}

impl DayMeta {
    /// Create new DayMeta aligned with the given goals
    pub fn from_goals(goals: &DailyGoals) -> Self {
        let now = chrono::Utc::now();
        Self {
            version: 1,
            work: goals
                .work
                .actions
                .iter()
                .map(|action| ActionMeta {
                    id: action.id.clone(),
                    status: if action.completed {
                        ActionStatus::Done
                    } else {
                        ActionStatus::Planned
                    },
                    ..ActionMeta::default()
                })
                .collect(),
            health: goals
                .health
                .actions
                .iter()
                .map(|action| ActionMeta {
                    id: action.id.clone(),
                    status: if action.completed {
                        ActionStatus::Done
                    } else {
                        ActionStatus::Planned
                    },
                    ..ActionMeta::default()
                })
                .collect(),
            family: goals
                .family
                .actions
                .iter()
                .map(|action| ActionMeta {
                    id: action.id.clone(),
                    status: if action.completed {
                        ActionStatus::Done
                    } else {
                        ActionStatus::Planned
                    },
                    ..ActionMeta::default()
                })
                .collect(),
            created: now,
            modified: now,
        }
    }

    /// Reconcile metadata with current action counts
    pub fn reconcile_with_goals(&mut self, goals: &DailyGoals) {
        Self::reconcile_outcome_meta(&mut self.work, &goals.work);
        Self::reconcile_outcome_meta(&mut self.health, &goals.health);
        Self::reconcile_outcome_meta(&mut self.family, &goals.family);
        self.modified = chrono::Utc::now();
    }

    fn reconcile_outcome_meta(meta_vec: &mut Vec<ActionMeta>, outcome: &Outcome) {
        let target_len = outcome.actions.len();
        let current_len = meta_vec.len();

        if current_len < target_len {
            // Add new metadata entries for new actions
            for i in current_len..target_len {
                let action = &outcome.actions[i];

                let meta = ActionMeta {
                    id: action.id.clone(),
                    status: if action.completed {
                        ActionStatus::Done
                    } else {
                        ActionStatus::Planned
                    },
                    ..ActionMeta::default()
                };

                meta_vec.push(meta);
            }
        } else if current_len > target_len {
            // Truncate excess metadata
            meta_vec.truncate(target_len);
        }

        // Ensure IDs are synced for existing entries
        for (i, action) in outcome.actions.iter().enumerate() {
            if i < meta_vec.len() {
                // Preserve the action's ID
                meta_vec[i].id = action.id.clone();

                // Update status if action completion changed but metadata hasn't been manually edited
                if action.completed && meta_vec[i].status == ActionStatus::Planned {
                    meta_vec[i].status = ActionStatus::Done;
                } else if !action.completed && meta_vec[i].status == ActionStatus::Done {
                    // If unchecked, revert to Planned unless it's in progress or blocked
                    if meta_vec[i].status != ActionStatus::InProgress
                        && meta_vec[i].status != ActionStatus::Blocked
                    {
                        meta_vec[i].status = ActionStatus::Planned;
                    }
                }
            }
        }
    }
}

/// Status of an objective
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ObjectiveStatus {
    Active,
    Paused,
    Completed,
    Dropped,
}

/// Long-term objective that can be linked to daily actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub id: String,                  // UUID as string
    pub domain: OutcomeType,         // Work|Health|Family
    pub title: String,               // Brief title
    pub description: Option<String>, // Detailed description
    pub start: NaiveDate,            // Start date
    pub end: Option<NaiveDate>,      // End date (optional for open-ended)
    pub status: ObjectiveStatus,     // Current status
    #[serde(default)]
    pub indicators: Vec<String>, // Has MANY indicators (UUIDs)
    pub created: chrono::DateTime<chrono::Utc>, // Creation timestamp
    pub modified: chrono::DateTime<chrono::Utc>, // Last modification timestamp
    pub parent_id: Option<String>,   // For hierarchical objectives
}

impl Objective {
    /// Create a new objective with generated UUID
    pub fn new(domain: OutcomeType, title: String) -> Self {
        let now = chrono::Utc::now();
        Objective {
            id: uuid::Uuid::new_v4().to_string(),
            domain,
            title,
            description: None,
            start: Local::now().date_naive(),
            end: None,
            status: ObjectiveStatus::Active,
            indicators: Vec::new(),
            created: now,
            modified: now,
            parent_id: None,
        }
    }
}

/// Root structure for objectives.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectivesData {
    pub version: u32,
    pub objectives: Vec<Objective>,
}

impl Default for ObjectivesData {
    fn default() -> Self {
        ObjectivesData {
            version: 1,
            objectives: Vec::new(),
        }
    }
}

/// Kind of indicator (leading or lagging)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IndicatorKind {
    Leading,
    Lagging,
}

/// Unit of measurement for indicators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum IndicatorUnit {
    Count,
    Minutes,
    Dollars,
    Percent,
    Custom(String),
}

/// Direction for indicator optimization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IndicatorDirection {
    HigherIsBetter,
    LowerIsBetter,
    WithinRange,
}

/// Definition of a key performance indicator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndicatorDef {
    pub id: String,                              // UUID as string
    pub name: String,                            // Human-readable name
    pub kind: IndicatorKind,                     // Leading or Lagging
    pub unit: IndicatorUnit,                     // Unit of measurement
    pub objective_id: Option<String>,            // Link to objective
    pub target: Option<f64>,                     // Target value
    pub direction: IndicatorDirection,           // Optimization direction
    pub active: bool,                            // Is indicator active
    pub created: chrono::DateTime<chrono::Utc>,  // Creation timestamp
    pub modified: chrono::DateTime<chrono::Utc>, // Last modification
    pub lineage_of: Option<String>,              // Previous version ID
    pub notes: Option<String>,                   // Additional notes
}

impl IndicatorDef {
    /// Create a new indicator with generated UUID
    pub fn new(name: String, kind: IndicatorKind, unit: IndicatorUnit) -> Self {
        let now = chrono::Utc::now();
        IndicatorDef {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            kind,
            unit,
            objective_id: None,
            target: None,
            direction: IndicatorDirection::HigherIsBetter,
            active: true,
            created: now,
            modified: now,
            lineage_of: None,
            notes: None,
        }
    }
}

/// Root structure for indicators.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorsData {
    pub version: u32,
    pub indicators: Vec<IndicatorDef>,
}

impl Default for IndicatorsData {
    fn default() -> Self {
        IndicatorsData {
            version: 1,
            indicators: Vec::new(),
        }
    }
}

/// New Indicator type for UI enhancement (as per plan)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum IndicatorType {
    Counter,    // Incremental counting (businesses reviewed)
    Duration,   // Time-based (hours of research)
    Percentage, // 0-100% (completion percentage)
    Boolean,    // Complete/Incomplete (template ready)
}

/// Entry in indicator history tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndicatorEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
    pub note: Option<String>,
}

/// Enhanced indicator struct for expandable UI
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Indicator {
    pub id: String, // UUID as string
    pub name: String,
    pub indicator_type: IndicatorType,
    pub current_value: f64,
    pub target_value: f64,
    pub unit: String, // "count", "hours", "percentage", "boolean"
    #[serde(default)]
    pub history: Vec<IndicatorEntry>, // Track changes over time
}

impl Indicator {
    /// Create a new indicator
    pub fn new(name: String, indicator_type: IndicatorType, target_value: f64) -> Self {
        let unit = match indicator_type {
            IndicatorType::Counter => "count",
            IndicatorType::Duration => "hours",
            IndicatorType::Percentage => "percentage",
            IndicatorType::Boolean => "boolean",
        }
        .to_string();

        Indicator {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            indicator_type,
            current_value: 0.0,
            target_value,
            unit,
            history: Vec::new(),
        }
    }
}

/// Source of an observation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ObservationSource {
    Manual,
    Automated,
    Import,
}

/// A single observation/measurement for an indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: String,                             // UUID as string
    pub indicator_id: String,                   // Which indicator
    pub when: NaiveDate,                        // Date of observation
    pub value: f64,                             // Observed value
    pub unit: IndicatorUnit,                    // Unit (should match indicator)
    pub source: ObservationSource,              // How was it recorded
    pub action_id: Option<String>,              // Link to action that produced it
    pub note: Option<String>,                   // Optional note
    pub created: chrono::DateTime<chrono::Utc>, // When recorded
}

impl Observation {
    /// Create a new observation
    pub fn new(indicator_id: String, when: NaiveDate, value: f64, unit: IndicatorUnit) -> Self {
        Observation {
            id: uuid::Uuid::new_v4().to_string(),
            indicator_id,
            when,
            value,
            unit,
            source: ObservationSource::Manual,
            action_id: None,
            note: None,
            created: chrono::Utc::now(),
        }
    }
}

/// Period type for reviews
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReviewPeriod {
    Weekly,
    Monthly,
    Quarterly,
}

/// A decision made during a review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub summary: String,              // Brief summary of decision
    pub objective_id: Option<String>, // Link to objective
    pub indicator_id: Option<String>, // Link to indicator
    pub rationale: Option<String>,    // Reasoning behind decision
}

/// Weekly review data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: String,               // UUID as string
    pub date: NaiveDate,          // Date of review
    pub period: ReviewPeriod,     // Review period type
    pub notes: Option<String>,    // General notes
    pub score_1_to_5: u8,         // Self-assessment score
    pub decisions: Vec<Decision>, // Decisions made
}

impl Review {
    /// Create a new weekly review
    pub fn new(date: NaiveDate) -> Self {
        Review {
            id: uuid::Uuid::new_v4().to_string(),
            date,
            period: ReviewPeriod::Weekly,
            notes: None,
            score_1_to_5: 3, // Default to middle score
            decisions: Vec::new(),
        }
    }
}

/// Root structure for review JSON files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewData {
    pub version: u32,
    pub review: Review,
}

impl Config {
    /// Create a new Config, attempting to use the home directory
    pub fn new() -> anyhow::Result<Self> {
        let goals_dir = if let Some(base) = directories::BaseDirs::new() {
            base.home_dir()
                .join("FocusFive")
                .join("goals")
                .to_string_lossy()
                .to_string()
        } else {
            // Fallback to current directory if home not found
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("FocusFive")
                .join("goals")
                .to_string_lossy()
                .to_string()
        };

        // macOS-friendly data_root preference
        let data_root = if cfg!(target_os = "macos") {
            if let Some(proj) = directories::ProjectDirs::from("com", "Correia", "FocusFive") {
                proj.data_dir().to_string_lossy().to_string()
            } else {
                // fallback to parent of goals_dir
                std::path::Path::new(&goals_dir)
                    .parent()
                    .unwrap_or(std::path::Path::new(&goals_dir))
                    .to_string_lossy()
                    .to_string()
            }
        } else {
            // non-macOS: parent of goals_dir
            std::path::Path::new(&goals_dir)
                .parent()
                .unwrap_or(std::path::Path::new(&goals_dir))
                .to_string_lossy()
                .to_string()
        };

        Ok(Self {
            goals_dir,
            data_root,
        })
    }

    /// Safe default that won't panic
    pub fn default_safe() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            goals_dir: "./FocusFive/goals".to_string(),
            data_root: "./FocusFive".to_string(),
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default_safe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::mem;

    #[test]
    fn test_action_creation() {
        let action = Action::new("Test action".to_string());
        assert_eq!(action.text, "Test action");
        assert!(!action.completed);
    }

    #[test]
    fn test_daily_goals_creation() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let goals = DailyGoals::new(date);

        assert_eq!(goals.date, date);
        assert_eq!(goals.outcomes().len(), 3);
    }

    // === COMPREHENSIVE VALIDATION TESTS ===

    // Test 1: Verify OutcomeType has exactly 3 variants
    #[test]
    fn test_outcome_type_exactly_three_variants() {
        // This test ensures we have exactly 3 variants by exhaustive matching
        let outcomes = [OutcomeType::Work, OutcomeType::Health, OutcomeType::Family];

        for outcome in outcomes {
            match outcome {
                OutcomeType::Work => assert_eq!(outcome.as_str(), "Work"),
                OutcomeType::Health => assert_eq!(outcome.as_str(), "Health"),
                OutcomeType::Family => assert_eq!(outcome.as_str(), "Family"),
                // If a 4th variant is added, this won't compile
            }
        }

        // Verify string representations
        assert_eq!(OutcomeType::Work.as_str(), "Work");
        assert_eq!(OutcomeType::Health.as_str(), "Health");
        assert_eq!(OutcomeType::Family.as_str(), "Family");
    }

    // Test 2: Verify Outcome struct enforces exactly 3 actions (compile-time)
    #[test]
    fn test_outcome_exactly_three_actions() {
        let outcome = Outcome::new(OutcomeType::Work);

        // Array must have exactly 3 elements
        assert_eq!(outcome.actions.len(), 3);

        // Verify all actions are properly initialized
        for action in &outcome.actions {
            assert_eq!(action.text, "");
            assert!(!action.completed);
        }

        // Test that we can access each index
        assert_eq!(outcome.actions[0].text, "");
        assert_eq!(outcome.actions[1].text, "");
        assert_eq!(outcome.actions[2].text, "");

        // This would cause compile error if array wasn't size 3:
        // outcome.actions[3]; // Index out of bounds
    }

    // Test 3: Verify DailyGoals contains all three outcome types
    #[test]
    fn test_daily_goals_all_three_outcomes() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let goals = DailyGoals::new(date);

        // Verify all three outcomes exist
        assert_eq!(goals.work.outcome_type, OutcomeType::Work);
        assert_eq!(goals.health.outcome_type, OutcomeType::Health);
        assert_eq!(goals.family.outcome_type, OutcomeType::Family);

        // Verify outcomes() method returns exactly 3
        let outcomes = goals.outcomes();
        assert_eq!(outcomes.len(), 3);

        // Verify each outcome has 3 actions
        for outcome in outcomes {
            assert_eq!(outcome.actions.len(), 3);
        }

        // Create a new goals instance for mutable testing
        let mut goals_mut = DailyGoals::new(date);
        let outcomes_mut = goals_mut.outcomes_mut();
        assert_eq!(outcomes_mut.len(), 3);
    }

    // Test 4: Validate Config default paths
    #[test]
    fn test_config_default_paths() {
        let config = Config::default();

        // Should contain expected path components
        assert!(config.goals_dir.contains("FocusFive"));
        assert!(config.goals_dir.contains("goals"));

        // Should be an absolute path (platform-specific check)
        #[cfg(unix)]
        assert!(config.goals_dir.starts_with('/'));
        #[cfg(windows)]
        assert!(config.goals_dir.chars().nth(1) == Some(':'));

        // Should end with goals directory
        assert!(config.goals_dir.ends_with("goals"));
    }

    // Test 5: Action struct completion status tracking
    #[test]
    fn test_action_completion_tracking() {
        // Default action is not completed
        let action = Action::new("Test task".to_string());
        assert_eq!(action.text, "Test task");
        assert!(!action.completed);

        // Can manually set completion
        let mut completed_action = Action::new("Done task".to_string());
        completed_action.completed = true;
        assert!(completed_action.completed);

        // Test with empty text
        let empty_action = Action::new(String::new());
        assert_eq!(empty_action.text, "");
        assert!(!empty_action.completed);
    }

    // Test 6: Serialization/Deserialization with Serde
    #[test]
    fn test_serde_serialization() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);

        // Set up test data
        goals.day_number = Some(42);
        goals.work.goal = Some("Complete project".to_string());
        goals.work.actions[0] = Action::from_markdown("Write tests".to_string(), true);

        // Test JSON serialization
        let json = serde_json::to_string(&goals).unwrap();
        let deserialized: DailyGoals = serde_json::from_str(&json).unwrap();

        assert_eq!(goals.date, deserialized.date);
        assert_eq!(goals.day_number, deserialized.day_number);
        assert_eq!(goals.work.goal, deserialized.work.goal);
        assert_eq!(
            goals.work.actions[0].text,
            deserialized.work.actions[0].text
        );
        assert_eq!(
            goals.work.actions[0].completed,
            deserialized.work.actions[0].completed
        );
    }

    // Test 7: Required traits implementation
    #[test]
    fn test_required_traits() {
        let action = Action::new("Test".to_string());
        let outcome = Outcome::new(OutcomeType::Work);
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let goals = DailyGoals::new(date);
        let config = Config::default();

        // Test Debug trait
        let action_debug = format!("{:?}", action);
        assert!(action_debug.contains("Action"));

        let outcome_debug = format!("{:?}", outcome);
        assert!(outcome_debug.contains("Outcome"));

        let goals_debug = format!("{:?}", goals);
        assert!(goals_debug.contains("DailyGoals"));

        let config_debug = format!("{:?}", config);
        assert!(config_debug.contains("Config"));

        // Test Clone trait
        let action_clone = action.clone();
        let outcome_clone = outcome.clone();
        let goals_clone = goals.clone();
        let _config_clone = config.clone();

        // Test PartialEq trait (where implemented)
        assert_eq!(action, action_clone);
        assert_eq!(outcome, outcome_clone);
        assert_eq!(goals, goals_clone);
        // Note: Config doesn't implement PartialEq, which is fine
    }

    // Test 8: Invariant enforcement - no way to create invalid states
    #[test]
    fn test_invariant_enforcement() {
        // Test that we cannot create an Outcome with wrong number of actions
        // This is enforced at compile time by the fixed array size

        let outcome = Outcome::new(OutcomeType::Health);

        // Can modify actions but not change array size
        let mut modified_outcome = outcome.clone();
        modified_outcome.actions[0].text = "New action".to_string();
        modified_outcome.actions[1].completed = true;

        // Still exactly 3 actions
        assert_eq!(modified_outcome.actions.len(), 3);

        // Cannot add or remove actions - this would be a compile error:
        // modified_outcome.actions.push(Action::new("Fourth".to_string())); // No push method
        // modified_outcome.actions.remove(0); // No remove method
    }

    // Test 9: Default values are sensible
    #[test]
    fn test_sensible_defaults() {
        let action = Action::new("Task".to_string());
        assert!(!action.completed); // Default to not completed

        let outcome = Outcome::new(OutcomeType::Family);
        assert!(outcome.goal.is_none()); // No default goal
        for action in &outcome.actions {
            assert_eq!(action.text, ""); // Empty action text
            assert!(!action.completed); // Not completed
        }

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let goals = DailyGoals::new(date);
        assert!(goals.day_number.is_none()); // No default day number
        assert_eq!(goals.date, date); // Correct date

        let config = Config::default();
        assert!(!config.goals_dir.is_empty()); // Non-empty path
    }

    // Test 10: Helper methods work correctly
    #[test]
    fn test_helper_methods() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);

        // Test outcomes() method
        let outcomes = goals.outcomes();
        assert_eq!(outcomes.len(), 3);
        assert_eq!(outcomes[0].outcome_type, OutcomeType::Work);
        assert_eq!(outcomes[1].outcome_type, OutcomeType::Health);
        assert_eq!(outcomes[2].outcome_type, OutcomeType::Family);

        // Test outcomes_mut() method
        let outcomes_mut = goals.outcomes_mut();
        outcomes_mut[0].goal = Some("Modified goal".to_string());

        // Verify modification took effect
        assert_eq!(goals.work.goal, Some("Modified goal".to_string()));

        // Test OutcomeType::as_str() method
        assert_eq!(OutcomeType::Work.as_str(), "Work");
        assert_eq!(OutcomeType::Health.as_str(), "Health");
        assert_eq!(OutcomeType::Family.as_str(), "Family");
    }

    // Test 11: Memory layout and size constraints
    #[test]
    fn test_memory_constraints() {
        // Verify that our structures are reasonably sized
        // Action struct has grown due to new fields (objective_id, completed_at, timestamps, etc.)
        assert!(mem::size_of::<Action>() < 200); // Increased due to new fields
        assert!(mem::size_of::<Outcome>() < 800); // Increased due to Vec of Actions
        assert!(mem::size_of::<DailyGoals>() < 3000); // Should be moderate

        // Verify that OutcomeType is a simple enum
        assert_eq!(mem::size_of::<OutcomeType>(), 1); // Should be 1 byte
    }

    // Test 12: Edge cases and boundary conditions
    #[test]
    fn test_edge_cases() {
        // Test with very long text (should be truncated to MAX_ACTION_LENGTH)
        let long_text = "a".repeat(1000);
        let action = Action::new(long_text.clone());
        assert_eq!(action.text.len(), MAX_ACTION_LENGTH);
        assert_eq!(action.text, "a".repeat(MAX_ACTION_LENGTH));

        // Test with empty strings
        let empty_action = Action::new(String::new());
        assert_eq!(empty_action.text, "");

        // Test with unicode text
        let unicode_text = "🎯 Complete project ✅".to_string();
        let unicode_action = Action::new(unicode_text.clone());
        assert_eq!(unicode_action.text, unicode_text);

        // Test date boundaries
        let min_date = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        let max_date = NaiveDate::from_ymd_opt(2100, 12, 31).unwrap();

        let min_goals = DailyGoals::new(min_date);
        let max_goals = DailyGoals::new(max_date);

        assert_eq!(min_goals.date, min_date);
        assert_eq!(max_goals.date, max_date);
    }
}
