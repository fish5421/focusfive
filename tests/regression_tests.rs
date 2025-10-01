use chrono::NaiveDate;
use focusfive::data::{generate_markdown, parse_markdown, write_goals_file};
use focusfive::models::{Action, Config, DailyGoals};
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

#[test]
fn test_no_panic_on_missing_home_directory() {
    // Test that Config::new() doesn't panic when HOME is missing
    let original_home = std::env::var("HOME").ok();
    std::env::remove_var("HOME");

    let config = Config::new();
    assert!(
        config.is_ok(),
        "Config::new() should not panic without HOME"
    );

    // Restore HOME if it existed
    if let Some(home) = original_home {
        std::env::set_var("HOME", home);
    }
}

#[test]
fn test_config_default_is_safe() {
    // Test that Default trait implementation doesn't panic
    let config = Config::default();
    assert!(!config.goals_dir.is_empty());
}

#[test]
fn test_concurrent_writes_no_collision() {
    let dir = TempDir::new().unwrap();
    let config = Config {
        goals_dir: dir.path().to_string_lossy().to_string(),
        data_root: dir.path().to_string_lossy().to_string(),
    };
    let config = Arc::new(config);

    let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

    // Spawn 10 threads writing simultaneously
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let config = config.clone();
            thread::spawn(move || {
                let mut goals = DailyGoals::new(date);
                goals.work.actions[0] = Action::new(format!("Thread {} task", i));
                write_goals_file(&goals, &config).unwrap();
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Check that we have exactly one .md file (no temp files left)
    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".md") || name.contains(".tmp")
        })
        .collect();

    assert_eq!(
        entries.len(),
        1,
        "Should have exactly one file, no temp files"
    );
    assert!(entries[0].file_name().to_string_lossy().ends_with(".md"));
}

#[test]
fn test_header_with_leading_content() {
    let markdown = r#"<!-- This is a comment -->

Some random text here

# January 15, 2025 - Day 12

## Work
- [x] Task 1
- [ ] Task 2  
- [ ] Task 3

## Health
- [ ] Exercise
- [ ] Meal prep
- [ ] Sleep early

## Family
- [x] Call parents
- [ ] Plan weekend
- [ ] Help with homework"#;

    let result = parse_markdown(markdown);
    assert!(result.is_ok(), "Should parse with leading content");

    let goals = result.unwrap();
    assert_eq!(goals.date, NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    assert_eq!(goals.day_number, Some(12));
    assert!(goals.work.actions[0].completed);
}

#[test]
fn test_case_insensitive_outcome_headers() {
    let markdown = r#"# January 15, 2025

## WORK
- [x] Capital letters work
- [ ] Task 2
- [ ] Task 3

## health
- [ ] Lowercase works
- [ ] Task 2
- [ ] Task 3

## FaMiLy
- [ ] Mixed case works
- [x] Task 2
- [ ] Task 3"#;

    let result = parse_markdown(markdown);
    assert!(result.is_ok(), "Should parse case-insensitive headers");

    let goals = result.unwrap();
    assert!(goals.work.actions[0].completed);
    assert!(goals.family.actions[1].completed);
}

#[test]
fn test_no_panic_on_malformed_regex_captures() {
    // Test various malformed inputs that would cause panics with array indexing
    let bad_inputs = vec![
        "# Not a valid date",
        "## Work without date header",
        "- [x] Action without header",
        "Random text",
        "",
    ];

    for input in bad_inputs {
        let result = parse_markdown(input);
        // Should return error, not panic
        assert!(result.is_err(), "Should error on: {}", input);
    }
}

#[test]
fn test_action_text_truncation() {
    let long_text = "x".repeat(600); // Longer than MAX_ACTION_LENGTH
    let action = Action::new(long_text.clone());

    assert_eq!(
        action.text.len(),
        500,
        "Action text should be truncated to 500 chars"
    );
    assert!(!action.completed);
}

#[test]
fn test_warning_on_extra_actions() {
    let markdown = r#"# January 15, 2025

## Work
- [ ] Task 1
- [ ] Task 2
- [ ] Task 3
- [ ] Task 4 - This should trigger warning
- [ ] Task 5 - This too

## Health
- [ ] Task 1
- [ ] Task 2
- [ ] Task 3

## Family
- [ ] Task 1
- [ ] Task 2
- [ ] Task 3"#;

    // This should parse successfully but only keep first 3 actions
    let result = parse_markdown(markdown);
    assert!(result.is_ok());

    let goals = result.unwrap();
    // Should only have 3 actions despite 5 in markdown
    assert_eq!(goals.work.actions.len(), 3);
}

#[test]
fn test_round_trip_with_special_cases() {
    let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
    let mut goals = DailyGoals::new(date);
    goals.day_number = Some(42);

    // Set up goals with edge cases
    goals.work.goal = Some("Handle (parentheses) & special chars!".to_string());
    goals.work.actions[0] =
        Action::from_markdown("Task with [brackets] and symbols @#$".to_string(), true);

    // Generate markdown and parse it back
    let markdown = generate_markdown(&goals);
    let parsed = parse_markdown(&markdown).unwrap();

    // Verify round-trip preserves data
    assert_eq!(parsed.date, goals.date);
    assert_eq!(parsed.day_number, goals.day_number);
    assert_eq!(parsed.work.goal, goals.work.goal);
    assert_eq!(parsed.work.actions[0].text, goals.work.actions[0].text);
    assert_eq!(
        parsed.work.actions[0].completed,
        goals.work.actions[0].completed
    );
}

#[test]
fn test_empty_file_handling() {
    let result = parse_markdown("");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No valid date header"));
}

#[test]
fn test_partial_data_handling() {
    // Missing some outcomes
    let markdown = r#"# January 15, 2025

## Work
- [x] Only work section exists
- [ ] Task 2
- [ ] Task 3"#;

    let result = parse_markdown(markdown);
    assert!(result.is_ok());

    let goals = result.unwrap();
    assert!(goals.work.actions[0].completed);
    // Other outcomes should have empty actions
    assert_eq!(goals.health.actions[0].text, "");
    assert_eq!(goals.family.actions[0].text, "");
}

#[test]
fn test_date_formats() {
    let test_cases = vec![
        (
            "# January 1, 2025",
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        ),
        (
            "# Feb 28, 2024",
            NaiveDate::from_ymd_opt(2024, 2, 28).unwrap(),
        ),
        (
            "# December 31, 2025",
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        ),
    ];

    for (header, expected_date) in test_cases {
        let markdown = format!("{}\n\n## Work\n- [ ] Test", header);
        let result = parse_markdown(&markdown);
        assert!(result.is_ok(), "Failed to parse: {}", header);
        assert_eq!(result.unwrap().date, expected_date);
    }
}

// Integration test module
#[cfg(test)]
mod integration {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_full_workflow() {
        let dir = TempDir::new().unwrap();
        let config = Config {
            goals_dir: dir.path().to_string_lossy().to_string(),
            data_root: dir.path().to_string_lossy().to_string(),
        };

        // Create goals
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);
        goals.work.goal = Some("Ship MVP".to_string());
        goals.work.actions[0] = Action::new("Fix critical bugs".to_string());
        goals.work.actions[0].completed = true;

        // Write to file
        let path = write_goals_file(&goals, &config).unwrap();
        assert!(path.exists());

        // Read back
        let content = fs::read_to_string(&path).unwrap();
        let loaded = parse_markdown(&content).unwrap();

        // Verify
        assert_eq!(loaded.date, goals.date);
        assert_eq!(loaded.work.goal, goals.work.goal);
        assert_eq!(loaded.work.actions[0].text, goals.work.actions[0].text);
        assert_eq!(
            loaded.work.actions[0].completed,
            goals.work.actions[0].completed
        );
    }
}
