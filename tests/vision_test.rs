use chrono::Local;
use focusfive::data::{load_or_create_vision, save_vision};
use focusfive::models::{Config, FiveYearVision, OutcomeType};
use std::path::Path;

#[test]
fn test_vision_creation() {
    let vision = FiveYearVision::new();
    assert_eq!(vision.work, "");
    assert_eq!(vision.health, "");
    assert_eq!(vision.family, "");
    assert_eq!(vision.created, Local::now().date_naive());
    assert_eq!(vision.modified, Local::now().date_naive());
}

#[test]
fn test_vision_get_set() {
    let mut vision = FiveYearVision::new();

    // Set visions
    vision.set_vision(
        &OutcomeType::Work,
        "Build a successful tech company".to_string(),
    );
    vision.set_vision(&OutcomeType::Health, "Run a marathon".to_string());
    vision.set_vision(
        &OutcomeType::Family,
        "Travel the world together".to_string(),
    );

    // Get visions
    assert_eq!(
        vision.get_vision(&OutcomeType::Work),
        "Build a successful tech company"
    );
    assert_eq!(vision.get_vision(&OutcomeType::Health), "Run a marathon");
    assert_eq!(
        vision.get_vision(&OutcomeType::Family),
        "Travel the world together"
    );
}

#[test]
fn test_vision_length_limit() {
    let mut vision = FiveYearVision::new();

    // Create a string longer than MAX_VISION_LENGTH (1000)
    let long_text = "a".repeat(1500);
    vision.set_vision(&OutcomeType::Work, long_text);

    // Should be truncated to MAX_VISION_LENGTH
    assert_eq!(vision.work.len(), 1000);
}

#[test]
fn test_vision_persistence() {
    // Create a test config with temp directory
    let temp_dir = tempfile::tempdir().unwrap();
    let test_config = Config {
        goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
    };

    // Create and save a vision
    let mut vision = FiveYearVision::new();
    vision.set_vision(&OutcomeType::Work, "Test work vision".to_string());
    vision.set_vision(&OutcomeType::Health, "Test health vision".to_string());
    vision.set_vision(&OutcomeType::Family, "Test family vision".to_string());

    save_vision(&vision, &test_config).unwrap();

    // Check that vision.json was created
    let vision_path = Path::new(&test_config.goals_dir)
        .parent()
        .unwrap()
        .join("vision.json");
    assert!(vision_path.exists());

    // Load the vision back
    let loaded_vision = load_or_create_vision(&test_config).unwrap();

    assert_eq!(loaded_vision.work, "Test work vision");
    assert_eq!(loaded_vision.health, "Test health vision");
    assert_eq!(loaded_vision.family, "Test family vision");
}

#[test]
fn test_vision_json_format() {
    let mut vision = FiveYearVision::new();
    vision.set_vision(&OutcomeType::Work, "Work goal".to_string());
    vision.set_vision(&OutcomeType::Health, "Health goal".to_string());
    vision.set_vision(&OutcomeType::Family, "Family goal".to_string());

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&vision).unwrap();

    // Verify JSON contains expected fields
    assert!(json.contains("\"work\": \"Work goal\""));
    assert!(json.contains("\"health\": \"Health goal\""));
    assert!(json.contains("\"family\": \"Family goal\""));
    assert!(json.contains("\"created\""));
    assert!(json.contains("\"modified\""));

    // Deserialize back
    let deserialized: FiveYearVision = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.work, "Work goal");
    assert_eq!(deserialized.health, "Health goal");
    assert_eq!(deserialized.family, "Family goal");
}

#[test]
fn test_vision_multiline() {
    let mut vision = FiveYearVision::new();

    let multiline_vision = "Goal 1: Build a company\nGoal 2: Scale to 100 employees\nGoal 3: IPO";
    vision.set_vision(&OutcomeType::Work, multiline_vision.to_string());

    assert_eq!(vision.get_vision(&OutcomeType::Work), multiline_vision);
}
