use anyhow::Result;
use chrono::NaiveDate;
use focusfive::data::{
    load_or_create_day_meta, load_or_create_objectives, save_day_meta, save_objectives,
    write_goals_file,
};
use focusfive::models::{
    Action, Config, DailyGoals, Objective, ObjectivesData, Outcome, OutcomeType,
};
use tempfile::TempDir;

#[test]
fn test_objective_action_linking() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;
    let date = NaiveDate::from_ymd_opt(2025, 8, 29).unwrap();

    // Step 1: Create and save an objective
    let objective = Objective::new(OutcomeType::Work, "Ship new feature".to_string());
    let objective_id = objective.id.clone();

    let objectives = ObjectivesData {
        version: 1,
        objectives: vec![objective],
    };

    save_objectives(&objectives, &config)?;

    // Step 2: Create goals with actions
    let goals = DailyGoals {
        date,
        day_number: Some(5),
        work: Outcome {
            outcome_type: OutcomeType::Work,
            goal: Some("Make progress on feature".to_string()),
            actions: vec![
                Action {
                    text: "Write unit tests".to_string(),
                    completed: false,
                },
                Action {
                    text: "Implement core logic".to_string(),
                    completed: false,
                },
                Action {
                    text: "Code review".to_string(),
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
                    text: "Morning walk".to_string(),
                    completed: true,
                },
                Action {
                    text: "Drink water".to_string(),
                    completed: false,
                },
                Action {
                    text: "Sleep early".to_string(),
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
                    text: "Breakfast together".to_string(),
                    completed: true,
                },
                Action {
                    text: "Call parents".to_string(),
                    completed: false,
                },
                Action {
                    text: "Game night".to_string(),
                    completed: false,
                },
            ],
            reflection: None,
        },
    };

    write_goals_file(&goals, &config)?;

    // Step 3: Create metadata and link first work action to objective
    let mut day_meta = load_or_create_day_meta(date, &goals, &config)?;

    // Link first work action to our objective
    day_meta.work[0].objective_id = Some(objective_id.clone());

    // Save the metadata
    save_day_meta(date, &day_meta, &config)?;

    // Step 4: Reload both objectives and metadata to verify linking
    let loaded_objectives = load_or_create_objectives(&config)?;
    let loaded_meta = load_or_create_day_meta(date, &goals, &config)?;

    // Verify objective still exists
    assert_eq!(loaded_objectives.objectives.len(), 1);
    assert_eq!(loaded_objectives.objectives[0].id, objective_id);
    assert_eq!(loaded_objectives.objectives[0].title, "Ship new feature");

    // Verify metadata preserved the link
    assert_eq!(loaded_meta.work[0].objective_id, Some(objective_id.clone()));
    assert!(loaded_meta.work[1].objective_id.is_none());
    assert!(loaded_meta.work[2].objective_id.is_none());

    // Verify other outcomes have no links
    for action_meta in &loaded_meta.health {
        assert!(action_meta.objective_id.is_none());
    }
    for action_meta in &loaded_meta.family {
        assert!(action_meta.objective_id.is_none());
    }

    Ok(())
}

#[test]
fn test_multiple_objectives_different_domains() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;
    let date = NaiveDate::from_ymd_opt(2025, 8, 30).unwrap();

    // Create objectives for different domains
    let work_obj = Objective::new(OutcomeType::Work, "Q3 deliverables".to_string());
    let health_obj = Objective::new(OutcomeType::Health, "Marathon training".to_string());
    let family_obj = Objective::new(OutcomeType::Family, "Summer vacation".to_string());

    let work_id = work_obj.id.clone();
    let health_id = health_obj.id.clone();
    let family_id = family_obj.id.clone();

    let objectives = ObjectivesData {
        version: 1,
        objectives: vec![work_obj, health_obj, family_obj],
    };

    save_objectives(&objectives, &config)?;

    // Create goals
    let goals = DailyGoals {
        date,
        day_number: Some(10),
        work: Outcome {
            outcome_type: OutcomeType::Work,
            goal: None,
            actions: vec![
                Action {
                    text: "Sprint planning".to_string(),
                    completed: false,
                },
                Action {
                    text: "Code review".to_string(),
                    completed: false,
                },
                Action {
                    text: "Documentation".to_string(),
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
                    text: "5K run".to_string(),
                    completed: false,
                },
                Action {
                    text: "Stretching".to_string(),
                    completed: false,
                },
                Action {
                    text: "Meal prep".to_string(),
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
                    text: "Research destinations".to_string(),
                    completed: false,
                },
                Action {
                    text: "Book flights".to_string(),
                    completed: false,
                },
                Action {
                    text: "Plan activities".to_string(),
                    completed: false,
                },
            ],
            reflection: None,
        },
    };

    write_goals_file(&goals, &config)?;

    // Link actions to objectives across domains
    let mut day_meta = load_or_create_day_meta(date, &goals, &config)?;

    day_meta.work[0].objective_id = Some(work_id.clone());
    day_meta.work[1].objective_id = Some(work_id.clone());
    day_meta.health[0].objective_id = Some(health_id.clone());
    day_meta.family[0].objective_id = Some(family_id.clone());
    day_meta.family[1].objective_id = Some(family_id.clone());

    save_day_meta(date, &day_meta, &config)?;

    // Reload and verify
    let loaded_meta = load_or_create_day_meta(date, &goals, &config)?;

    assert_eq!(loaded_meta.work[0].objective_id, Some(work_id.clone()));
    assert_eq!(loaded_meta.work[1].objective_id, Some(work_id));
    assert!(loaded_meta.work[2].objective_id.is_none());

    assert_eq!(loaded_meta.health[0].objective_id, Some(health_id));
    assert!(loaded_meta.health[1].objective_id.is_none());

    assert_eq!(loaded_meta.family[0].objective_id, Some(family_id.clone()));
    assert_eq!(loaded_meta.family[1].objective_id, Some(family_id));
    assert!(loaded_meta.family[2].objective_id.is_none());

    Ok(())
}

#[test]
fn test_objective_deletion_preserves_metadata() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;
    let date = NaiveDate::from_ymd_opt(2025, 8, 31).unwrap();

    // Create objective
    let objective = Objective::new(OutcomeType::Work, "Temporary objective".to_string());
    let objective_id = objective.id.clone();

    let mut objectives = ObjectivesData {
        version: 1,
        objectives: vec![objective],
    };

    save_objectives(&objectives, &config)?;

    // Create goals and metadata with link
    let goals = DailyGoals {
        date,
        day_number: Some(15),
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

    write_goals_file(&goals, &config)?;

    let mut day_meta = load_or_create_day_meta(date, &goals, &config)?;
    day_meta.work[0].objective_id = Some(objective_id.clone());
    save_day_meta(date, &day_meta, &config)?;

    // Now delete the objective
    objectives.objectives.clear();
    save_objectives(&objectives, &config)?;

    // Reload metadata - the objective_id should still be there
    // (orphaned reference, but metadata preserved)
    let loaded_meta = load_or_create_day_meta(date, &goals, &config)?;
    assert_eq!(loaded_meta.work[0].objective_id, Some(objective_id));

    // Verify objectives are indeed empty
    let loaded_objectives = load_or_create_objectives(&config)?;
    assert_eq!(loaded_objectives.objectives.len(), 0);

    Ok(())
}
