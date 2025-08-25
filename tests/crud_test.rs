use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use focusfive::app::{App, InputMode, Pane};
use focusfive::data::{read_goals_file, write_goals_file};
use focusfive::models::{Config, DailyGoals};
use std::fs;
use std::path::Path;

#[test]
fn test_crud_operations() {
    // Setup test environment
    let config = Config::default();
    let test_dir = Path::new(&config.goals_dir);
    fs::create_dir_all(test_dir).unwrap();

    // Create initial goals
    let date = Local::now().date_naive();
    let mut goals = DailyGoals::new(date);

    // Set some initial data
    goals.work.actions[0].text = "Initial task".to_string();
    goals.work.actions[0].completed = false;

    // Create app
    let mut app = App::new(goals, config.clone());

    // Test 1: Enter edit mode
    app.active_pane = Pane::Actions;
    app.outcome_index = 0; // Work
    app.action_index = 0; // First action

    // Press 'e' to enter edit mode
    let edit_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key(edit_key).unwrap();

    // Verify we're in edit mode
    match &app.input_mode {
        InputMode::Editing { buffer, original } => {
            assert_eq!(buffer, "Initial task");
            assert_eq!(original, "Initial task");
        }
        _ => panic!("Should be in edit mode"),
    }

    // Test 2: Modify text in edit mode
    let char_key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
    app.handle_key(char_key).unwrap();
    let char_key = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE);
    app.handle_key(char_key).unwrap();

    // Verify buffer updated
    match &app.input_mode {
        InputMode::Editing { buffer, .. } => {
            assert_eq!(buffer, "Initial task 2");
        }
        _ => panic!("Should still be in edit mode"),
    }

    // Test 3: Save the edit
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(enter_key).unwrap();

    // Verify we're back in normal mode and text is updated
    assert!(matches!(app.input_mode, InputMode::Normal));
    assert_eq!(app.goals.work.actions[0].text, "Initial task 2");
    assert!(app.needs_save);

    // Test 4: Delete an action
    app.action_index = 1; // Move to second action
    app.goals.work.actions[1].text = "Task to delete".to_string();

    let delete_key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    app.handle_key(delete_key).unwrap();

    // Verify action is cleared
    assert_eq!(app.goals.work.actions[1].text, "");
    assert!(!app.goals.work.actions[1].completed);
    assert!(app.needs_save);

    // Test 5: Save to file and verify persistence
    let file_path = write_goals_file(&app.goals, &config).unwrap();

    // Read back and verify
    let loaded_goals = read_goals_file(&file_path).unwrap();
    assert_eq!(loaded_goals.work.actions[0].text, "Initial task 2");
    assert_eq!(loaded_goals.work.actions[1].text, "");

    // Cleanup
    let _ = fs::remove_file(file_path);
}

#[test]
fn test_edit_mode_escape() {
    let config = Config::default();
    let date = Local::now().date_naive();
    let mut goals = DailyGoals::new(date);
    goals.work.actions[0].text = "Original text".to_string();

    let mut app = App::new(goals, config);
    app.active_pane = Pane::Actions;
    app.outcome_index = 0;
    app.action_index = 0;

    // Enter edit mode
    let edit_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key(edit_key).unwrap();

    // Type some changes
    let char_key = KeyEvent::new(KeyCode::Char('X'), KeyModifiers::NONE);
    app.handle_key(char_key).unwrap();

    // Press Escape to cancel
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key(esc_key).unwrap();

    // Verify we're back in normal mode and text is unchanged
    assert!(matches!(app.input_mode, InputMode::Normal));
    assert_eq!(app.goals.work.actions[0].text, "Original text");
    assert!(!app.needs_save);
}

#[test]
fn test_backspace_in_edit_mode() {
    let config = Config::default();
    let date = Local::now().date_naive();
    let mut goals = DailyGoals::new(date);
    goals.work.actions[0].text = "Test".to_string();

    let mut app = App::new(goals, config);
    app.active_pane = Pane::Actions;
    app.outcome_index = 0;
    app.action_index = 0;

    // Enter edit mode
    let edit_key = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key(edit_key).unwrap();

    // Press backspace twice
    let backspace_key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
    app.handle_key(backspace_key).unwrap();
    app.handle_key(backspace_key).unwrap();

    // Verify buffer is updated
    match &app.input_mode {
        InputMode::Editing { buffer, .. } => {
            assert_eq!(buffer, "Te");
        }
        _ => panic!("Should be in edit mode"),
    }

    // Save the changes
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(enter_key).unwrap();

    assert_eq!(app.goals.work.actions[0].text, "Te");
}
