use anyhow::Result;
use chrono::{Local, NaiveDate};
use focusfive::data::{load_or_create_objectives, save_objectives};
use focusfive::models::{Config, Objective, ObjectiveStatus, ObjectivesData, OutcomeType};
use tempfile::TempDir;

#[test]
fn test_objective_serialization_deserialization() -> Result<()> {
    // Test all enum variants and date serialization
    let mut objectives = ObjectivesData {
        version: 1,
        objectives: vec![
            Objective {
                id: "test-id-1".to_string(),
                domain: OutcomeType::Work,
                title: "Grow MRR to $50k".to_string(),
                description: Some("Increase monthly recurring revenue".to_string()),
                start: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                end: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
                status: ObjectiveStatus::Active,
                created: chrono::Utc::now(),
                modified: chrono::Utc::now(),
                parent_id: None,
            },
            Objective {
                id: "test-id-2".to_string(),
                domain: OutcomeType::Health,
                title: "Run a marathon".to_string(),
                description: None,
                start: NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
                end: None, // Open-ended
                status: ObjectiveStatus::Paused,
                created: chrono::Utc::now(),
                modified: chrono::Utc::now(),
                parent_id: Some("parent-id".to_string()),
            },
            Objective {
                id: "test-id-3".to_string(),
                domain: OutcomeType::Family,
                title: "Weekly family dinners".to_string(),
                description: Some("Have dinner together every Sunday".to_string()),
                start: Local::now().date_naive(),
                end: Some(NaiveDate::from_ymd_opt(2025, 6, 30).unwrap()),
                status: ObjectiveStatus::Completed,
                created: chrono::Utc::now(),
                modified: chrono::Utc::now(),
                parent_id: None,
            },
        ],
    };

    // Test JSON serialization
    let json = serde_json::to_string_pretty(&objectives)?;
    assert!(json.contains("\"version\": 1"));
    assert!(json.contains("\"domain\": \"Work\""));
    assert!(json.contains("\"domain\": \"Health\""));
    assert!(json.contains("\"domain\": \"Family\""));
    assert!(json.contains("\"status\": \"Active\""));
    assert!(json.contains("\"status\": \"Paused\""));
    assert!(json.contains("\"status\": \"Completed\""));
    assert!(json.contains("\"title\": \"Grow MRR to $50k\""));

    // Test deserialization
    let deserialized: ObjectivesData = serde_json::from_str(&json)?;
    assert_eq!(deserialized.version, 1);
    assert_eq!(deserialized.objectives.len(), 3);

    // Verify first objective
    assert_eq!(deserialized.objectives[0].id, "test-id-1");
    assert_eq!(deserialized.objectives[0].domain, OutcomeType::Work);
    assert_eq!(deserialized.objectives[0].title, "Grow MRR to $50k");
    assert_eq!(deserialized.objectives[0].status, ObjectiveStatus::Active);
    assert!(deserialized.objectives[0].description.is_some());
    assert!(deserialized.objectives[0].end.is_some());
    assert!(deserialized.objectives[0].parent_id.is_none());

    // Verify second objective
    assert_eq!(deserialized.objectives[1].status, ObjectiveStatus::Paused);
    assert!(deserialized.objectives[1].description.is_none());
    assert!(deserialized.objectives[1].end.is_none());
    assert!(deserialized.objectives[1].parent_id.is_some());

    // Test dropped status
    objectives.objectives[2].status = ObjectiveStatus::Dropped;
    let json2 = serde_json::to_string(&objectives)?;
    assert!(json2.contains("\"Dropped\""));

    Ok(())
}

#[test]
fn test_objective_creation() -> Result<()> {
    // Test the new() constructor
    let obj = Objective::new(OutcomeType::Work, "Test objective".to_string());

    assert!(!obj.id.is_empty());
    assert_eq!(obj.domain, OutcomeType::Work);
    assert_eq!(obj.title, "Test objective");
    assert!(obj.description.is_none());
    assert_eq!(obj.start, Local::now().date_naive());
    assert!(obj.end.is_none());
    assert_eq!(obj.status, ObjectiveStatus::Active);
    assert!(obj.parent_id.is_none());

    // Verify UUID format
    assert!(obj.id.len() >= 32); // UUIDs are at least 32 chars when stringified

    Ok(())
}

#[test]
fn test_objectives_save_and_load() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Create test objectives
    let objectives = ObjectivesData {
        version: 1,
        objectives: vec![
            Objective::new(OutcomeType::Work, "Ship feature X".to_string()),
            Objective::new(OutcomeType::Health, "Lose 10 pounds".to_string()),
        ],
    };

    // Save objectives
    let save_path = save_objectives(&objectives, &config)?;
    assert!(save_path.exists());
    assert!(save_path.to_string_lossy().ends_with("objectives.json"));

    // Load objectives back
    let loaded = load_or_create_objectives(&config)?;
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.objectives.len(), 2);
    assert_eq!(loaded.objectives[0].title, "Ship feature X");
    assert_eq!(loaded.objectives[1].title, "Lose 10 pounds");

    Ok(())
}

#[test]
fn test_objectives_load_creates_default() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Load when file doesn't exist should return default
    let objectives = load_or_create_objectives(&config)?;
    assert_eq!(objectives.version, 1);
    assert_eq!(objectives.objectives.len(), 0);

    Ok(())
}

#[test]
fn test_objectives_with_hierarchical_structure() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Create parent objective
    let parent = Objective::new(OutcomeType::Work, "Q1 Goals".to_string());
    let parent_id = parent.id.clone();

    // Create child objectives
    let mut child1 = Objective::new(OutcomeType::Work, "Complete project A".to_string());
    child1.parent_id = Some(parent_id.clone());

    let mut child2 = Objective::new(OutcomeType::Work, "Launch feature B".to_string());
    child2.parent_id = Some(parent_id.clone());

    let objectives = ObjectivesData {
        version: 1,
        objectives: vec![parent, child1, child2],
    };

    // Save and reload
    save_objectives(&objectives, &config)?;
    let loaded = load_or_create_objectives(&config)?;

    // Verify hierarchy preserved
    assert_eq!(loaded.objectives[0].parent_id, None);
    assert_eq!(loaded.objectives[1].parent_id, Some(parent_id.clone()));
    assert_eq!(loaded.objectives[2].parent_id, Some(parent_id));

    Ok(())
}
