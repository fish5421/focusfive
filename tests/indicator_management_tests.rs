use anyhow::Result;
use chrono::Utc;
use chrono::NaiveDate;
use focusfive::models::{
    Action, Config, DailyGoals, FiveYearVision, IndicatorDef, IndicatorDirection, IndicatorKind,
    IndicatorUnit, IndicatorsData, Outcome, OutcomeType,
};
use focusfive::app::{App, InputMode};
use std::fs;
use tempfile::TempDir;

fn create_test_app() -> Result<(App, TempDir)> {
    let temp_dir = TempDir::new()?;
    let goals_dir = temp_dir.path().join("FocusFive/goals");
    fs::create_dir_all(&goals_dir)?;

    // Create config
    let config = Config {
        goals_dir: goals_dir.to_str().unwrap().to_string(),
        data_root: temp_dir.path().to_str().unwrap().to_string(),
    };

    // Create empty goals
    let goals = DailyGoals {
        date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        day_number: Some(1),
        work: Outcome {
            outcome_type: OutcomeType::Work,
            goal: None,
            actions: vec![
                Action::new_empty(),
                Action::new_empty(),
                Action::new_empty(),
            ],
            reflection: None,
        },
        health: Outcome {
            outcome_type: OutcomeType::Health,
            goal: None,
            actions: vec![
                Action::new_empty(),
                Action::new_empty(),
                Action::new_empty(),
            ],
            reflection: None,
        },
        family: Outcome {
            outcome_type: OutcomeType::Family,
            goal: None,
            actions: vec![
                Action::new_empty(),
                Action::new_empty(),
                Action::new_empty(),
            ],
            reflection: None,
        },
    };

    // Create empty vision
    let vision = FiveYearVision {
        work: "Test work vision".to_string(),
        health: "Test health vision".to_string(),
        family: "Test family vision".to_string(),
        created: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        modified: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
    };

    let app = App::new(goals, config, vision);
    Ok((app, temp_dir))
}

fn create_test_indicator(objective_id: &str, name: &str) -> IndicatorDef {
    IndicatorDef {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        kind: IndicatorKind::Leading,
        unit: IndicatorUnit::Count,
        objective_id: Some(objective_id.to_string()),
        target: Some(10.0),
        direction: IndicatorDirection::HigherIsBetter,
        active: true,
        created: Utc::now(),
        modified: Utc::now(),
        lineage_of: None,
        notes: Some("Test indicator".to_string()),
    }
}

#[test]
fn test_indicator_management_mode_initialization() -> Result<()> {
    let (mut app, _temp_dir) = create_test_app()?;
    
    // Create a test objective
    let objective_id = "test-obj-123";
    let objective_title = "Test Objective";
    
    // Set up indicator management mode
    app.input_mode = InputMode::IndicatorManagement {
        objective_id: objective_id.to_string(),
        objective_title: objective_title.to_string(),
        indicators: vec![
            create_test_indicator(objective_id, "Indicator 1"),
            create_test_indicator(objective_id, "Indicator 2"),
        ],
        selection_index: 0,
        editing_field: None,
    };
    
    // Verify the mode was set correctly
    if let InputMode::IndicatorManagement {
        objective_id: id,
        objective_title: title,
        indicators,
        selection_index,
        editing_field,
    } = &app.input_mode
    {
        assert_eq!(id, objective_id);
        assert_eq!(title, objective_title);
        assert_eq!(indicators.len(), 2);
        assert_eq!(*selection_index, 0);
        assert!(editing_field.is_none());
    } else {
        panic!("Wrong input mode");
    }
    
    Ok(())
}

#[test]
fn test_indicator_creation_mode_initialization() -> Result<()> {
    let (mut app, _temp_dir) = create_test_app()?;
    
    let objective_id = "test-obj-456";
    let objective_title = "Another Test Objective";
    
    // Set up indicator creation mode
    app.input_mode = InputMode::IndicatorCreation {
        objective_id: objective_id.to_string(),
        objective_title: objective_title.to_string(),
        field_index: 0,
        name_buffer: String::new(),
        kind: IndicatorKind::Leading,
        unit: IndicatorUnit::Count,
        unit_custom_buffer: String::new(),
        target_buffer: String::new(),
        direction: IndicatorDirection::HigherIsBetter,
        notes_buffer: String::new(),
    };
    
    // Verify the mode was set correctly
    if let InputMode::IndicatorCreation {
        objective_id: id,
        objective_title: title,
        field_index,
        name_buffer,
        kind,
        unit,
        direction,
        ..
    } = &app.input_mode
    {
        assert_eq!(id, objective_id);
        assert_eq!(title, objective_title);
        assert_eq!(*field_index, 0);
        assert!(name_buffer.is_empty());
        assert!(matches!(kind, IndicatorKind::Leading));
        assert!(matches!(unit, IndicatorUnit::Count));
        assert!(matches!(direction, IndicatorDirection::HigherIsBetter));
    } else {
        panic!("Wrong input mode");
    }
    
    Ok(())
}

#[test]
fn test_indicator_save_and_load() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let goals_dir = temp_dir.path().join("FocusFive/goals");
    fs::create_dir_all(&goals_dir)?;
    
    // Create config
    let config = Config {
        goals_dir: goals_dir.to_str().unwrap().to_string(),
        data_root: temp_dir.path().to_str().unwrap().to_string(),
    };
    
    // Create test indicators
    let mut indicators = IndicatorsData {
        version: 1,
        indicators: vec![
            create_test_indicator("obj-1", "Sales Calls"),
            create_test_indicator("obj-1", "Revenue Generated"),
            create_test_indicator("obj-2", "Hours Exercised"),
        ],
    };
    
    // Set different properties for variety
    indicators.indicators[0].kind = IndicatorKind::Leading;
    indicators.indicators[0].unit = IndicatorUnit::Count;
    
    indicators.indicators[1].kind = IndicatorKind::Lagging;
    indicators.indicators[1].unit = IndicatorUnit::Dollars;
    
    indicators.indicators[2].kind = IndicatorKind::Leading;
    indicators.indicators[2].unit = IndicatorUnit::Minutes;
    
    // Save indicators
    focusfive::data::save_indicators(&indicators, &config)?;
    
    // Load indicators
    let loaded = focusfive::data::load_or_create_indicators(&config)?;
    
    // Verify loaded data
    assert_eq!(loaded.indicators.len(), 3);
    assert_eq!(loaded.indicators[0].name, "Sales Calls");
    assert_eq!(loaded.indicators[1].name, "Revenue Generated");
    assert_eq!(loaded.indicators[2].name, "Hours Exercised");
    
    assert!(matches!(loaded.indicators[0].kind, IndicatorKind::Leading));
    assert!(matches!(loaded.indicators[1].kind, IndicatorKind::Lagging));
    assert!(matches!(loaded.indicators[0].unit, IndicatorUnit::Count));
    assert!(matches!(loaded.indicators[1].unit, IndicatorUnit::Dollars));
    assert!(matches!(loaded.indicators[2].unit, IndicatorUnit::Minutes));
    
    Ok(())
}

#[test]
fn test_indicator_filtering_by_objective() -> Result<()> {
    let indicators = vec![
        create_test_indicator("obj-1", "Indicator A"),
        create_test_indicator("obj-2", "Indicator B"),
        create_test_indicator("obj-1", "Indicator C"),
        create_test_indicator("obj-3", "Indicator D"),
    ];
    
    // Filter for obj-1
    let filtered: Vec<&IndicatorDef> = indicators
        .iter()
        .filter(|ind| ind.objective_id.as_deref() == Some("obj-1"))
        .collect();
    
    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0].name, "Indicator A");
    assert_eq!(filtered[1].name, "Indicator C");
    
    Ok(())
}

#[test]
fn test_indicator_active_status() -> Result<()> {
    let mut indicator = create_test_indicator("obj-1", "Test Indicator");
    
    // Test initial active state
    assert!(indicator.active);
    
    // Toggle to inactive
    indicator.active = false;
    assert!(!indicator.active);
    
    // Toggle back to active
    indicator.active = true;
    assert!(indicator.active);
    
    Ok(())
}

#[test]
fn test_indicator_target_values() -> Result<()> {
    let mut indicator = create_test_indicator("obj-1", "Test Indicator");
    
    // Test initial target
    assert_eq!(indicator.target, Some(10.0));
    
    // Update target
    indicator.target = Some(25.5);
    assert_eq!(indicator.target, Some(25.5));
    
    // Remove target
    indicator.target = None;
    assert!(indicator.target.is_none());
    
    Ok(())
}

#[test]
fn test_indicator_direction_variants() -> Result<()> {
    let mut indicator = create_test_indicator("obj-1", "Test Indicator");
    
    // Test HigherIsBetter
    indicator.direction = IndicatorDirection::HigherIsBetter;
    assert!(matches!(
        indicator.direction,
        IndicatorDirection::HigherIsBetter
    ));
    
    // Test LowerIsBetter
    indicator.direction = IndicatorDirection::LowerIsBetter;
    assert!(matches!(
        indicator.direction,
        IndicatorDirection::LowerIsBetter
    ));
    
    // Test WithinRange
    indicator.direction = IndicatorDirection::WithinRange;
    assert!(matches!(
        indicator.direction,
        IndicatorDirection::WithinRange
    ));
    
    Ok(())
}

#[test]
fn test_indicator_unit_variants() -> Result<()> {
    let mut indicator = create_test_indicator("obj-1", "Test Indicator");
    
    // Test all unit types
    let units = vec![
        IndicatorUnit::Count,
        IndicatorUnit::Minutes,
        IndicatorUnit::Dollars,
        IndicatorUnit::Percent,
        IndicatorUnit::Custom("Points".to_string()),
    ];
    
    for unit in units {
        indicator.unit = unit.clone();
        match &indicator.unit {
            IndicatorUnit::Count => assert!(true),
            IndicatorUnit::Minutes => assert!(true),
            IndicatorUnit::Dollars => assert!(true),
            IndicatorUnit::Percent => assert!(true),
            IndicatorUnit::Custom(s) => assert_eq!(s, "Points"),
        }
    }
    
    Ok(())
}

#[test]
fn test_indicator_lineage_tracking() -> Result<()> {
    let original_id = "original-123";
    let mut indicator = create_test_indicator("obj-1", "Evolving Indicator");
    
    // Initially no lineage
    assert!(indicator.lineage_of.is_none());
    
    // Set lineage to track previous version
    indicator.lineage_of = Some(original_id.to_string());
    assert_eq!(indicator.lineage_of, Some(original_id.to_string()));
    
    Ok(())
}

#[test]
fn test_indicator_notes_field() -> Result<()> {
    let mut indicator = create_test_indicator("obj-1", "Test Indicator");
    
    // Test initial notes
    assert_eq!(indicator.notes, Some("Test indicator".to_string()));
    
    // Update notes
    indicator.notes = Some("Updated notes with more detail".to_string());
    assert_eq!(
        indicator.notes,
        Some("Updated notes with more detail".to_string())
    );
    
    // Clear notes
    indicator.notes = None;
    assert!(indicator.notes.is_none());
    
    Ok(())
}