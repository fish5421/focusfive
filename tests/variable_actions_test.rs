use anyhow::Result;
use chrono::NaiveDate;
use focusfive::app::App;
use focusfive::models::{Action, Config, DailyGoals, FiveYearVision, Outcome, OutcomeType};
use std::fs;
use tempfile::TempDir;

fn setup_test_app() -> (App, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let goals_dir = temp_dir.path().join("FocusFive").join("goals");
    fs::create_dir_all(&goals_dir).expect("Failed to create goals dir");

    let config = Config {
        goals_dir: goals_dir.to_str().unwrap().to_string(),
        data_root: temp_dir.path().to_str().unwrap().to_string(),
    };

    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    let vision = FiveYearVision::new();

    let app = App::new(goals, config, vision);
    (app, temp_dir)
}

#[test]
fn test_outcome_starts_with_three_actions() {
    let outcome = Outcome::new(OutcomeType::Work);
    assert_eq!(outcome.actions.len(), 3);
    assert!(outcome.actions.iter().all(|a| a.text.is_empty()));
}

#[test]
fn test_add_action_up_to_five() {
    let mut outcome = Outcome::new(OutcomeType::Work);

    // Can add 4th action
    assert!(outcome.add_action().is_ok());
    assert_eq!(outcome.actions.len(), 4);

    // Can add 5th action
    assert!(outcome.add_action().is_ok());
    assert_eq!(outcome.actions.len(), 5);

    // Cannot add 6th action
    assert!(outcome.add_action().is_err());
    assert_eq!(outcome.actions.len(), 5);
}

#[test]
fn test_remove_action_minimum_one() {
    let mut outcome = Outcome::new(OutcomeType::Work);

    // Remove 2 actions (from 3 to 1)
    assert!(outcome.remove_action(2).is_ok());
    assert_eq!(outcome.actions.len(), 2);

    assert!(outcome.remove_action(1).is_ok());
    assert_eq!(outcome.actions.len(), 1);

    // Cannot remove last action
    assert!(outcome.remove_action(0).is_err());
    assert_eq!(outcome.actions.len(), 1);
}

#[test]
fn test_remove_action_invalid_index() {
    let mut outcome = Outcome::new(OutcomeType::Work);

    // Try to remove non-existent action
    assert!(outcome.remove_action(5).is_err());
    assert_eq!(outcome.actions.len(), 3);
}

#[test]
fn test_completion_percentage_with_variable_actions() {
    let mut outcome = Outcome::new(OutcomeType::Health);

    // 3 actions, 1 completed = 33%
    outcome.actions[0].completed = true;
    assert_eq!(outcome.completion_percentage(), 33);

    // Add 4th action, still 1 completed = 25%
    outcome.add_action().unwrap();
    assert_eq!(outcome.completion_percentage(), 25);

    // Add 5th action, still 1 completed = 20%
    outcome.add_action().unwrap();
    assert_eq!(outcome.completion_percentage(), 20);

    // Complete all 5 = 100%
    for action in &mut outcome.actions {
        action.completed = true;
    }
    assert_eq!(outcome.completion_percentage(), 100);
}

#[test]
fn test_completion_stats_with_variable_actions() {
    let mut goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());

    // Add extra actions to work
    goals.work.add_action().unwrap();
    goals.work.add_action().unwrap(); // Now has 5 actions

    // Remove actions from health
    goals.health.remove_action(2).unwrap(); // Now has 2 actions

    // Family stays at 3 actions

    // Total should be 5 + 2 + 3 = 10 actions
    let stats = goals.completion_stats();
    assert_eq!(stats.total, 10);

    // Complete some actions
    goals.work.actions[0].completed = true;
    goals.work.actions[1].completed = true;
    goals.health.actions[0].completed = true;

    let stats = goals.completion_stats();
    assert_eq!(stats.completed, 3);
    assert_eq!(stats.total, 10);
    assert_eq!(stats.percentage, 30); // 3/10 = 30%
}

#[test]
fn test_parser_handles_variable_actions() {
    use focusfive::data::parse_markdown;

    let markdown = r#"# January 15, 2025

## Work
- [x] Task 1
- [ ] Task 2
- [ ] Task 3
- [ ] Task 4
- [ ] Task 5

## Health
- [x] Exercise

## Family
- [ ] Call parents
- [ ] Plan dinner
"#;

    let goals = parse_markdown(markdown).unwrap();

    // Work should have 5 actions
    assert_eq!(goals.work.actions.len(), 5);
    assert_eq!(goals.work.actions[0].text, "Task 1");
    assert_eq!(goals.work.actions[4].text, "Task 5");

    // Health should have 3 actions (parser fills to minimum 3)
    assert_eq!(goals.health.actions.len(), 3);
    assert_eq!(goals.health.actions[0].text, "Exercise");
    assert_eq!(goals.health.actions[1].text, "");

    // Family should have 3 actions
    assert_eq!(goals.family.actions.len(), 3);
    assert_eq!(goals.family.actions[0].text, "Call parents");
    assert_eq!(goals.family.actions[1].text, "Plan dinner");
}

#[test]
fn test_parser_rejects_more_than_five_actions() {
    use focusfive::data::parse_markdown;

    let markdown = r#"# January 15, 2025

## Work
- [ ] Task 1
- [ ] Task 2
- [ ] Task 3
- [ ] Task 4
- [ ] Task 5
- [ ] Task 6
- [ ] Task 7

## Health
- [ ] Exercise

## Family
- [ ] Call parents
"#;

    let goals = parse_markdown(markdown).unwrap();

    // Work should only have 5 actions (6 and 7 ignored)
    assert_eq!(goals.work.actions.len(), 5);
    assert_eq!(goals.work.actions[4].text, "Task 5");
}

#[test]
fn test_markdown_generation_with_variable_actions() {
    use focusfive::data::generate_markdown;

    let mut goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());

    // Work with 4 actions
    goals.work.add_action().unwrap();
    goals.work.actions[0].text = "Task 1".to_string();
    goals.work.actions[1].text = "Task 2".to_string();
    goals.work.actions[2].text = "Task 3".to_string();
    goals.work.actions[3].text = "Task 4".to_string();
    goals.work.actions[0].completed = true;

    // Health with 2 actions (remove one)
    goals.health.remove_action(2).unwrap();
    goals.health.actions[0].text = "Exercise".to_string();
    goals.health.actions[1].text = "Meditate".to_string();

    // Family with 3 actions (default)
    goals.family.actions[0].text = "Call parents".to_string();

    let markdown = generate_markdown(&goals);

    // Check Work section has 4 actions
    assert!(markdown.contains("- [x] Task 1"));
    assert!(markdown.contains("- [ ] Task 2"));
    assert!(markdown.contains("- [ ] Task 3"));
    assert!(markdown.contains("- [ ] Task 4"));

    // Check Health section has 2 actions
    assert!(markdown.contains("- [ ] Exercise"));
    assert!(markdown.contains("- [ ] Meditate"));

    // Check Family section has 3 actions (with 2 empty)
    assert!(markdown.contains("- [ ] Call parents"));
    assert!(markdown.contains("- [ ] \n- [ ]")); // Two empty actions
}

#[test]
fn test_navigation_with_variable_actions() {
    let (mut app, _temp_dir) = setup_test_app();

    // Add actions to work
    app.goals.work.add_action().unwrap();
    app.goals.work.add_action().unwrap(); // 5 actions

    // Remove actions from health
    app.goals.health.remove_action(2).unwrap();
    app.goals.health.remove_action(1).unwrap(); // 1 action

    // Test navigation bounds
    app.outcome_index = 0; // Work
    app.action_index = 4;
    assert_eq!(app.action_index, 4); // Can navigate to 5th action

    app.outcome_index = 1; // Health
    app.action_index = 0;
    // Should only have 1 action, so action_index should stay at 0
    assert_eq!(app.action_index, 0);
}

#[test]
fn test_global_index_toggle() {
    let (mut app, _temp_dir) = setup_test_app();

    // Set up variable actions
    app.goals.work.add_action().unwrap(); // 4 actions
    app.goals.health.remove_action(2).unwrap(); // 2 actions
                                                // Family has 3 actions
                                                // Total: 4 + 2 + 3 = 9 actions

    // Toggle action at global index 5 (should be Family action 0)
    app.toggle_action_by_global_index(6);
    assert!(app.goals.family.actions[0].completed);

    // Toggle action at global index 3 (should be Work action 3)
    app.toggle_action_by_global_index(3);
    assert!(app.goals.work.actions[3].completed);
}

#[test]
fn test_copy_from_yesterday_with_variable_actions() {
    let (mut app, _temp_dir) = setup_test_app();

    // Create yesterday's goals with variable actions
    let mut yesterday = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 14).unwrap());

    // Yesterday had 4 work actions
    yesterday.work.add_action().unwrap();
    yesterday.work.actions[0].text = "Work 1".to_string();
    yesterday.work.actions[1].text = "Work 2".to_string();
    yesterday.work.actions[2].text = "Work 3".to_string();
    yesterday.work.actions[3].text = "Work 4".to_string();
    yesterday.work.actions[1].completed = false; // Incomplete

    // Apply incomplete from yesterday
    app.apply_yesterday_incomplete(&yesterday);

    // Should have copied the incomplete task
    assert_eq!(app.goals.work.actions[1].text, "Work 2");
}

#[test]
fn test_empty_outcome_handling() {
    let mut outcome = Outcome::new(OutcomeType::Work);

    // Remove all but one action
    outcome.remove_action(2).unwrap();
    outcome.remove_action(1).unwrap();

    assert_eq!(outcome.actions.len(), 1);
    assert_eq!(outcome.completion_percentage(), 0);

    // Complete the single action
    outcome.actions[0].completed = true;
    assert_eq!(outcome.completion_percentage(), 100);
}
