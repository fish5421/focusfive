use anyhow::Result;
use chrono::NaiveDate;
use focusfive::data::{load_or_create_templates, save_templates};
use focusfive::models::{ActionTemplates, OutcomeType};
use std::fs;
use tempfile::TempDir;

fn setup_test_config() -> (focusfive::models::Config, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let goals_dir = temp_dir.path().join("FocusFive").join("goals");
    fs::create_dir_all(&goals_dir).expect("Failed to create goals dir");

    let config = focusfive::models::Config {
        goals_dir: goals_dir.to_str().unwrap().to_string(),
    };

    (config, temp_dir)
}

#[test]
fn test_action_templates_creation() {
    let mut templates = ActionTemplates::new();

    // Test adding a template
    let actions = vec![
        "Review pull requests".to_string(),
        "Write documentation".to_string(),
        "Deploy to staging".to_string(),
    ];

    templates.add_template("Daily Dev Tasks".to_string(), actions.clone());

    // Verify template was added
    assert_eq!(templates.get_template_names().len(), 1);
    assert_eq!(templates.get_template_names()[0], "Daily Dev Tasks");

    // Verify template contents
    let retrieved = templates.get_template("Daily Dev Tasks").unwrap();
    assert_eq!(retrieved.len(), 3);
    assert_eq!(retrieved[0], "Review pull requests");
}

#[test]
fn test_template_persistence() -> Result<()> {
    let (config, _temp_dir) = setup_test_config();

    // Create and save templates
    let mut templates = ActionTemplates::new();
    templates.add_template(
        "Morning Routine".to_string(),
        vec![
            "Check emails".to_string(),
            "Review calendar".to_string(),
            "Plan priorities".to_string(),
        ],
    );

    save_templates(&templates, &config)?;

    // Load templates back
    let loaded = load_or_create_templates(&config)?;

    assert_eq!(loaded.get_template_names().len(), 1);
    assert_eq!(loaded.get_template_names()[0], "Morning Routine");

    let actions = loaded.get_template("Morning Routine").unwrap();
    assert_eq!(actions.len(), 3);
    assert_eq!(actions[0], "Check emails");

    Ok(())
}

#[test]
fn test_template_limits() {
    let mut templates = ActionTemplates::new();

    // Test that only 3 actions are kept even if more are provided
    let actions = vec![
        "Task 1".to_string(),
        "Task 2".to_string(),
        "Task 3".to_string(),
        "Task 4".to_string(), // This should be ignored
        "Task 5".to_string(), // This should be ignored
    ];

    templates.add_template("Too Many Tasks".to_string(), actions);

    let retrieved = templates.get_template("Too Many Tasks").unwrap();
    assert_eq!(retrieved.len(), 3, "Should only keep first 3 actions");
    assert_eq!(retrieved[2], "Task 3");
}

#[test]
fn test_template_action_length_limit() {
    let mut templates = ActionTemplates::new();

    // Create a very long action text
    let long_text: String = "x".repeat(600); // Exceeds MAX_ACTION_LENGTH of 500

    let actions = vec![long_text.clone()];
    templates.add_template("Long Action".to_string(), actions);

    let retrieved = templates.get_template("Long Action").unwrap();
    assert_eq!(
        retrieved[0].len(),
        500,
        "Action should be truncated to 500 chars"
    );
}

#[test]
fn test_multiple_templates() {
    let mut templates = ActionTemplates::new();

    // Add multiple templates
    templates.add_template(
        "Work Tasks".to_string(),
        vec!["Code review".to_string(), "Sprint planning".to_string()],
    );

    templates.add_template(
        "Health Tasks".to_string(),
        vec!["Morning walk".to_string(), "Drink water".to_string()],
    );

    templates.add_template(
        "Family Tasks".to_string(),
        vec!["Call parents".to_string(), "Plan weekend".to_string()],
    );

    // Verify all templates exist
    let names = templates.get_template_names();
    assert_eq!(names.len(), 3);

    // Names should be sorted alphabetically
    assert_eq!(names[0], "Family Tasks");
    assert_eq!(names[1], "Health Tasks");
    assert_eq!(names[2], "Work Tasks");
}

#[test]
fn test_remove_template() {
    let mut templates = ActionTemplates::new();

    // Add templates
    templates.add_template("Template 1".to_string(), vec!["Task 1".to_string()]);
    templates.add_template("Template 2".to_string(), vec!["Task 2".to_string()]);

    assert_eq!(templates.get_template_names().len(), 2);

    // Remove a template
    let removed = templates.remove_template("Template 1");
    assert!(removed);
    assert_eq!(templates.get_template_names().len(), 1);
    assert_eq!(templates.get_template_names()[0], "Template 2");

    // Try to remove non-existent template
    let removed = templates.remove_template("Non-existent");
    assert!(!removed);
    assert_eq!(templates.get_template_names().len(), 1);
}

#[test]
fn test_empty_templates_file() -> Result<()> {
    let (config, _temp_dir) = setup_test_config();

    // Load templates when no file exists
    let templates = load_or_create_templates(&config)?;

    assert_eq!(templates.get_template_names().len(), 0);
    assert!(templates.get_template("Any").is_none());

    Ok(())
}

#[test]
fn test_update_existing_template() {
    let mut templates = ActionTemplates::new();

    // Add initial template
    templates.add_template(
        "Daily Tasks".to_string(),
        vec!["Task A".to_string(), "Task B".to_string()],
    );

    // Update the same template
    templates.add_template(
        "Daily Tasks".to_string(),
        vec![
            "New Task 1".to_string(),
            "New Task 2".to_string(),
            "New Task 3".to_string(),
        ],
    );

    // Verify it was updated, not duplicated
    assert_eq!(templates.get_template_names().len(), 1);

    let actions = templates.get_template("Daily Tasks").unwrap();
    assert_eq!(actions.len(), 3);
    assert_eq!(actions[0], "New Task 1");
}
