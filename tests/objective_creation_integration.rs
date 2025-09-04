use anyhow::Result;
use chrono::NaiveDate;
use focusfive::models::{Config, DailyGoals, OutcomeType, FiveYearVision};
use focusfive::app::{App, InputMode, Pane};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
fn test_objective_creation_workflow() -> Result<()> {
    let config = Config::new()?;
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2024, 8, 31).unwrap());
    let vision = FiveYearVision::default();
    let mut app = App::new(goals, config, vision);
    
    // Ensure clean state for testing - clear any existing objectives
    app.objectives = focusfive::models::ObjectivesData::default();
    
    // Ensure we're in Actions pane
    app.active_pane = Pane::Actions;
    
    // Press 'o' key to open objective selector
    let key_event = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
    app.handle_key(key_event)?;
    
    // Should be in ObjectiveSelection mode
    if let InputMode::ObjectiveSelection { domain, selection_index } = app.input_mode {
        assert_eq!(domain, OutcomeType::Work);
        assert_eq!(selection_index, 0); // Should select "Create New Objective" since no objectives exist
    } else {
        panic!("Expected ObjectiveSelection input mode");
    }
    
    // Press Enter to select "Create New Objective"
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(enter_key)?;
    
    // Should now be in ObjectiveCreation mode
    if let InputMode::ObjectiveCreation { domain, ref buffer } = app.input_mode {
        assert_eq!(domain, OutcomeType::Work);
        assert_eq!(buffer, "");
    } else {
        panic!("Expected ObjectiveCreation input mode, got: {:?}", app.input_mode);
    }
    
    // Type some text for the objective title
    let objective_title = "Launch new product";
    for ch in objective_title.chars() {
        let char_key = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        app.handle_key(char_key)?;
    }
    
    // Verify buffer contains our text
    if let InputMode::ObjectiveCreation { domain, ref buffer } = app.input_mode {
        assert_eq!(domain, OutcomeType::Work);
        assert_eq!(buffer, objective_title);
    } else {
        panic!("Expected ObjectiveCreation input mode");
    }
    
    // Press Enter to create the objective
    let create_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(create_key)?;
    
    // Should return to Normal mode
    assert!(matches!(app.input_mode, InputMode::Normal));
    
    // Verify objective was created
    assert_eq!(app.objectives.objectives.len(), 1);
    let created_objective = &app.objectives.objectives[0];
    assert_eq!(created_objective.title, objective_title);
    assert_eq!(created_objective.domain, OutcomeType::Work);
    
    // Verify current action is linked to the objective
    let action_meta = &app.day_meta.work[0];
    assert_eq!(action_meta.objective_id, Some(created_objective.id.clone()));
    
    Ok(())
}

#[test]
fn test_objective_creation_escape_cancels() -> Result<()> {
    let config = Config::new()?;
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2024, 8, 31).unwrap());
    let vision = FiveYearVision::default();
    let mut app = App::new(goals, config, vision);
    
    // Ensure clean state for testing - clear any existing objectives
    app.objectives = focusfive::models::ObjectivesData::default();
    
    // Enter objective creation mode
    app.input_mode = InputMode::ObjectiveCreation {
        domain: OutcomeType::Work,
        buffer: "Some partial text".to_string(),
    };
    
    // Press Escape
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key(esc_key)?;
    
    // Should return to Normal mode
    assert!(matches!(app.input_mode, InputMode::Normal));
    
    // No objective should have been created
    assert_eq!(app.objectives.objectives.len(), 0);
    
    Ok(())
}

#[test]
fn test_objective_creation_empty_title_returns_to_normal() -> Result<()> {
    let config = Config::new()?;
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2024, 8, 31).unwrap());
    let vision = FiveYearVision::default();
    let mut app = App::new(goals, config, vision);
    
    // Ensure clean state for testing - clear any existing objectives
    app.objectives = focusfive::models::ObjectivesData::default();
    
    // Enter objective creation mode with empty buffer
    app.input_mode = InputMode::ObjectiveCreation {
        domain: OutcomeType::Health,
        buffer: String::new(),
    };
    
    // Press Enter with empty buffer
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(enter_key)?;
    
    // Should return to Normal mode
    assert!(matches!(app.input_mode, InputMode::Normal));
    
    // No objective should have been created
    assert_eq!(app.objectives.objectives.len(), 0);
    
    Ok(())
}

#[test]
fn test_objective_creation_whitespace_only_title_returns_to_normal() -> Result<()> {
    let config = Config::new()?;
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2024, 8, 31).unwrap());
    let vision = FiveYearVision::default();
    let mut app = App::new(goals, config, vision);
    
    // Ensure clean state for testing - clear any existing objectives
    app.objectives = focusfive::models::ObjectivesData::default();
    
    // Enter objective creation mode with whitespace-only buffer
    app.input_mode = InputMode::ObjectiveCreation {
        domain: OutcomeType::Family,
        buffer: "   \t\n  ".to_string(),
    };
    
    // Press Enter with whitespace-only buffer
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(enter_key)?;
    
    // Should return to Normal mode
    assert!(matches!(app.input_mode, InputMode::Normal));
    
    // No objective should have been created
    assert_eq!(app.objectives.objectives.len(), 0);
    
    Ok(())
}