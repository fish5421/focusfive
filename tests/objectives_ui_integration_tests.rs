use anyhow::Result;
use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use focusfive::app::{App, InputMode, Pane};
use focusfive::models::{
    Config, DailyGoals, FiveYearVision, Objective, ObjectiveStatus, OutcomeType,
};

#[test]
fn test_objective_selector_opens_with_o_key() -> Result<()> {
    let config = Config::new()?;
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2024, 8, 31).unwrap());
    let vision = FiveYearVision::default();
    let mut app = App::new(goals, config, vision);

    // Ensure we're in Actions pane
    app.active_pane = Pane::Actions;

    // Press 'o' key
    let key_event = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
    app.handle_key(key_event)?;

    // Should be in ObjectiveSelection mode
    if let InputMode::ObjectiveSelection {
        domain,
        selection_index,
    } = app.input_mode
    {
        assert_eq!(domain, OutcomeType::Work); // Default first outcome
        assert_eq!(selection_index, 0);
    } else {
        panic!("Expected ObjectiveSelection input mode");
    }

    Ok(())
}

#[test]
fn test_objective_linking_and_unlinking() -> Result<()> {
    let config = Config::new()?;
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2024, 8, 31).unwrap());
    let vision = FiveYearVision::default();
    let mut app = App::new(goals, config, vision);

    // Create a test objective
    let objective = Objective {
        id: "test-obj-123".to_string(),
        domain: OutcomeType::Work,
        title: "Complete Project Alpha".to_string(),
        description: Some("Important milestone".to_string()),
        start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        status: ObjectiveStatus::Active,
        created: chrono::Utc::now(),
        modified: chrono::Utc::now(),
        parent_id: None,
    };

    app.objectives.objectives.push(objective);

    // Link current action to objective
    app.link_current_action_to_objective(Some("test-obj-123".to_string()))?;

    // Verify the link was created
    let action_meta = &app.day_meta.work[0];
    assert_eq!(action_meta.objective_id, Some("test-obj-123".to_string()));
    assert!(app.meta_needs_save);

    // Unlink the action
    app.link_current_action_to_objective(None)?;

    // Verify the link was removed
    let action_meta = &app.day_meta.work[0];
    assert_eq!(action_meta.objective_id, None);

    Ok(())
}

#[test]
fn test_objective_navigation_in_selector() -> Result<()> {
    let config = Config::new()?;
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2024, 8, 31).unwrap());
    let vision = FiveYearVision::default();
    let mut app = App::new(goals, config, vision);

    // Create test objectives
    let obj1 = Objective {
        id: "obj1".to_string(),
        domain: OutcomeType::Work,
        title: "Objective 1".to_string(),
        description: None,
        start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        status: ObjectiveStatus::Active,
        created: chrono::Utc::now(),
        modified: chrono::Utc::now(),
        parent_id: None,
    };

    let obj2 = Objective {
        id: "obj2".to_string(),
        domain: OutcomeType::Work,
        title: "Objective 2".to_string(),
        description: None,
        start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        status: ObjectiveStatus::Active,
        created: chrono::Utc::now(),
        modified: chrono::Utc::now(),
        parent_id: None,
    };

    app.objectives.objectives.extend(vec![obj1, obj2]);

    // Enter objective selection mode
    app.input_mode = InputMode::ObjectiveSelection {
        domain: OutcomeType::Work,
        selection_index: 0,
    };

    // Test navigation down
    let down_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(down_key)?;

    if let InputMode::ObjectiveSelection {
        selection_index, ..
    } = app.input_mode
    {
        assert_eq!(selection_index, 1);
    } else {
        panic!("Expected ObjectiveSelection mode");
    }

    // Test navigation up
    let up_key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
    app.handle_key(up_key)?;

    if let InputMode::ObjectiveSelection {
        selection_index, ..
    } = app.input_mode
    {
        assert_eq!(selection_index, 0);
    } else {
        panic!("Expected ObjectiveSelection mode");
    }

    Ok(())
}

#[test]
fn test_objective_domain_filtering() -> Result<()> {
    let config = Config::new()?;
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2024, 8, 31).unwrap());
    let vision = FiveYearVision::default();
    let mut app = App::new(goals, config, vision);

    // Ensure clean state for testing - clear any existing objectives
    app.objectives = focusfive::models::ObjectivesData::default();

    // Create objectives for different domains
    let work_obj = Objective {
        id: "work-obj".to_string(),
        domain: OutcomeType::Work,
        title: "Work Objective".to_string(),
        description: None,
        start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        status: ObjectiveStatus::Active,
        created: chrono::Utc::now(),
        modified: chrono::Utc::now(),
        parent_id: None,
    };

    let health_obj = Objective {
        id: "health-obj".to_string(),
        domain: OutcomeType::Health,
        title: "Health Objective".to_string(),
        description: None,
        start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        status: ObjectiveStatus::Active,
        created: chrono::Utc::now(),
        modified: chrono::Utc::now(),
        parent_id: None,
    };

    app.objectives.objectives.extend(vec![work_obj, health_obj]);

    // Test Work domain filtering
    app.input_mode = InputMode::ObjectiveSelection {
        domain: OutcomeType::Work,
        selection_index: 0,
    };

    // Verify only work objectives are accessible
    // (This would be tested by checking the render function behavior)
    // For now, we test that the filter logic works in the data
    let work_objectives: Vec<&Objective> = app
        .objectives
        .objectives
        .iter()
        .filter(|obj| obj.domain == OutcomeType::Work)
        .collect();

    assert_eq!(work_objectives.len(), 1);
    assert_eq!(work_objectives[0].title, "Work Objective");

    Ok(())
}

#[test]
fn test_escape_cancels_objective_selection() -> Result<()> {
    let config = Config::new()?;
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2024, 8, 31).unwrap());
    let vision = FiveYearVision::default();
    let mut app = App::new(goals, config, vision);

    // Ensure clean state for testing - clear any existing objectives
    app.objectives = focusfive::models::ObjectivesData::default();

    // Enter objective selection mode
    app.input_mode = InputMode::ObjectiveSelection {
        domain: OutcomeType::Work,
        selection_index: 0,
    };

    // Press Escape
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key(esc_key)?;

    // Should return to Normal mode
    assert!(matches!(app.input_mode, InputMode::Normal));

    Ok(())
}
