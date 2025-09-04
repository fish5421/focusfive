use anyhow::Result;
use chrono::NaiveDate;
use focusfive::app::App;
use focusfive::data::*;
use focusfive::models::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_controller_saves_all_data_types() -> Result<()> {
    // Setup test environment
    let temp_dir = TempDir::new()?;
    let config = Config {
        goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
        data_root: temp_dir.path().join("data").to_string_lossy().to_string(),
    };

    // Create directories
    fs::create_dir_all(&config.goals_dir)?;
    fs::create_dir_all(&config.data_root)?;

    // Create app with test data
    let date = NaiveDate::from_ymd_opt(2025, 8, 29).unwrap();
    let goals = DailyGoals::new(date);
    let vision = FiveYearVision::new();
    let mut app = App::new(goals, config.clone(), vision);

    // Modify data and set save flags
    app.goals.work.actions[0].text = "Test work action".to_string();
    app.needs_save = true;

    app.day_meta.work[0].status = ActionMetaStatus::InProgress;
    app.meta_needs_save = true;

    app.vision.work = "Test work vision".to_string();
    app.vision_needs_save = true;

    app.templates.add_template(
        "Test Template".to_string(),
        vec![
            "Action 1".to_string(),
            "Action 2".to_string(),
            "Action 3".to_string(),
        ],
    );
    app.templates_needs_save = true;

    // Add a test objective
    let objective = Objective {
        id: "test-obj-1".to_string(),
        domain: OutcomeType::Work,
        title: "Test Objective".to_string(),
        description: Some("Test description".to_string()),
        start: date,
        end: None,
        status: ObjectiveStatus::Active,
        created: chrono::Utc::now(),
        modified: chrono::Utc::now(),
        parent_id: None,
    };
    app.objectives.objectives.push(objective);
    app.objectives_needs_save = true;

    // Add a test indicator
    let indicator = IndicatorDef {
        id: "test-ind-1".to_string(),
        name: "Test Indicator".to_string(),
        kind: IndicatorKind::Leading,
        unit: IndicatorUnit::Count,
        objective_id: Some("test-obj-1".to_string()),
        target: Some(100.0),
        direction: IndicatorDirection::HigherIsBetter,
        active: true,
        created: chrono::Utc::now(),
        modified: chrono::Utc::now(),
        lineage_of: None,
        notes: Some("Test notes".to_string()),
    };
    app.indicators.indicators.push(indicator);
    app.indicators_needs_save = true;

    // Simulate saves (what the controller loop would do)
    if app.needs_save {
        write_goals_file(&app.goals, &config)?;
        app.day_meta.reconcile_with_goals(&app.goals);
        app.needs_save = false;
        app.meta_needs_save = true;
    }

    if app.meta_needs_save {
        save_day_meta(app.goals.date, &app.day_meta, &config)?;
        app.meta_needs_save = false;
    }

    if app.vision_needs_save {
        save_vision(&app.vision, &config)?;
        app.vision_needs_save = false;
    }

    if app.templates_needs_save {
        save_templates(&app.templates, &config)?;
        app.templates_needs_save = false;
    }

    if app.objectives_needs_save {
        save_objectives(&app.objectives, &config)?;
        app.objectives_needs_save = false;
    }

    if app.indicators_needs_save {
        save_indicators(&app.indicators, &config)?;
        app.indicators_needs_save = false;
    }

    // Verify all files were created
    let goals_file = temp_dir.path().join("goals").join("2025-08-29.md");
    assert!(goals_file.exists(), "Goals file should exist");

    let meta_file = temp_dir
        .path()
        .join("data")
        .join("meta")
        .join("2025-08-29.meta.json");
    assert!(meta_file.exists(), "Meta file should exist");

    let vision_file = temp_dir.path().join("vision.json");
    assert!(vision_file.exists(), "Vision file should exist");

    let templates_file = temp_dir.path().join("templates.json");
    assert!(templates_file.exists(), "Templates file should exist");

    let objectives_file = temp_dir.path().join("data").join("objectives.json");
    assert!(objectives_file.exists(), "Objectives file should exist");

    let indicators_file = temp_dir.path().join("data").join("indicators.json");
    assert!(indicators_file.exists(), "Indicators file should exist");

    // Load data back and verify
    let loaded_goals = read_goals_file(&goals_file)?;
    assert_eq!(loaded_goals.work.actions[0].text, "Test work action");

    let loaded_meta = load_or_create_day_meta(date, &loaded_goals, &config)?;
    assert_eq!(loaded_meta.work[0].status, ActionMetaStatus::InProgress);

    let loaded_vision = load_or_create_vision(&config)?;
    assert_eq!(loaded_vision.work, "Test work vision");

    let loaded_templates = load_or_create_templates(&config)?;
    assert!(loaded_templates.templates.contains_key("Test Template"));

    let loaded_objectives = load_or_create_objectives(&config)?;
    assert_eq!(loaded_objectives.objectives.len(), 1);
    assert_eq!(loaded_objectives.objectives[0].title, "Test Objective");

    let loaded_indicators = load_or_create_indicators(&config)?;
    assert_eq!(loaded_indicators.indicators.len(), 1);
    assert_eq!(loaded_indicators.indicators[0].name, "Test Indicator");

    // Verify all save flags are reset
    assert!(!app.needs_save);
    assert!(!app.meta_needs_save);
    assert!(!app.vision_needs_save);
    assert!(!app.templates_needs_save);
    assert!(!app.objectives_needs_save);
    assert!(!app.indicators_needs_save);

    Ok(())
}

#[test]
fn test_atomic_write_prevents_corruption() -> Result<()> {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::thread;

    let temp_dir = TempDir::new()?;
    let config = Arc::new(Config {
        goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
        data_root: temp_dir.path().join("data").to_string_lossy().to_string(),
    });

    fs::create_dir_all(&config.data_root)?;

    let success_count = Arc::new(AtomicU32::new(0));
    let error_count = Arc::new(AtomicU32::new(0));

    // Spawn multiple threads trying to save simultaneously
    let mut handles = vec![];
    for i in 0..10 {
        let config = Arc::clone(&config);
        let success_count = Arc::clone(&success_count);
        let error_count = Arc::clone(&error_count);

        let handle = thread::spawn(move || {
            let mut objectives = ObjectivesData::default();
            objectives.version = i;

            // Add unique objective to detect if writes get mixed
            objectives.objectives.push(Objective {
                id: format!("thread-{}-obj", i),
                domain: OutcomeType::Work,
                title: format!("Thread {} Objective", i),
                description: None,
                start: NaiveDate::from_ymd_opt(2025, 8, 29).unwrap(),
                end: None,
                status: ObjectiveStatus::Active,
                created: chrono::Utc::now(),
                modified: chrono::Utc::now(),
                parent_id: None,
            });

            // Try to save
            match save_objectives(&objectives, &config) {
                Ok(_) => {
                    success_count.fetch_add(1, Ordering::SeqCst);
                }
                Err(_) => {
                    error_count.fetch_add(1, Ordering::SeqCst);
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // At least some should succeed (last write wins)
    let successes = success_count.load(Ordering::SeqCst);
    let errors = error_count.load(Ordering::SeqCst);

    println!(
        "Concurrent saves - Success: {}, Errors: {}",
        successes, errors
    );
    assert!(successes > 0, "At least some saves should succeed");

    // Verify the final file is valid and not corrupted
    let loaded = load_or_create_objectives(&config)?;
    assert_eq!(
        loaded.objectives.len(),
        1,
        "Should have exactly one objective"
    );
    assert!(
        loaded.objectives[0].id.starts_with("thread-"),
        "Should be from one of the threads"
    );

    Ok(())
}
