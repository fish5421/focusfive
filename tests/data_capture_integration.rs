use anyhow::Result;
use chrono::{NaiveDate, Utc};
use focusfive::data::{generate_markdown, read_goals_file};
use focusfive::data_capture::{
    ActionStatus, DataStorage, Indicator, MeasurementFrequency, 
    Objective, Observation, Review, ReviewPeriod
};
use focusfive::models::{Action, Config, DailyGoals, Outcome, OutcomeType};
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn test_backward_compatibility_with_existing_markdown() -> Result<()> {
    let temp = TempDir::new()?;
    let goals_dir = temp.path().join("goals");
    fs::create_dir_all(&goals_dir)?;
    
    // Create a DailyGoals structure (existing functionality)
    let date = NaiveDate::from_ymd_opt(2025, 8, 28).unwrap();
    let goals = DailyGoals {
        date,
        day_number: Some(7),
        work: Outcome {
            outcome_type: OutcomeType::Work,
            goal: Some("Ship feature X".to_string()),
            actions: vec![
                Action { text: "Complete PR review".to_string(), completed: true },
                Action { text: "Write documentation".to_string(), completed: false },
                Action { text: "Deploy to staging".to_string(), completed: false },
            ],
            reflection: None,
        },
        health: Outcome {
            outcome_type: OutcomeType::Health,
            goal: Some("Stay active".to_string()),
            actions: vec![
                Action { text: "Morning walk".to_string(), completed: true },
                Action { text: "Drink 8 glasses water".to_string(), completed: false },
                Action { text: "Sleep before 11pm".to_string(), completed: false },
            ],
            reflection: None,
        },
        family: Outcome {
            outcome_type: OutcomeType::Family,
            goal: Some("Be present".to_string()),
            actions: vec![
                Action { text: "Breakfast together".to_string(), completed: false },
                Action { text: "Call parents".to_string(), completed: true },
                Action { text: "Plan weekend activity".to_string(), completed: false },
            ],
            reflection: None,
        },
    };
    
    // Save using existing markdown system
    let file_path = goals_dir.join(format!("{}.md", date.format("%Y-%m-%d")));
    let markdown = generate_markdown(&goals);
    let mut file = fs::File::create(&file_path)?;
    file.write_all(markdown.as_bytes())?;
    
    // Load it back using existing system
    let loaded_goals = read_goals_file(&file_path)?;
    assert_eq!(loaded_goals.date, date);
    assert_eq!(loaded_goals.day_number, Some(7));
    assert_eq!(loaded_goals.work.goal, Some("Ship feature X".to_string()));
    
    // Now create the new data capture storage
    std::env::set_var("HOME", temp.path());
    let config = Config::new()?;
    let storage = DataStorage::new(&config)?;
    
    // Create metadata from the existing goals
    let metadata = storage.create_day_metadata_from_goals(&loaded_goals);
    
    // Verify metadata was created correctly
    assert_eq!(metadata.date, date);
    assert_eq!(metadata.day_number, Some(7));
    assert_eq!(metadata.actions.len(), 9); // 3 outcomes * 3 actions each
    
    // Check that action metadata preserves the original data
    let work_actions: Vec<_> = metadata.actions.iter()
        .filter(|a| a.outcome_type == OutcomeType::Work)
        .collect();
    assert_eq!(work_actions.len(), 3);
    assert_eq!(work_actions[0].text, "Complete PR review");
    assert_eq!(work_actions[0].completed, true);
    assert_eq!(work_actions[0].status, ActionStatus::Completed);
    
    // Save the metadata
    storage.save_day_metadata(&metadata)?;
    
    // Load it back
    let loaded_metadata = storage.load_day_metadata(date)?;
    assert!(loaded_metadata.is_some());
    let loaded_metadata = loaded_metadata.unwrap();
    assert_eq!(loaded_metadata.date, date);
    assert_eq!(loaded_metadata.actions.len(), 9);
    
    Ok(())
}

#[test]
fn test_objectives_storage() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());
    let config = Config::new()?;
    let storage = DataStorage::new(&config)?;
    
    let objective = Objective {
        id: Uuid::new_v4(),
        title: "Complete Q3 roadmap".to_string(),
        description: Some("Deliver all planned features for Q3".to_string()),
        outcome_type: OutcomeType::Work,
        target_date: Some(NaiveDate::from_ymd_opt(2025, 9, 30).unwrap()),
        key_results: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        archived: false,
    };
    
    storage.save_objectives(&[objective.clone()])?;
    
    let loaded = storage.load_objectives()?;
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].title, "Complete Q3 roadmap");
    assert_eq!(loaded[0].outcome_type, OutcomeType::Work);
    
    Ok(())
}

#[test]
fn test_indicators_and_observations() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());
    let config = Config::new()?;
    let storage = DataStorage::new(&config)?;
    
    let indicator = Indicator {
        id: Uuid::new_v4(),
        name: "Daily steps".to_string(),
        description: Some("Track daily step count".to_string()),
        outcome_type: OutcomeType::Health,
        unit: Some("steps".to_string()),
        target_value: Some(10000.0),
        frequency: MeasurementFrequency::Daily,
        created_at: Utc::now(),
        archived: false,
    };
    
    storage.save_indicators(&[indicator.clone()])?;
    
    // Add observations
    let obs1 = Observation {
        id: Uuid::new_v4(),
        indicator_id: indicator.id,
        value: 8500.0,
        notes: Some("Rainy day".to_string()),
        observed_at: Utc::now(),
        created_at: Utc::now(),
    };
    
    let obs2 = Observation {
        id: Uuid::new_v4(),
        indicator_id: indicator.id,
        value: 12000.0,
        notes: Some("Great weather for walking".to_string()),
        observed_at: Utc::now(),
        created_at: Utc::now(),
    };
    
    storage.append_observation(&obs1)?;
    storage.append_observation(&obs2)?;
    
    let observations = storage.load_observations(Some(indicator.id))?;
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0].value, 8500.0);
    assert_eq!(observations[1].value, 12000.0);
    
    Ok(())
}

#[test]
fn test_weekly_review() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());
    let config = Config::new()?;
    let storage = DataStorage::new(&config)?;
    
    let mut completion_stats = std::collections::HashMap::new();
    completion_stats.insert(
        OutcomeType::Work,
        focusfive::data_capture::CompletionSummary {
            total_actions: 21,
            completed_actions: 18,
            completion_rate: 85.7,
        },
    );
    
    let review = Review {
        version: 1,
        id: Uuid::new_v4(),
        period_type: ReviewPeriod::Weekly,
        period_identifier: "2025-W35".to_string(),
        start_date: NaiveDate::from_ymd_opt(2025, 8, 25).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2025, 8, 31).unwrap(),
        wins: vec!["Completed feature X".to_string()],
        challenges: vec!["Time management".to_string()],
        learnings: vec!["Need better planning".to_string()],
        next_actions: vec!["Improve estimation".to_string()],
        completion_stats,
        created_at: Utc::now(),
    };
    
    storage.save_review(&review)?;
    
    // Verify the review file was created
    let review_path = storage.reviews_dir.join("2025-W35.json");
    assert!(review_path.exists());
    
    Ok(())
}

#[test]
fn test_macos_application_support_directory() -> Result<()> {
    // This test verifies the directory structure preference on macOS
    let temp = TempDir::new()?;
    
    // Simulate macOS environment
    #[cfg(target_os = "macos")]
    {
        let app_support = temp.path()
            .join("Library")
            .join("Application Support")
            .join("FocusFive");
        fs::create_dir_all(&app_support)?;
        
        std::env::set_var("HOME", temp.path());
        let config = Config::new()?;
    let storage = DataStorage::new(&config)?;
        
        // On macOS, should use Application Support
        assert!(storage.data_root.to_string_lossy().contains("Application Support"));
        assert!(storage.goals_dir.exists());
        assert!(storage.meta_dir.exists());
        assert!(storage.reviews_dir.exists());
    }
    
    // On non-macOS, should fall back to ~/FocusFive
    #[cfg(not(target_os = "macos"))]
    {
        std::env::set_var("HOME", temp.path());
        let config = Config::new()?;
    let storage = DataStorage::new(&config)?;
        
        assert!(storage.data_root.to_string_lossy().contains("FocusFive"));
        assert!(storage.goals_dir.exists());
    }
    
    Ok(())
}