use anyhow::Result;
use chrono::NaiveDate;
use focusfive::data::{load_or_create_day_meta, save_day_meta};
use focusfive::models::{Action, Config, DailyGoals, DayMeta, Outcome, OutcomeType};
use tempfile::TempDir;

#[test]
fn test_metadata_creation_and_save() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;
    let date = NaiveDate::from_ymd_opt(2025, 8, 29).unwrap();

    // Create test goals
    let goals = DailyGoals {
        date,
        day_number: Some(5),
        work: Outcome {
            outcome_type: OutcomeType::Work,
            goal: Some("Ship feature".to_string()),
            actions: vec![
{
                    let mut action = Action::new_empty();
                    action.text = "Write code".to_string();
                    action.completed = true;
                    action
                },
                {
                    let mut action = Action::new_empty();
                    action.text = "Write tests".to_string();
                    action.completed = false;
                    action
                },
                {
                    let mut action = Action::new_empty();
                    action.text = "Deploy".to_string();
                    action.completed = false;
                    action
                },
            ],
            reflection: None,
        },
        health: Outcome {
            outcome_type: OutcomeType::Health,
            goal: Some("Stay active".to_string()),
            actions: vec![
{
                    let mut action = Action::new_empty();
                    action.text = "Morning walk".to_string();
                    action.completed = true;
                    action
                },
                {
                    let mut action = Action::new_empty();
                    action.text = "Drink water".to_string();
                    action.completed = true;
                    action
                },
                {
                    let mut action = Action::new_empty();
                    action.text = "Sleep early".to_string();
                    action.completed = false;
                    action
                },
            ],
            reflection: None,
        },
        family: Outcome {
            outcome_type: OutcomeType::Family,
            goal: Some("Be present".to_string()),
            actions: vec![
{
                    let mut action = Action::new_empty();
                    action.text = "Breakfast together".to_string();
                    action.completed = false;
                    action
                },
                {
                    let mut action = Action::new_empty();
                    action.text = "Call parents".to_string();
                    action.completed = true;
                    action
                },
                {
                    let mut action = Action::new_empty();
                    action.text = "Plan weekend".to_string();
                    action.completed = false;
                    action
                },
            ],
            reflection: None,
        },
    };

    // Create metadata from goals
    let meta = DayMeta::from_goals(&goals);

    // Verify metadata has correct structure
    assert_eq!(meta.work.len(), 3);
    assert_eq!(meta.health.len(), 3);
    assert_eq!(meta.family.len(), 3);
    assert_eq!(meta.version, 1);

    // Save metadata
    let meta_path = save_day_meta(date, &meta, &config)?;
    assert!(meta_path.exists());

    // Load metadata back
    let loaded_meta = load_or_create_day_meta(date, &goals, &config)?;
    assert_eq!(loaded_meta.work.len(), 3);
    assert_eq!(loaded_meta.health.len(), 3);
    assert_eq!(loaded_meta.family.len(), 3);

    Ok(())
}

#[test]
fn test_metadata_reconciliation() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;
    let date = NaiveDate::from_ymd_opt(2025, 8, 29).unwrap();

    // Create initial goals with 3 actions each
    let mut goals = DailyGoals {
        date,
        day_number: Some(5),
        work: Outcome {
            outcome_type: OutcomeType::Work,
            goal: None,
            actions: vec![
                Action {
                    text: "Task 1".to_string(),
                    completed: false,
                },
                Action {
                    text: "Task 2".to_string(),
                    completed: false,
                },
                Action {
                    text: "Task 3".to_string(),
                    completed: false,
                },
            ],
            reflection: None,
        },
        health: Outcome {
            outcome_type: OutcomeType::Health,
            goal: None,
            actions: vec![
                Action {
                    text: "Task 1".to_string(),
                    completed: false,
                },
                Action {
                    text: "Task 2".to_string(),
                    completed: false,
                },
                Action {
                    text: "Task 3".to_string(),
                    completed: false,
                },
            ],
            reflection: None,
        },
        family: Outcome {
            outcome_type: OutcomeType::Family,
            goal: None,
            actions: vec![
                Action {
                    text: "Task 1".to_string(),
                    completed: false,
                },
                Action {
                    text: "Task 2".to_string(),
                    completed: false,
                },
                Action {
                    text: "Task 3".to_string(),
                    completed: false,
                },
            ],
            reflection: None,
        },
    };

    // Create and save initial metadata
    let mut meta = DayMeta::from_goals(&goals);
    save_day_meta(date, &meta, &config)?;

    // Now modify goals - remove one action from work, add one to health
    goals.work.actions = vec![
        Action {
            text: "Task 1".to_string(),
            completed: false,
        },
        Action {
            text: "Task 2".to_string(),
            completed: false,
        },
        Action {
            text: "".to_string(),
            completed: false,
        }, // Empty action (simulating removal)
    ];

    // For testing, let's simulate adding by creating a new array
    // In practice this would be done differently
    let health_actions = vec![
        Action {
            text: "Task 1".to_string(),
            completed: false,
        },
        Action {
            text: "Task 2".to_string(),
            completed: false,
        },
        Action {
            text: "Task 3".to_string(),
            completed: false,
        },
        Action {
            text: "Task 4".to_string(),
            completed: false,
        }, // Added action
    ];

    // Reconcile metadata with modified goals
    meta.reconcile_with_goals(&goals);

    // Verify work metadata was adjusted (still 3 because we have empty action)
    assert_eq!(meta.work.len(), 3);

    // Manually test with different action counts
    let mut test_meta = DayMeta::from_goals(&goals);
    test_meta.work = vec![test_meta.work[0].clone()]; // Only 1 metadata entry
    test_meta.reconcile_with_goals(&goals);
    assert_eq!(test_meta.work.len(), 3); // Should expand to 3

    Ok(())
}

#[test]
fn test_metadata_persistence_across_sessions() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;
    let date = NaiveDate::from_ymd_opt(2025, 8, 29).unwrap();

    // Create goals
    let goals = DailyGoals {
        date,
        day_number: Some(1),
        work: Outcome {
            outcome_type: OutcomeType::Work,
            goal: Some("Test goal".to_string()),
            actions: vec![
                Action {
                    text: "Action 1".to_string(),
                    completed: false,
                },
                Action {
                    text: "Action 2".to_string(),
                    completed: false,
                },
                Action {
                    text: "Action 3".to_string(),
                    completed: false,
                },
            ],
            reflection: None,
        },
        health: Outcome {
            outcome_type: OutcomeType::Health,
            goal: None,
            actions: vec![
                Action {
                    text: "Action 1".to_string(),
                    completed: false,
                },
                Action {
                    text: "Action 2".to_string(),
                    completed: false,
                },
                Action {
                    text: "Action 3".to_string(),
                    completed: false,
                },
            ],
            reflection: None,
        },
        family: Outcome {
            outcome_type: OutcomeType::Family,
            goal: None,
            actions: vec![
                Action {
                    text: "Action 1".to_string(),
                    completed: false,
                },
                Action {
                    text: "Action 2".to_string(),
                    completed: false,
                },
                Action {
                    text: "Action 3".to_string(),
                    completed: false,
                },
            ],
            reflection: None,
        },
    };

    // Session 1: Create and save metadata with specific IDs
    let mut meta = DayMeta::from_goals(&goals);
    meta.work[0].estimated_min = Some(25);
    meta.work[0].tags = vec!["important".to_string()];
    let original_id = meta.work[0].id.clone();
    save_day_meta(date, &meta, &config)?;

    // Session 2: Load metadata and verify persistence
    let loaded_meta = load_or_create_day_meta(date, &goals, &config)?;

    // Verify the metadata was loaded and has the same values
    assert_eq!(loaded_meta.work[0].id, original_id);
    assert_eq!(loaded_meta.work[0].estimated_min, Some(25));
    assert_eq!(loaded_meta.work[0].tags, vec!["important".to_string()]);

    Ok(())
}

#[test]
fn test_metadata_directory_creation() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;
    let date = NaiveDate::from_ymd_opt(2025, 8, 29).unwrap();

    // Ensure meta directory doesn't exist initially
    let meta_dir = std::path::Path::new(&config.data_root).join("meta");
    assert!(!meta_dir.exists());

    // Create simple goals
    let goals = DailyGoals {
        date,
        day_number: None,
        work: Outcome {
            outcome_type: OutcomeType::Work,
            goal: None,
            actions: vec![
                Action {
                    text: "".to_string(),
                    completed: false,
                },
                Action {
                    text: "".to_string(),
                    completed: false,
                },
                Action {
                    text: "".to_string(),
                    completed: false,
                },
            ],
            reflection: None,
        },
        health: Outcome {
            outcome_type: OutcomeType::Health,
            goal: None,
            actions: vec![
                Action {
                    text: "".to_string(),
                    completed: false,
                },
                Action {
                    text: "".to_string(),
                    completed: false,
                },
                Action {
                    text: "".to_string(),
                    completed: false,
                },
            ],
            reflection: None,
        },
        family: Outcome {
            outcome_type: OutcomeType::Family,
            goal: None,
            actions: vec![
                Action {
                    text: "".to_string(),
                    completed: false,
                },
                Action {
                    text: "".to_string(),
                    completed: false,
                },
                Action {
                    text: "".to_string(),
                    completed: false,
                },
            ],
            reflection: None,
        },
    };

    // Save metadata should create the directory
    let meta = DayMeta::from_goals(&goals);
    save_day_meta(date, &meta, &config)?;

    // Verify directory was created
    assert!(meta_dir.exists());
    assert!(meta_dir.is_dir());

    // Verify metadata file exists
    let meta_file = meta_dir.join("2025-08-29.meta.json");
    assert!(meta_file.exists());

    Ok(())
}
