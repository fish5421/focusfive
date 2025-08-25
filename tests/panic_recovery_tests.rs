use anyhow::Result;
use chrono::NaiveDate;
/// Panic Recovery and Graceful Degradation Tests
///
/// This test suite specifically validates that the FocusFive Phase 1 implementation
/// never panics under any circumstances and always provides graceful error handling.
///
/// Tests cover:
/// - Panic detection in all public APIs
/// - Resource cleanup on failures
/// - Error propagation without panics
/// - Memory safety under stress
/// - Recovery from partial failures
use focusfive::data::{load_or_create_goals, parse_markdown, read_goals_file, write_goals_file};
use focusfive::models::{Action, Config, DailyGoals};
use std::fs;
use std::panic;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[cfg(test)]
mod panic_safety_tests {
    use super::*;

    /// Test that parse_markdown never panics with any input
    #[test]
    fn test_parse_markdown_never_panics() {
        let dangerous_inputs = generate_dangerous_inputs();

        for (i, input) in dangerous_inputs.iter().enumerate() {
            let result = panic::catch_unwind(|| parse_markdown(input));

            match result {
                Ok(_) => {
                    // No panic - good! Check that result is reasonable
                    println!("✅ Input {} handled without panic", i);
                }
                Err(panic_info) => {
                    // Extract panic message if possible
                    let panic_msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "Unknown panic".to_string()
                    };

                    panic!(
                        "PANIC DETECTED in parse_markdown with input {}: {}\nInput preview: {:?}",
                        i,
                        panic_msg,
                        input.chars().take(100).collect::<String>()
                    );
                }
            }
        }

        println!(
            "✅ parse_markdown handled {} dangerous inputs without panicking",
            dangerous_inputs.len()
        );
    }

    /// Test that file I/O operations never panic
    #[test]
    fn test_file_operations_never_panic() -> Result<()> {
        let temp_dir = TempDir::new()?;

        let dangerous_configs = vec![
            Config {
                goals_dir: "/dev/null".to_string(),
            },
            Config {
                goals_dir: "/proc/cpuinfo".to_string(),
            },
            Config {
                goals_dir: "".to_string(),
            },
            Config {
                goals_dir: "\0".to_string(),
            },
            Config {
                goals_dir: "/".repeat(1000),
            },
            Config {
                goals_dir: temp_dir
                    .path()
                    .join("nonexistent")
                    .to_string_lossy()
                    .to_string(),
            },
        ];

        let dangerous_dates = vec![
            NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2100, 12, 31).unwrap(),
            NaiveDate::from_ymd_opt(2000, 2, 29).unwrap(), // Leap year
        ];

        for (config_i, config) in dangerous_configs.iter().enumerate() {
            for (date_i, &date) in dangerous_dates.iter().enumerate() {
                let goals = DailyGoals::new(date);

                // Test write_goals_file
                let write_result = panic::catch_unwind(|| write_goals_file(&goals, config));

                match write_result {
                    Ok(_) => {
                        println!(
                            "✅ write_goals_file config {} date {} no panic",
                            config_i, date_i
                        );
                    }
                    Err(_) => {
                        panic!(
                            "PANIC in write_goals_file with config {} date {}: {:?}",
                            config_i, date_i, config.goals_dir
                        );
                    }
                }

                // Test load_or_create_goals
                let load_result = panic::catch_unwind(|| load_or_create_goals(date, config));

                match load_result {
                    Ok(_) => {
                        println!(
                            "✅ load_or_create_goals config {} date {} no panic",
                            config_i, date_i
                        );
                    }
                    Err(_) => {
                        panic!(
                            "PANIC in load_or_create_goals with config {} date {}: {:?}",
                            config_i, date_i, config.goals_dir
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Test concurrent operations under stress don't cause panics
    #[test]
    fn test_concurrent_stress_no_panics() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Arc::new(Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
        });

        let panic_counter = Arc::new(Mutex::new(0));
        let error_counter = Arc::new(Mutex::new(0));
        let success_counter = Arc::new(Mutex::new(0));

        const NUM_THREADS: usize = 20;
        const OPERATIONS_PER_THREAD: usize = 100;

        let handles: Vec<_> = (0..NUM_THREADS)
            .map(|thread_id| {
                let config = Arc::clone(&config);
                let panic_counter = Arc::clone(&panic_counter);
                let error_counter = Arc::clone(&error_counter);
                let success_counter = Arc::clone(&success_counter);

                thread::spawn(move || {
                    for op_id in 0..OPERATIONS_PER_THREAD {
                        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
                            + chrono::Duration::days(
                                (thread_id * OPERATIONS_PER_THREAD + op_id) as i64,
                            );

                        // Create goals with stress-inducing content
                        let mut goals = DailyGoals::new(date);
                        goals.work.actions[0] = Action {
                            text: format!(
                                "Thread {} op {} {}",
                                thread_id,
                                op_id,
                                "X".repeat(thread_id * 10)
                            ),
                            completed: (thread_id + op_id) % 3 == 0,
                        };

                        // Try write operation with panic detection
                        let write_result =
                            panic::catch_unwind(|| write_goals_file(&goals, &config));

                        match write_result {
                            Ok(result) => match result {
                                Ok(_) => {
                                    *success_counter.lock().unwrap() += 1;
                                }
                                Err(_) => {
                                    *error_counter.lock().unwrap() += 1;
                                }
                            },
                            Err(_) => {
                                *panic_counter.lock().unwrap() += 1;
                            }
                        }

                        // Try read operation with panic detection
                        let read_result =
                            panic::catch_unwind(|| load_or_create_goals(date, &config));

                        match read_result {
                            Ok(result) => match result {
                                Ok(_) => {
                                    *success_counter.lock().unwrap() += 1;
                                }
                                Err(_) => {
                                    *error_counter.lock().unwrap() += 1;
                                }
                            },
                            Err(_) => {
                                *panic_counter.lock().unwrap() += 1;
                            }
                        }

                        // Small delay to increase contention
                        thread::sleep(Duration::from_millis(1));
                    }
                })
            })
            .collect();

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        let panics = *panic_counter.lock().unwrap();
        let errors = *error_counter.lock().unwrap();
        let successes = *success_counter.lock().unwrap();

        println!("Concurrent stress test results:");
        println!("  Successes: {}", successes);
        println!("  Errors: {}", errors);
        println!("  Panics: {}", panics);

        assert_eq!(
            panics, 0,
            "No panics should occur during concurrent stress test"
        );
        assert!(successes > 0, "Some operations should succeed");

        Ok(())
    }

    /// Test memory pressure scenarios don't cause panics
    #[test]
    fn test_memory_pressure_no_panics() {
        let memory_intensive_scenarios = vec![
            // Very long single action
            ("Single huge action", create_goals_with_huge_action()),
            // Many medium actions (this will be truncated to 3, but shouldn't panic)
            ("Many actions", create_goals_with_many_actions()),
            // Goals with unicode and special characters
            ("Unicode stress", create_goals_with_unicode_stress()),
            // Goals with control characters
            ("Control characters", create_goals_with_control_chars()),
        ];

        for (scenario_name, goals) in memory_intensive_scenarios {
            // Test serialization to markdown
            let markdown_result =
                panic::catch_unwind(|| focusfive::data::generate_markdown(&goals));

            match markdown_result {
                Ok(markdown) => {
                    // Test parsing the generated markdown
                    let parse_result = panic::catch_unwind(|| parse_markdown(&markdown));

                    match parse_result {
                        Ok(_) => {
                            println!(
                                "✅ Memory scenario '{}' handled without panic",
                                scenario_name
                            );
                        }
                        Err(_) => {
                            panic!("PANIC in parse_markdown for scenario '{}'", scenario_name);
                        }
                    }
                }
                Err(_) => {
                    panic!(
                        "PANIC in generate_markdown for scenario '{}'",
                        scenario_name
                    );
                }
            }
        }
    }

    /// Test that resource cleanup happens even on errors
    #[test]
    fn test_resource_cleanup_on_errors() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
        };

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let goals = DailyGoals::new(date);

        // Count initial files
        fs::create_dir_all(&config.goals_dir)?;
        let initial_files = count_files_in_dir(&config.goals_dir)?;

        // Force multiple write attempts that might fail
        for i in 0..10 {
            let mut test_goals = goals.clone();
            test_goals.work.actions[0] = Action {
                text: format!("Attempt {} with large content {}", i, "X".repeat(100_000)),
                completed: false,
            };

            let result = panic::catch_unwind(|| write_goals_file(&test_goals, &config));

            // Ensure no panics regardless of success/failure
            match result {
                Ok(_) => {
                    // Write succeeded or failed gracefully
                }
                Err(_) => {
                    panic!("Panic detected during write attempt {}", i);
                }
            }
        }

        // Check for resource leaks (temporary files)
        let final_files = count_files_in_dir(&config.goals_dir)?;
        let temp_files = count_temp_files_in_dir(&config.goals_dir)?;

        println!(
            "File count: {} -> {} (temp files: {})",
            initial_files, final_files, temp_files
        );

        // Should not have excessive temporary files (some might be acceptable)
        assert!(
            temp_files < 5,
            "Should not have excessive temporary files: {}",
            temp_files
        );

        Ok(())
    }

    /// Test error propagation without panics
    #[test]
    fn test_error_propagation_no_panics() {
        let error_inducing_inputs = vec![
            "",                                                             // Empty
            "# Invalid date format",                                        // Bad date
            "# January 15, 2025\n## InvalidOutcome\n- [x] Task",            // Unknown outcome
            "# January 15, 2025\n## Work\n- [invalid] Bad checkbox",        // Bad checkbox
            "# January 15, 2025\n## Work\n- [x] " + &"X".repeat(1_000_000), // Huge action
        ];

        for (i, input) in error_inducing_inputs.iter().enumerate() {
            let result = panic::catch_unwind(|| parse_markdown(input));

            match result {
                Ok(parse_result) => {
                    // No panic - check that either success or proper error
                    match parse_result {
                        Ok(_) => {
                            println!("✅ Error-inducing input {} unexpectedly succeeded", i);
                        }
                        Err(e) => {
                            // Verify error is well-formed
                            let error_msg = e.to_string();
                            assert!(!error_msg.is_empty(), "Error message should not be empty");
                            assert!(
                                !error_msg.contains("panic"),
                                "Error should not mention panic"
                            );

                            // Verify error chain is reasonable
                            let chain_count = e.chain().count();
                            assert!(
                                chain_count >= 1 && chain_count <= 10,
                                "Error chain should be reasonable length: {}",
                                chain_count
                            );

                            println!(
                                "✅ Error-inducing input {} properly rejected: {}",
                                i, error_msg
                            );
                        }
                    }
                }
                Err(_) => {
                    panic!(
                        "PANIC detected with error-inducing input {}: {:?}",
                        i,
                        input.chars().take(50).collect::<String>()
                    );
                }
            }
        }
    }

    // Helper functions for creating test scenarios

    fn generate_dangerous_inputs() -> Vec<String> {
        vec![
            // Empty and whitespace
            String::new(),
            " ".repeat(1000),
            "\n".repeat(1000),
            "\t".repeat(1000),
            // Control characters
            format!("# January 15, 2025{}\n## Work\n- [x] Task", '\0'),
            (0..32u8).map(|b| b as char).collect::<String>(),
            // Unicode edge cases
            "\u{FEFF}# January 15, 2025".to_string(), // BOM
            "# January 15, 2025 \u{202E}REVERSED".to_string(), // RTL override
            // Extremely long lines
            "# ".to_string() + &"January 15, 2025 ".repeat(100_000),
            "## Work\n- [x] ".to_string() + &"Very long task ".repeat(100_000),
            // Malformed structures
            "## ".repeat(1000),
            "- [".repeat(1000),
            "(".repeat(1000) + &")".repeat(1000),
            // Binary data disguised as text
            String::from_utf8_lossy(&(0..255u8).cycle().take(10000).collect::<Vec<_>>())
                .to_string(),
            // Regex stress patterns
            "## Work (".to_string() + &"(".repeat(1000) + "Goal: test" + &")".repeat(1000) + ")",
            // Mixed valid/invalid
            "# January 15, 2025\n\n## Work\n- [x] Valid task\n".to_string()
                + &String::from_utf8_lossy(&vec![0xFF; 1000]),
        ]
    }

    fn create_goals_with_huge_action() -> DailyGoals {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);

        goals.work.actions[0] = Action {
            text: "Huge action: ".to_string() + &"X".repeat(1_000_000),
            completed: false,
        };

        goals
    }

    fn create_goals_with_many_actions() -> DailyGoals {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);

        // Fill all actions with medium-sized text
        for i in 0..3 {
            goals.work.actions[i] = Action {
                text: format!("Action {} with content: {}", i, "Y".repeat(10_000)),
                completed: i % 2 == 0,
            };
        }

        goals
    }

    fn create_goals_with_unicode_stress() -> DailyGoals {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);

        let unicode_stress = "🎯🚀💪🧘‍♂️👨‍👩‍👧‍👦🌟⭐️✨🔥💯🎉🎊🎈🎁🎀🎂🍰🧁".repeat(1000);

        goals.work.actions[0] = Action {
            text: unicode_stress,
            completed: true,
        };

        goals
    }

    fn create_goals_with_control_chars() -> DailyGoals {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);

        let control_chars: String = (0..32u8).map(|b| b as char).cycle().take(1000).collect();

        goals.work.actions[0] = Action {
            text: format!("Task with controls: {}", control_chars),
            completed: false,
        };

        goals
    }

    fn count_files_in_dir(dir: &str) -> Result<usize> {
        match fs::read_dir(dir) {
            Ok(entries) => Ok(entries.count()),
            Err(_) => Ok(0), // Directory doesn't exist
        }
    }

    fn count_temp_files_in_dir(dir: &str) -> Result<usize> {
        match fs::read_dir(dir) {
            Ok(entries) => Ok(entries
                .filter_map(|e| e.ok())
                .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
                .count()),
            Err(_) => Ok(0),
        }
    }
}

#[cfg(test)]
mod graceful_degradation_tests {
    use super::*;

    /// Test that partial data is better than no data
    #[test]
    fn test_partial_data_recovery() {
        let partial_markdown_examples = vec![
            // Only header, no outcomes
            "# January 15, 2025 - Day 12",
            
            // Header + one outcome
            "# January 15, 2025\n\n## Work\n- [x] Single task",
            
            // Header + incomplete outcome
            "# January 15, 2025\n\n## Work\n- [x] Task 1\n- [ ] Task 2",
            
            // All outcomes but some incomplete
            "# January 15, 2025\n\n## Work\n- [x] Work task\n\n## Health\n- [ ] Health task\n\n## Family",
        ];

        for (i, partial_content) in partial_markdown_examples.iter().enumerate() {
            let result = parse_markdown(partial_content);

            match result {
                Ok(goals) => {
                    // Should have valid date
                    assert_eq!(goals.date.month(), 1);
                    assert_eq!(goals.date.day(), 15);

                    // Should have all three outcomes (even if empty)
                    assert_eq!(goals.outcomes().len(), 3);

                    println!("✅ Partial data {} recovered successfully", i);
                }
                Err(e) => {
                    // If it fails, should be graceful
                    let error_msg = e.to_string();
                    assert!(!error_msg.contains("panic"));
                    println!("✅ Partial data {} rejected gracefully: {}", i, error_msg);
                }
            }
        }
    }

    /// Test that the system continues working even after errors
    #[test]
    fn test_continued_operation_after_errors() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
        };

        // Sequence of operations: some will fail, some will succeed
        let operations = vec![
            // This should succeed
            (
                NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                "Normal task",
                true,
            ),
            // This might fail due to filesystem issues, but shouldn't break subsequent operations
            (
                NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
                &"X".repeat(1_000_000),
                false,
            ),
            // This should succeed even after previous failure
            (
                NaiveDate::from_ymd_opt(2025, 1, 3).unwrap(),
                "Recovery task",
                true,
            ),
            // Another potential failure
            (
                NaiveDate::from_ymd_opt(2025, 1, 4).unwrap(),
                &format!("Unicode stress: {}", "🎯".repeat(100_000)),
                false,
            ),
            // Final success to prove system still works
            (
                NaiveDate::from_ymd_opt(2025, 1, 5).unwrap(),
                "Final task",
                true,
            ),
        ];

        let mut success_count = 0;
        let mut error_count = 0;

        for (date, task_text, should_succeed) in operations {
            let mut goals = DailyGoals::new(date);
            goals.work.actions[0] = Action {
                text: task_text.to_string(),
                completed: false,
            };

            let result = write_goals_file(&goals, &config);

            match result {
                Ok(_) => {
                    success_count += 1;

                    // Verify we can still read it back
                    let loaded = load_or_create_goals(date, &config)?;
                    assert_eq!(loaded.date, date);

                    if should_succeed {
                        println!("✅ Expected success for {}", date);
                    } else {
                        println!("⚠️  Unexpected success for {}", date);
                    }
                }
                Err(e) => {
                    error_count += 1;

                    // Error should be graceful
                    let error_msg = e.to_string();
                    assert!(!error_msg.contains("panic"));

                    if should_succeed {
                        println!("⚠️  Unexpected error for {}: {}", date, error_msg);
                    } else {
                        println!("✅ Expected error for {}: {}", date, error_msg);
                    }
                }
            }
        }

        println!(
            "Operation sequence: {} successes, {} errors",
            success_count, error_count
        );

        // Should have at least some successes, proving system still works
        assert!(
            success_count >= 2,
            "Should have at least some successful operations"
        );

        Ok(())
    }

    /// Test that errors provide actionable information
    #[test]
    fn test_actionable_error_messages() {
        let error_scenarios = vec![
            ("", "Empty file should provide clear guidance"),
            (
                "Not a markdown file",
                "Invalid format should explain expected format",
            ),
            (
                "# February 30, 2025",
                "Invalid date should suggest valid dates",
            ),
            (
                "# January 15, 2025\n## InvalidOutcome",
                "Invalid outcome should list valid outcomes",
            ),
        ];

        for (input, expectation) in error_scenarios {
            let result = parse_markdown(input);

            if let Err(error) = result {
                let error_msg = error.to_string();

                // Error message should be informative
                assert!(!error_msg.is_empty(), "Error message should not be empty");
                assert!(error_msg.len() > 10, "Error message should be descriptive");

                // Should not contain internal implementation details
                assert!(
                    !error_msg.contains("unwrap"),
                    "Should not leak internal implementation"
                );
                assert!(!error_msg.contains("panic"), "Should not mention panics");

                // Should provide some context about what went wrong
                assert!(
                    error_msg.contains("file")
                        || error_msg.contains("date")
                        || error_msg.contains("header")
                        || error_msg.contains("format")
                        || error_msg.contains("Invalid"),
                    "Error should provide context: {}",
                    error_msg
                );

                println!(
                    "✅ Error for '{}': {}",
                    input.chars().take(20).collect::<String>(),
                    error_msg
                );
            } else {
                println!("⚠️  Expected error for '{}' but parsing succeeded", input);
            }
        }
    }
}
