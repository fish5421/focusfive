use chrono::NaiveDate;
use focusfive::app::App;
use focusfive::models::{
    CompletionStats, Config, DailyGoals, FiveYearVision, Outcome, RitualPhase,
};
use std::fs;
use tempfile::TempDir;

fn setup_test_config() -> (Config, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let goals_dir = temp_dir.path().join("FocusFive").join("goals");
    fs::create_dir_all(&goals_dir).expect("Failed to create goals dir");

    let config = Config {
        goals_dir: goals_dir.to_str().unwrap().to_string(),
    };

    (config, temp_dir)
}

#[test]
fn test_outcome_completion_tracking() {
    let mut outcome = Outcome::new(focusfive::models::OutcomeType::Work);

    // Initially no completions
    assert_eq!(outcome.count_completed(), 0);
    assert_eq!(outcome.completion_percentage(), 0);

    // Mark one completed
    outcome.actions[0].text = "Task 1".to_string();
    outcome.actions[0].completed = true;
    assert_eq!(outcome.count_completed(), 1);
    assert_eq!(outcome.completion_percentage(), 33);

    // Mark two completed
    outcome.actions[1].text = "Task 2".to_string();
    outcome.actions[1].completed = true;
    assert_eq!(outcome.count_completed(), 2);
    assert_eq!(outcome.completion_percentage(), 66);

    // Mark all completed
    outcome.actions[2].text = "Task 3".to_string();
    outcome.actions[2].completed = true;
    assert_eq!(outcome.count_completed(), 3);
    assert_eq!(outcome.completion_percentage(), 100);
}

#[test]
fn test_daily_goals_completion_stats() {
    let mut goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    goals.day_number = Some(5);

    // Set up some completed tasks
    goals.work.actions[0].text = "Work task 1".to_string();
    goals.work.actions[0].completed = true;
    goals.work.actions[1].text = "Work task 2".to_string();
    goals.work.actions[1].completed = true;

    goals.health.actions[0].text = "Health task 1".to_string();
    goals.health.actions[0].completed = true;

    // Calculate stats
    let stats = goals.completion_stats();

    assert_eq!(stats.completed, 3);
    assert_eq!(stats.total, 9);
    assert_eq!(stats.percentage, 33);
    assert_eq!(stats.streak_days, Some(5));

    // Check by_outcome breakdown
    assert_eq!(stats.by_outcome.len(), 3);
    assert_eq!(stats.by_outcome[0].0, "Work");
    assert_eq!(stats.by_outcome[0].1, 2); // 2 completed
    assert_eq!(stats.by_outcome[0].2, 3); // out of 3
}

#[test]
fn test_completion_stats_best_outcome() {
    let mut goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());

    // Health has all tasks completed
    goals.health.actions[0].text = "Task 1".to_string();
    goals.health.actions[0].completed = true;
    goals.health.actions[1].text = "Task 2".to_string();
    goals.health.actions[1].completed = true;
    goals.health.actions[2].text = "Task 3".to_string();
    goals.health.actions[2].completed = true;

    // Work has one completed
    goals.work.actions[0].text = "Work task".to_string();
    goals.work.actions[0].completed = true;

    let stats = goals.completion_stats();

    assert_eq!(stats.best_outcome, Some("Health".to_string()));
    assert_eq!(stats.needs_attention.len(), 2); // Work and Family need attention
    assert!(stats.needs_attention.contains(&"Work".to_string()));
    assert!(stats.needs_attention.contains(&"Family".to_string()));
}

#[test]
fn test_outcome_reflection_field() {
    let mut outcome = Outcome::new(focusfive::models::OutcomeType::Work);

    // Initially no reflection
    assert_eq!(outcome.reflection, None);

    // Add reflection
    outcome.reflection = Some("Today was productive!".to_string());
    assert_eq!(
        outcome.reflection,
        Some("Today was productive!".to_string())
    );
}

#[test]
fn test_app_morning_phase_initialization() {
    let (config, _temp_dir) = setup_test_config();
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    let vision = FiveYearVision::new();

    // Manually set morning phase for testing
    let mut app = App::new(goals, config, vision);
    app.ritual_phase = RitualPhase::Morning;
    app.show_morning_prompt = true;

    assert_eq!(app.ritual_phase, RitualPhase::Morning);
    assert!(app.show_morning_prompt);
}

#[test]
fn test_app_evening_phase_initialization() {
    let (config, _temp_dir) = setup_test_config();
    let mut goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());

    // Add some completed tasks
    goals.work.actions[0].completed = true;
    goals.health.actions[0].completed = true;

    let vision = FiveYearVision::new();

    // Create app and manually set evening phase
    let mut app = App::new(goals, config, vision);
    app.ritual_phase = RitualPhase::Evening;
    app.completion_stats = Some(app.goals.completion_stats());

    assert_eq!(app.ritual_phase, RitualPhase::Evening);
    assert!(app.completion_stats.is_some());

    if let Some(stats) = app.completion_stats {
        assert_eq!(stats.completed, 2);
        assert_eq!(stats.percentage, 22); // 2/9
    }
}

#[test]
fn test_toggle_action_by_index() {
    let (config, _temp_dir) = setup_test_config();
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    let vision = FiveYearVision::new();

    let mut app = App::new(goals, config, vision);

    // Add some actions
    app.goals.work.actions[0].text = "Work task 1".to_string();
    app.goals.work.actions[1].text = "Work task 2".to_string();
    app.goals.health.actions[0].text = "Health task 1".to_string();

    // Toggle index 0 (Work action 0)
    app.toggle_action_by_index(0);
    assert!(app.goals.work.actions[0].completed);

    // Toggle index 3 (Health action 0)
    app.toggle_action_by_index(3);
    assert!(app.goals.health.actions[0].completed);

    // Toggle index 0 again to uncheck
    app.toggle_action_by_index(0);
    assert!(!app.goals.work.actions[0].completed);
}

#[test]
fn test_apply_yesterday_incomplete() {
    let (config, _temp_dir) = setup_test_config();
    let mut goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 16).unwrap());
    let vision = FiveYearVision::new();

    let mut app = App::new(goals, config, vision);

    // Create yesterday's goals with some incomplete tasks
    let mut yesterday = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    yesterday.work.actions[0].text = "Incomplete work task".to_string();
    yesterday.work.actions[0].completed = false;
    yesterday.work.actions[1].text = "Completed work task".to_string();
    yesterday.work.actions[1].completed = true;
    yesterday.health.actions[0].text = "Incomplete health task".to_string();
    yesterday.health.actions[0].completed = false;

    // Apply yesterday's incomplete tasks
    app.apply_yesterday_incomplete(&yesterday);

    // Check that incomplete tasks were copied
    assert_eq!(app.goals.work.actions[0].text, "Incomplete work task");
    assert_eq!(app.goals.work.actions[1].text, ""); // Completed task not copied
    assert_eq!(app.goals.health.actions[0].text, "Incomplete health task");
}

#[test]
fn test_generate_daily_summary() {
    let (config, _temp_dir) = setup_test_config();
    let mut goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    goals.day_number = Some(10);

    // Set up some completed tasks and reflections
    goals.work.actions[0].completed = true;
    goals.work.actions[1].completed = true;
    goals.work.reflection = Some("Good progress on project".to_string());

    goals.health.actions[0].completed = true;
    goals.health.actions[1].completed = true;
    goals.health.actions[2].completed = true;
    goals.health.reflection = Some("Great workout day!".to_string());

    let vision = FiveYearVision::new();
    let mut app = App::new(goals, config, vision);

    // Generate summary
    app.generate_daily_summary();

    // Check summary content
    assert!(app.daily_summary.contains("Day 10"));
    assert!(app.daily_summary.contains("5/9")); // 5 completed out of 9
    assert!(app.daily_summary.contains("55%")); // percentage
    assert!(app.daily_summary.contains("Work: 2/3"));
    assert!(app.daily_summary.contains("Health: 3/3"));
    assert!(app.daily_summary.contains("Reflections"));
    assert!(app.daily_summary.contains("Good progress on project"));
    assert!(app.daily_summary.contains("Great workout day!"));
}

#[test]
fn test_reflection_serialization() {
    let mut outcome = Outcome::new(focusfive::models::OutcomeType::Work);
    outcome.reflection = Some("Test reflection".to_string());

    // Serialize
    let json = serde_json::to_string(&outcome).unwrap();
    assert!(json.contains("reflection"));
    assert!(json.contains("Test reflection"));

    // Deserialize
    let deserialized: Outcome = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.reflection, Some("Test reflection".to_string()));
}

#[test]
fn test_reflection_serializes_as_null() {
    let outcome = Outcome::new(focusfive::models::OutcomeType::Work);

    // Serialize with no reflection
    let json = serde_json::to_string(&outcome).unwrap();

    // Should include reflection field as null when None (no backward compatibility)
    assert!(json.contains("\"reflection\":null"));
}
