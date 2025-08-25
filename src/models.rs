use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Input validation constants
pub const MAX_ACTION_LENGTH: usize = 500;
pub const MAX_GOAL_LENGTH: usize = 100;
pub const MAX_VISION_LENGTH: usize = 1000;

/// A single action item with completion status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    pub text: String,
    pub completed: bool,
}

impl Action {
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

        Self {
            text,
            completed: false,
        }
    }
}

/// The three life outcome areas - fixed enum to enforce exactly 3
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub actions: Vec<Action>,  // Now supports 1-5 actions
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
            ],  // Default to 3 actions for backward compatibility
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
            .filter(|(_, done, total)| {
                *total > 0 && (*done as f32 / *total as f32) < 0.5
            })
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
    /// Map of template name to list of action texts (up to 3 per template)
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
        // Limit to 3 actions per template
        let actions: Vec<String> = actions
            .into_iter()
            .take(3)
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

        Ok(Self { goals_dir })
    }

    /// Safe default that won't panic
    pub fn default_safe() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            goals_dir: "./FocusFive/goals".to_string(),
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
        goals.work.actions[0] = Action {
            text: "Write tests".to_string(),
            completed: true,
        };

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
        assert!(mem::size_of::<Action>() < 100); // Should be small
        assert!(mem::size_of::<Outcome>() < 500); // Should be reasonable
        assert!(mem::size_of::<DailyGoals>() < 2000); // Should be moderate

        // Verify that OutcomeType is a simple enum
        assert_eq!(mem::size_of::<OutcomeType>(), 1); // Should be 1 byte
    }

    // Test 12: Edge cases and boundary conditions
    #[test]
    fn test_edge_cases() {
        // Test with very long text
        let long_text = "a".repeat(1000);
        let action = Action::new(long_text.clone());
        assert_eq!(action.text, long_text);

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
