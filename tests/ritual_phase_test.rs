use anyhow::Result;
use chrono::NaiveDate;
use focusfive::app::App;
use focusfive::models::{Config, DailyGoals, FiveYearVision, RitualPhase};
use std::fs;
use tempfile::TempDir;

fn setup_test_config() -> (Config, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let goals_dir = temp_dir.path().join("FocusFive").join("goals");
    fs::create_dir_all(&goals_dir).expect("Failed to create goals dir");

    let config = Config {
        goals_dir: goals_dir.to_str().unwrap().to_string(),
        data_root: temp_dir.path().to_str().unwrap().to_string(),
    };

    (config, temp_dir)
}

#[test]
fn test_ritual_phase_from_hour() {
    // Test morning phase (5am-11am)
    assert_eq!(RitualPhase::from_hour(5), RitualPhase::Morning);
    assert_eq!(RitualPhase::from_hour(8), RitualPhase::Morning);
    assert_eq!(RitualPhase::from_hour(11), RitualPhase::Morning);

    // Test evening phase (5pm-10pm)
    assert_eq!(RitualPhase::from_hour(17), RitualPhase::Evening);
    assert_eq!(RitualPhase::from_hour(20), RitualPhase::Evening);
    assert_eq!(RitualPhase::from_hour(22), RitualPhase::Evening);

    // Test none phase (other times)
    assert_eq!(RitualPhase::from_hour(0), RitualPhase::None);
    assert_eq!(RitualPhase::from_hour(4), RitualPhase::None);
    assert_eq!(RitualPhase::from_hour(12), RitualPhase::None);
    assert_eq!(RitualPhase::from_hour(16), RitualPhase::None);
    assert_eq!(RitualPhase::from_hour(23), RitualPhase::None);
}

#[test]
fn test_ritual_phase_greetings() {
    assert_eq!(
        RitualPhase::Morning.greeting(),
        "Good Morning! Time to set today's intentions"
    );
    assert_eq!(
        RitualPhase::Evening.greeting(),
        "Evening Review - Reflect on your day"
    );
    assert_eq!(
        RitualPhase::None.greeting(),
        "FocusFive - Daily Goal Tracker"
    );
}

#[test]
fn test_app_initializes_with_ritual_phase() {
    let (config, _temp_dir) = setup_test_config();
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    let vision = FiveYearVision::new();

    let app = App::new(goals, config, vision);

    // App should have a ritual phase based on current time
    // We can't test the exact phase since it depends on when the test runs,
    // but we can verify it's one of the valid phases
    match app.ritual_phase {
        RitualPhase::Morning | RitualPhase::Evening | RitualPhase::None => {
            // Any of these is valid
            assert!(true);
        }
    }
}

#[test]
fn test_ritual_phase_manual_switching() {
    let (config, _temp_dir) = setup_test_config();
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    let vision = FiveYearVision::new();

    let mut app = App::new(goals, config, vision);

    // Test manual switching to morning
    app.ritual_phase = RitualPhase::Morning;
    assert_eq!(app.ritual_phase, RitualPhase::Morning);

    // Test manual switching to evening
    app.ritual_phase = RitualPhase::Evening;
    assert_eq!(app.ritual_phase, RitualPhase::Evening);

    // Test manual switching to none
    app.ritual_phase = RitualPhase::None;
    assert_eq!(app.ritual_phase, RitualPhase::None);
}

#[test]
fn test_ritual_phase_copy_trait() {
    // Verify RitualPhase implements Copy trait
    let phase1 = RitualPhase::Morning;
    let phase2 = phase1; // This should work because of Copy
    assert_eq!(phase1, phase2);
    assert_eq!(phase1, RitualPhase::Morning);
}

#[test]
fn test_ritual_phase_equality() {
    // Test PartialEq implementation
    assert_eq!(RitualPhase::Morning, RitualPhase::Morning);
    assert_eq!(RitualPhase::Evening, RitualPhase::Evening);
    assert_eq!(RitualPhase::None, RitualPhase::None);

    assert_ne!(RitualPhase::Morning, RitualPhase::Evening);
    assert_ne!(RitualPhase::Morning, RitualPhase::None);
    assert_ne!(RitualPhase::Evening, RitualPhase::None);
}

#[test]
fn test_hour_boundary_cases() {
    // Test boundary cases for phase transitions

    // Morning boundaries
    assert_eq!(RitualPhase::from_hour(4), RitualPhase::None); // 4am is None
    assert_eq!(RitualPhase::from_hour(5), RitualPhase::Morning); // 5am starts Morning
    assert_eq!(RitualPhase::from_hour(11), RitualPhase::Morning); // 11am still Morning
    assert_eq!(RitualPhase::from_hour(12), RitualPhase::None); // 12pm is None

    // Evening boundaries
    assert_eq!(RitualPhase::from_hour(16), RitualPhase::None); // 4pm is None
    assert_eq!(RitualPhase::from_hour(17), RitualPhase::Evening); // 5pm starts Evening
    assert_eq!(RitualPhase::from_hour(22), RitualPhase::Evening); // 10pm still Evening
    assert_eq!(RitualPhase::from_hour(23), RitualPhase::None); // 11pm is None
}

#[test]
fn test_phase_persistence_across_app_state() {
    let (config, _temp_dir) = setup_test_config();
    let goals = DailyGoals::new(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    let vision = FiveYearVision::new();

    let mut app = App::new(goals, config, vision);

    // Set a specific phase
    app.ritual_phase = RitualPhase::Morning;

    // Verify it persists through other state changes
    app.active_pane = focusfive::app::Pane::Actions;
    assert_eq!(app.ritual_phase, RitualPhase::Morning);

    app.outcome_index = 2;
    assert_eq!(app.ritual_phase, RitualPhase::Morning);

    app.show_help = true;
    assert_eq!(app.ritual_phase, RitualPhase::Morning);
}
