use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use focusfive::app::{App, InputMode};
use focusfive::models::{Config, DailyGoals, FiveYearVision, OutcomeType};
use std::fs;
use tempfile::TempDir;

fn setup_test_app() -> (App, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let goals_dir = temp_dir.path().join("FocusFive").join("goals");
    fs::create_dir_all(&goals_dir).expect("Failed to create goals dir");

    let config = Config {
        goals_dir: goals_dir.to_str().unwrap().to_string(),
        data_root: temp_dir.path().to_str().unwrap().to_string(),
    };

    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    let vision = FiveYearVision::new();

    let app = App::new(goals, config, vision);
    (app, temp_dir)
}

#[test]
fn test_ctrl_enter_saves_vision_on_windows_linux() {
    let (mut app, _temp_dir) = setup_test_app();

    // Enter vision editing mode
    app.input_mode = InputMode::VisionEditing {
        outcome_type: OutcomeType::Work,
        buffer: "Test vision".to_string(),
        original: "".to_string(),
    };

    // Simulate Ctrl+Enter
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
    let _ = app.handle_key(key);

    // Should be back in normal mode
    assert_eq!(app.input_mode, InputMode::Normal);
    // Vision should be saved
    assert_eq!(app.vision.get_vision(&OutcomeType::Work), "Test vision");
}

#[test]
fn test_cmd_enter_saves_vision_on_mac() {
    let (mut app, _temp_dir) = setup_test_app();

    // Enter vision editing mode
    app.input_mode = InputMode::VisionEditing {
        outcome_type: OutcomeType::Work,
        buffer: "Test vision Mac".to_string(),
        original: "".to_string(),
    };

    // Simulate Cmd+Enter (SUPER modifier on Mac)
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::SUPER);
    let _ = app.handle_key(key);

    // Should be back in normal mode
    assert_eq!(app.input_mode, InputMode::Normal);
    // Vision should be saved
    assert_eq!(app.vision.get_vision(&OutcomeType::Work), "Test vision Mac");
}

#[test]
fn test_ctrl_enter_saves_reflection_on_windows_linux() {
    let (mut app, _temp_dir) = setup_test_app();

    // Enter reflection mode
    app.input_mode = InputMode::Reflecting {
        outcome_type: OutcomeType::Health,
        buffer: "Today was great!".to_string(),
        original: "".to_string(),
    };

    // Simulate Ctrl+Enter
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
    let _ = app.handle_key(key);

    // Should be back in normal mode
    assert_eq!(app.input_mode, InputMode::Normal);
    // Reflection should be saved
    assert_eq!(
        app.goals.health.reflection,
        Some("Today was great!".to_string())
    );
}

#[test]
fn test_cmd_enter_saves_reflection_on_mac() {
    let (mut app, _temp_dir) = setup_test_app();

    // Enter reflection mode
    app.input_mode = InputMode::Reflecting {
        outcome_type: OutcomeType::Family,
        buffer: "Quality time spent".to_string(),
        original: "".to_string(),
    };

    // Simulate Cmd+Enter (SUPER modifier on Mac)
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::SUPER);
    let _ = app.handle_key(key);

    // Should be back in normal mode
    assert_eq!(app.input_mode, InputMode::Normal);
    // Reflection should be saved
    assert_eq!(
        app.goals.family.reflection,
        Some("Quality time spent".to_string())
    );
}

#[test]
fn test_plain_enter_adds_newline_in_vision_mode() {
    let (mut app, _temp_dir) = setup_test_app();

    // Enter vision editing mode
    app.input_mode = InputMode::VisionEditing {
        outcome_type: OutcomeType::Work,
        buffer: "Line 1".to_string(),
        original: "".to_string(),
    };

    // Simulate plain Enter (no modifiers)
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
    let _ = app.handle_key(key);

    // Should still be in vision editing mode with newline added
    if let InputMode::VisionEditing { buffer, .. } = &app.input_mode {
        assert_eq!(buffer, "Line 1\n");
    } else {
        panic!("Should still be in vision editing mode");
    }
}

#[test]
fn test_plain_enter_adds_newline_in_reflection_mode() {
    let (mut app, _temp_dir) = setup_test_app();

    // Enter reflection mode
    app.input_mode = InputMode::Reflecting {
        outcome_type: OutcomeType::Health,
        buffer: "First line".to_string(),
        original: "".to_string(),
    };

    // Simulate plain Enter (no modifiers)
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
    let _ = app.handle_key(key);

    // Should still be in reflection mode with newline added
    if let InputMode::Reflecting { buffer, .. } = &app.input_mode {
        assert_eq!(buffer, "First line\n");
    } else {
        panic!("Should still be in reflection mode");
    }
}

#[test]
fn test_both_modifiers_work_simultaneously() {
    let (mut app, _temp_dir) = setup_test_app();

    // Test with both modifiers pressed (edge case)
    app.input_mode = InputMode::VisionEditing {
        outcome_type: OutcomeType::Work,
        buffer: "Both modifiers".to_string(),
        original: "".to_string(),
    };

    // Simulate both Ctrl and Cmd pressed
    let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL | KeyModifiers::SUPER);
    let _ = app.handle_key(key);

    // Should save and exit to normal mode
    assert_eq!(app.input_mode, InputMode::Normal);
    assert_eq!(app.vision.get_vision(&OutcomeType::Work), "Both modifiers");
}
