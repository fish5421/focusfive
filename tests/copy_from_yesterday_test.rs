use chrono::Local;
use focusfive::data::{get_yesterday_goals, write_goals_file};
use focusfive::models::{Action, Config, DailyGoals};
use std::fs;
use tempfile::TempDir;

fn create_test_config() -> (Config, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let config = Config {
        goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
    };
    fs::create_dir_all(&config.goals_dir).unwrap();
    (config, temp_dir)
}

#[test]
fn test_get_yesterday_goals_exists() {
    let (config, _temp_dir) = create_test_config();

    // Create yesterday's goals
    let yesterday = Local::now().date_naive().pred_opt().unwrap();
    let mut yesterday_goals = DailyGoals::new(yesterday);
    yesterday_goals.work.actions[0] = Action {
        text: "Complete report".to_string(),
        completed: true,
    };
    yesterday_goals.work.actions[1] = Action {
        text: "Review code".to_string(),
        completed: false,
    };
    yesterday_goals.health.actions[0] = Action {
        text: "Morning run".to_string(),
        completed: false,
    };

    // Save yesterday's goals
    write_goals_file(&yesterday_goals, &config).unwrap();

    // Test getting yesterday's goals
    let today = Local::now().date_naive();
    let result = get_yesterday_goals(today, &config).unwrap();

    assert!(result.is_some());
    let loaded_goals = result.unwrap();
    assert_eq!(loaded_goals.date, yesterday);
    assert_eq!(loaded_goals.work.actions[0].text, "Complete report");
    assert!(loaded_goals.work.actions[0].completed);
    assert_eq!(loaded_goals.work.actions[1].text, "Review code");
    assert!(!loaded_goals.work.actions[1].completed);
}

#[test]
fn test_get_yesterday_goals_not_exists() {
    let (config, _temp_dir) = create_test_config();

    // Don't create any goals file
    let today = Local::now().date_naive();
    let result = get_yesterday_goals(today, &config).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_copy_uncompleted_actions() {
    let (config, _temp_dir) = create_test_config();

    // Create yesterday's goals with mixed completed/uncompleted
    let yesterday = Local::now().date_naive().pred_opt().unwrap();
    let mut yesterday_goals = DailyGoals::new(yesterday);

    // Work actions
    yesterday_goals.work.actions[0] = Action {
        text: "Task 1 - completed".to_string(),
        completed: true,
    };
    yesterday_goals.work.actions[1] = Action {
        text: "Task 2 - not completed".to_string(),
        completed: false,
    };
    yesterday_goals.work.actions[2] = Action {
        text: "Task 3 - not completed".to_string(),
        completed: false,
    };

    // Health actions
    yesterday_goals.health.actions[0] = Action {
        text: "Exercise".to_string(),
        completed: false,
    };

    // Family actions
    yesterday_goals.family.actions[1] = Action {
        text: "Call parents".to_string(),
        completed: false,
    };

    write_goals_file(&yesterday_goals, &config).unwrap();

    // Load yesterday's goals
    let today = Local::now().date_naive();
    let loaded = get_yesterday_goals(today, &config).unwrap().unwrap();

    // Check that we can identify uncompleted actions
    let mut uncompleted_count = 0;
    for outcome in loaded.outcomes() {
        for action in &outcome.actions {
            if !action.text.is_empty() && !action.completed {
                uncompleted_count += 1;
            }
        }
    }

    assert_eq!(uncompleted_count, 4); // Task 2, Task 3, Exercise, Call parents
}

#[test]
fn test_copy_selections_array() {
    // Test the selections array logic used in the UI
    let mut selections = [false; 9];

    // Simulate pre-selecting uncompleted actions
    let yesterday = Local::now().date_naive().pred_opt().unwrap();
    let mut yesterday_goals = DailyGoals::new(yesterday);

    yesterday_goals.work.actions[0] = Action {
        text: "Completed task".to_string(),
        completed: true,
    };
    yesterday_goals.work.actions[1] = Action {
        text: "Uncompleted task".to_string(),
        completed: false,
    };

    // Pre-select logic
    let mut index = 0;
    for outcome in yesterday_goals.outcomes() {
        for action in &outcome.actions {
            if index < 9 && !action.text.is_empty() && !action.completed {
                selections[index] = true;
            }
            index += 1;
        }
    }

    // Check that only the uncompleted task is selected
    assert!(!selections[0]); // Completed task - not selected
    assert!(selections[1]); // Uncompleted task - selected
    assert!(!selections[2]); // Empty - not selected
}

#[test]
fn test_copy_to_empty_slots_only() {
    let (config, _temp_dir) = create_test_config();

    // Create yesterday's goals
    let yesterday = Local::now().date_naive().pred_opt().unwrap();
    let mut yesterday_goals = DailyGoals::new(yesterday);
    yesterday_goals.work.actions[0] = Action {
        text: "Yesterday task 1".to_string(),
        completed: false,
    };
    yesterday_goals.work.actions[1] = Action {
        text: "Yesterday task 2".to_string(),
        completed: false,
    };

    // Create today's goals with some existing content
    let today = Local::now().date_naive();
    let mut today_goals = DailyGoals::new(today);
    today_goals.work.actions[0] = Action {
        text: "Today's existing task".to_string(),
        completed: false,
    };

    // Simulate copying logic
    let selections = [true, true, false, false, false, false, false, false, false];
    let mut action_index = 0;

    for (outcome_idx, outcome) in yesterday_goals.outcomes().iter().enumerate() {
        for (action_idx, action) in outcome.actions.iter().enumerate() {
            if action_index < 9 && selections[action_index] && !action.text.is_empty() {
                match outcome_idx {
                    0 => {
                        // Only copy if slot is empty
                        if today_goals.work.actions[action_idx].text.is_empty() {
                            today_goals.work.actions[action_idx].text = action.text.clone();
                        }
                    }
                    _ => {}
                }
            }
            action_index += 1;
        }
    }

    // Check that existing content was preserved
    assert_eq!(today_goals.work.actions[0].text, "Today's existing task");
    // Second slot should have been filled
    assert_eq!(today_goals.work.actions[1].text, "Yesterday task 2");
}

#[test]
fn test_edge_case_no_actions_yesterday() {
    let (config, _temp_dir) = create_test_config();

    // Create yesterday's goals with no actions
    let yesterday = Local::now().date_naive().pred_opt().unwrap();
    let yesterday_goals = DailyGoals::new(yesterday);
    write_goals_file(&yesterday_goals, &config).unwrap();

    // Should still load successfully
    let today = Local::now().date_naive();
    let result = get_yesterday_goals(today, &config).unwrap();

    assert!(result.is_some());
    let loaded = result.unwrap();

    // All actions should be empty
    for outcome in loaded.outcomes() {
        for action in &outcome.actions {
            assert!(action.text.is_empty());
        }
    }
}
