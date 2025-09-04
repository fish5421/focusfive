use chrono::Local;
use focusfive::models::{Action, DailyGoals, Outcome, OutcomeType};

#[test]
fn test_dynamic_action_count_in_stats() {
    // Create goals with variable action counts
    let mut goals = DailyGoals {
        date: Local::now().date_naive(),
        day_number: Some(1),
        work: Outcome {
            outcome_type: OutcomeType::Work,
            goal: Some("Test variable actions".to_string()),
            actions: vec![
                Action::new("Task 1".to_string()),
                Action::new("Task 2".to_string()),
            ],
            reflection: None,
        },
        health: Outcome {
            outcome_type: OutcomeType::Health,
            goal: Some("Stay healthy".to_string()),
            actions: vec![
                Action::new("Exercise".to_string()),
                Action::new("Drink water".to_string()),
                Action::new("Sleep well".to_string()),
            ],
            reflection: None,
        },
        family: Outcome {
            outcome_type: OutcomeType::Family,
            goal: Some("Family time".to_string()),
            actions: vec![
                Action::new("Dinner together".to_string()),
                Action::new("Game night".to_string()),
                Action::new("Movie time".to_string()),
                Action::new("Walk in park".to_string()),
            ],
            reflection: None,
        },
    };

    // Test 1: Initial stats (2 + 3 + 4 = 9 actions)
    let stats = goals.completion_stats();
    assert_eq!(stats.total, 9, "Should have 9 total actions initially");
    assert_eq!(stats.completed, 0, "Should have 0 completed initially");

    // Complete some actions
    goals.work.actions[0].completed = true;
    goals.health.actions[1].completed = true;

    // Test 2: After completing 2 actions
    let stats = goals.completion_stats();
    assert_eq!(stats.completed, 2, "Should have 2 completed");
    assert_eq!(stats.total, 9, "Should still have 9 total");

    // Add an action to work
    goals.work.add_action().unwrap();

    // Test 3: After adding action (3 + 3 + 4 = 10 actions)
    let stats = goals.completion_stats();
    assert_eq!(stats.total, 10, "Should have 10 total after adding");

    // Remove an action from family
    goals.family.remove_action(3).unwrap();

    // Test 4: After removing action (3 + 3 + 3 = 9 actions)
    let stats = goals.completion_stats();
    assert_eq!(stats.total, 9, "Should have 9 total after removing");

    // Test 5: Add max actions to health
    goals.health.add_action().unwrap();
    goals.health.add_action().unwrap();

    // Should now have 3 + 5 + 3 = 11 actions
    let stats = goals.completion_stats();
    assert_eq!(
        stats.total, 11,
        "Should have 11 total with max health actions"
    );

    // Test percentage calculation with variable totals
    goals.work.actions[1].completed = true;
    goals.work.actions[2].completed = true;
    goals.health.actions[0].completed = true;
    // 5 completed out of 11 = 45%
    let stats = goals.completion_stats();
    assert_eq!(stats.completed, 5, "Should have 5 completed");
    assert_eq!(stats.percentage, 45, "Should be 45% complete");
}

#[test]
fn test_minimum_and_maximum_actions() {
    let mut goals = DailyGoals::new(Local::now().date_naive());

    // Start with default 3 actions per outcome
    let stats = goals.completion_stats();
    assert_eq!(
        stats.total, 9,
        "Default should be 9 actions (3 per outcome)"
    );

    // Remove to minimum (1 per outcome)
    goals.work.remove_action(2).unwrap();
    goals.work.remove_action(1).unwrap();
    goals.health.remove_action(2).unwrap();
    goals.health.remove_action(1).unwrap();
    goals.family.remove_action(2).unwrap();
    goals.family.remove_action(1).unwrap();

    let stats = goals.completion_stats();
    assert_eq!(
        stats.total, 3,
        "Minimum should be 3 actions (1 per outcome)"
    );

    // Add to maximum (5 per outcome)
    for _ in 0..4 {
        goals.work.add_action().unwrap();
        goals.health.add_action().unwrap();
        goals.family.add_action().unwrap();
    }

    let stats = goals.completion_stats();
    assert_eq!(
        stats.total, 15,
        "Maximum should be 15 actions (5 per outcome)"
    );
}
