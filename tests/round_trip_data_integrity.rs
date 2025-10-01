use anyhow::Result;
use chrono::NaiveDate;
use focusfive::data::{generate_markdown, load_or_create_goals, parse_markdown, write_goals_file};
use focusfive::models::{Action, Config, DailyGoals, Outcome, OutcomeType};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

/// Comprehensive Round-Trip Data Integrity Test Suite
///
/// This test suite validates that data remains intact through complete parse→modify→save→parse cycles.
/// It focuses on ensuring no data loss, corruption, or format inconsistencies occur during the full
/// round-trip workflow that users will experience in real-world usage.
///
/// Test Philosophy:
/// - Every test should perform complete round-trips, not just individual functions
/// - Edge cases and Unicode content must be preserved exactly
/// - Formatting consistency should be maintained across cycles
/// - Performance should remain acceptable even with complex data
/// - Error conditions should not corrupt existing data

#[cfg(test)]
mod round_trip_tests {
    use super::*;

    // =====================================
    // Test 1: Basic Round-Trip Validation
    // =====================================

    #[test]
    fn test_basic_round_trip_integrity() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
            data_root: temp_dir.path().to_string_lossy().to_string(),
        };

        // Create initial goals with basic data
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut original_goals = DailyGoals::new(date);
        original_goals.day_number = Some(42);

        // Populate with test data
        original_goals.work.goal = Some("Complete project".to_string());
        original_goals.work.actions[0] = Action::from_markdown("Write tests".to_string(), true);
        original_goals.work.actions[1] = Action::from_markdown("Review code".to_string(), false);
        original_goals.work.actions[2] =
            Action::from_markdown("Deploy to staging".to_string(), true);

        original_goals.health.goal = Some("Daily exercise".to_string());
        original_goals.health.actions[0] = Action::from_markdown("Morning run".to_string(), true);
        original_goals.health.actions[1] =
            Action::from_markdown("Strength training".to_string(), false);
        original_goals.health.actions[2] =
            Action::from_markdown("Evening stretching".to_string(), false);

        original_goals.family.goal = Some("Quality time".to_string());
        original_goals.family.actions[0] = Action::from_markdown("Call parents".to_string(), false);
        original_goals.family.actions[1] = Action::from_markdown("Family dinner".to_string(), true);
        original_goals.family.actions[2] =
            Action::from_markdown("Bedtime stories".to_string(), true);

        // Round trip 1: Save and load
        write_goals_file(&original_goals, &config)?;
        let first_load = load_or_create_goals(date, &config)?;

        // Verify complete equality
        assert_eq!(original_goals, first_load);

        // Round trip 2: Modify and save again
        let mut modified_goals = first_load;
        modified_goals.work.actions[1].completed = true; // Mark as completed
        modified_goals.health.goal = Some("Daily exercise - updated".to_string());

        write_goals_file(&modified_goals, &config)?;
        let second_load = load_or_create_goals(date, &config)?;

        // Verify modifications persisted
        assert_eq!(modified_goals, second_load);
        assert!(second_load.work.actions[1].completed);
        assert_eq!(
            second_load.health.goal,
            Some("Daily exercise - updated".to_string())
        );

        // Round trip 3: Convert to markdown and back
        let markdown = generate_markdown(&second_load);
        let parsed_from_markdown = parse_markdown(&markdown)?;

        // Verify markdown round-trip integrity
        assert_eq!(second_load, parsed_from_markdown);

        Ok(())
    }

    // =====================================
    // Test 2: Unicode and Special Characters
    // =====================================

    #[test]
    fn test_unicode_preservation_round_trip() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
            data_root: temp_dir.path().to_string_lossy().to_string(),
        };

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);

        // Test various Unicode content
        goals.work.goal = Some("完成项目 🎯".to_string());
        goals.work.actions[0] =
            Action::from_markdown("编写测试 ✅ with émojis 🚀".to_string(), true);
        goals.work.actions[1] =
            Action::from_markdown("Español: Revisar código con acentos y ñ".to_string(), false);
        goals.work.actions[2] =
            Action::from_markdown("Русский: Проверить кодировку UTF-8".to_string(), true);

        goals.health.goal = Some("健康目标 💪".to_string());
        goals.health.actions[0] = Action::from_markdown("晨跑 🏃‍♂️ 5公里".to_string(), true);
        goals.health.actions[1] =
            Action::from_markdown("Café ☕ and méditation 🧘".to_string(), false);
        goals.health.actions[2] =
            Action::from_markdown("Ωμέγα-3 supplements & vitamins".to_string(), false);

        goals.family.goal = Some("Familie Zeit 👨‍👩‍👧‍👦".to_string());
        goals.family.actions[0] =
            Action::from_markdown("Appeler les parents 📞 à 19h".to_string(), false);
        goals.family.actions[1] = Action::from_markdown("مشاهدة فيلم عائلي".to_string(), true);
        goals.family.actions[2] = Action::from_markdown(
            "Read bedtime story: \"The 🐻 and the 🍯\"".to_string(),
            true,
        );

        // Multiple round-trips to ensure Unicode stability
        for round in 1..=5 {
            // Save to file
            write_goals_file(&goals, &config)?;

            // Load back
            let loaded = load_or_create_goals(date, &config)?;

            // Convert to markdown
            let markdown = generate_markdown(&loaded);

            // Parse from markdown
            let parsed = parse_markdown(&markdown)?;

            // Verify complete equality in each round
            assert_eq!(goals, loaded, "Round {} failed at file I/O", round);
            assert_eq!(goals, parsed, "Round {} failed at markdown parsing", round);

            // Update for next round (minor modification)
            goals.day_number = Some(round as u32);
        }

        Ok(())
    }

    #[test]
    fn test_special_characters_and_formatting() -> Result<()> {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);

        // Test edge case characters that might interfere with markdown parsing
        goals.work.goal = Some("Goal with [brackets] and (parentheses) and #hash".to_string());
        goals.work.actions[0] =
            Action::from_markdown("Task with - [x] false checkbox text".to_string(), true);
        goals.work.actions[1] =
            Action::from_markdown("Task with ## header-like text".to_string(), false);
        goals.work.actions[2] =
            Action::from_markdown("Task with\nembedded newlines\nand\ttabs".to_string(), true);

        goals.health.goal = Some("Goal: with colons and & ampersands".to_string());
        goals.health.actions[0] = Action::from_markdown(
            "Action with * asterisks * and _ underscores _".to_string(),
            true,
        );
        goals.health.actions[1] =
            Action::from_markdown("Action with `backticks` and |pipes|".to_string(), false);
        goals.health.actions[2] = Action::from_markdown(
            "Action with quotes: \"double\" and 'single'".to_string(),
            false,
        );

        goals.family.goal = Some("Goal with < > angle brackets".to_string());
        goals.family.actions[0] =
            Action::from_markdown("HTML-like <tag>content</tag> in action".to_string(), false);
        goals.family.actions[1] =
            Action::from_markdown("Math symbols: α + β = γ, ∑, ∫, ∞".to_string(), true);
        goals.family.actions[2] =
            Action::from_markdown("Symbols: © ® ™ ¶ § † ‡ • ◦ ‣".to_string(), true);

        // Multiple markdown round-trips
        let original_markdown = generate_markdown(&goals);

        for round in 1..=3 {
            let parsed = parse_markdown(&original_markdown)?;
            let regenerated_markdown = generate_markdown(&parsed);
            let reparsed = parse_markdown(&regenerated_markdown)?;

            // Verify stability across multiple conversions
            assert_eq!(goals, parsed, "Round {} initial parsing failed", round);
            assert_eq!(
                goals, reparsed,
                "Round {} regeneration parsing failed",
                round
            );
            assert_eq!(
                original_markdown, regenerated_markdown,
                "Round {} markdown format changed",
                round
            );
        }

        Ok(())
    }

    // =====================================
    // Test 3: Edge Cases and Boundary Conditions
    // =====================================

    #[test]
    fn test_empty_and_minimal_data_round_trip() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
            data_root: temp_dir.path().to_string_lossy().to_string(),
        };

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        // Test completely empty goals
        let empty_goals = DailyGoals::new(date);

        write_goals_file(&empty_goals, &config)?;
        let loaded_empty = load_or_create_goals(date, &config)?;
        assert_eq!(empty_goals, loaded_empty);

        // Test goals with empty strings
        let mut minimal_goals = DailyGoals::new(date);
        minimal_goals.work.goal = Some(String::new());
        minimal_goals.health.goal = None;
        minimal_goals.family.goal = Some("   ".to_string()); // Whitespace only

        // All actions remain empty by default
        for outcome in minimal_goals.outcomes_mut() {
            for action in &mut outcome.actions {
                action.text = String::new();
                action.completed = false;
            }
        }

        write_goals_file(&minimal_goals, &config)?;
        let loaded_minimal = load_or_create_goals(date, &config)?;
        assert_eq!(minimal_goals, loaded_minimal);

        // Test single character content
        let mut single_char_goals = DailyGoals::new(date);
        single_char_goals.work.goal = Some("X".to_string());
        single_char_goals.work.actions[0] = Action::from_markdown("A".to_string(), true);
        single_char_goals.work.actions[1] = Action::from_markdown(
            "🎯".to_string(), // Single emoji
            false,
        );
        single_char_goals.work.actions[2] = Action::from_markdown(
            "中".to_string(), // Single CJK character
            true,
        );

        write_goals_file(&single_char_goals, &config)?;
        let loaded_single_char = load_or_create_goals(date, &config)?;
        assert_eq!(single_char_goals, loaded_single_char);

        Ok(())
    }

    #[test]
    fn test_maximum_length_content_round_trip() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
            data_root: temp_dir.path().to_string_lossy().to_string(),
        };

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut max_goals = DailyGoals::new(date);

        // Create very long content to test limits
        let long_goal = "This is a very long goal description that tests the system's ability to handle extensive text content without data loss or corruption. ".repeat(100);
        let long_action = "This is an extremely detailed action description that contains comprehensive information about the task, including context, requirements, steps, and expected outcomes. ".repeat(50);

        max_goals.work.goal = Some(long_goal.clone());
        max_goals.health.goal = Some(long_goal.clone());
        max_goals.family.goal = Some(long_goal);

        for outcome in max_goals.outcomes_mut() {
            for (i, action) in outcome.actions.iter_mut().enumerate() {
                action.text = format!("{} - Action #{}", long_action, i + 1);
                action.completed = i % 2 == 0;
            }
        }

        // Test multiple round-trips with large content
        for round in 1..=3 {
            write_goals_file(&max_goals, &config)?;
            let loaded = load_or_create_goals(date, &config)?;

            // Verify all content preserved
            assert_eq!(
                max_goals, loaded,
                "Round {} failed with large content",
                round
            );

            // Verify specific long content
            assert!(loaded.work.goal.as_ref().unwrap().len() > 10000);
            assert!(loaded.work.actions[0].text.len() > 5000);

            // Convert to markdown and back
            let markdown = generate_markdown(&loaded);
            let parsed = parse_markdown(&markdown)?;
            assert_eq!(loaded, parsed, "Round {} markdown conversion failed", round);
        }

        Ok(())
    }

    // =====================================
    // Test 4: Date and Metadata Preservation
    // =====================================

    #[test]
    fn test_date_preservation_edge_cases() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
            data_root: temp_dir.path().to_string_lossy().to_string(),
        };

        // Test edge dates
        let edge_dates = vec![
            NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(), // Very old
            NaiveDate::from_ymd_opt(2000, 2, 29).unwrap(), // Leap year
            NaiveDate::from_ymd_opt(2100, 12, 31).unwrap(), // Far future
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(), // Year end
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), // Year start
        ];

        for (i, date) in edge_dates.iter().enumerate() {
            let mut goals = DailyGoals::new(*date);
            goals.day_number = Some(i as u32 + 1);
            goals.work.goal = Some(format!("Goal for {}", date.format("%Y-%m-%d")));

            // Full round-trip
            write_goals_file(&goals, &config)?;
            let loaded = load_or_create_goals(*date, &config)?;

            assert_eq!(goals.date, loaded.date);
            assert_eq!(goals.day_number, loaded.day_number);
            assert_eq!(goals, loaded);

            // Markdown round-trip
            let markdown = generate_markdown(&loaded);
            let parsed = parse_markdown(&markdown)?;

            assert_eq!(loaded.date, parsed.date);
            assert_eq!(loaded.day_number, parsed.day_number);
            assert_eq!(loaded, parsed);
        }

        Ok(())
    }

    #[test]
    fn test_day_number_edge_cases() -> Result<()> {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        // Test various day number values
        let day_numbers = vec![
            None,
            Some(0),
            Some(1),
            Some(365),
            Some(366), // Leap year
            Some(1000),
            Some(u32::MAX),
        ];

        for day_num in day_numbers {
            let mut goals = DailyGoals::new(date);
            goals.day_number = day_num;

            // Markdown round-trip test
            let markdown = generate_markdown(&goals);
            let parsed = parse_markdown(&markdown)?;

            assert_eq!(goals.day_number, parsed.day_number);
            assert_eq!(goals, parsed);
        }

        Ok(())
    }

    // =====================================
    // Test 5: Completion Status Consistency
    // =====================================

    #[test]
    fn test_completion_status_round_trip() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
            data_root: temp_dir.path().to_string_lossy().to_string(),
        };

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        // Test all possible completion combinations
        let completion_patterns = vec![
            [false, false, false], // All incomplete
            [true, true, true],    // All complete
            [true, false, true],   // Mixed pattern 1
            [false, true, false],  // Mixed pattern 2
            [true, false, false],  // Single complete
            [false, false, true],  // Last complete
        ];

        for (pattern_idx, pattern) in completion_patterns.iter().enumerate() {
            let mut goals = DailyGoals::new(date);
            goals.day_number = Some(pattern_idx as u32);

            // Apply pattern to all outcomes
            for outcome in goals.outcomes_mut() {
                for (i, action) in outcome.actions.iter_mut().enumerate() {
                    action.text = format!("Action {} for pattern {}", i + 1, pattern_idx);
                    action.completed = pattern[i];
                }
            }

            // Multiple round-trips
            for round in 1..=3 {
                write_goals_file(&goals, &config)?;
                let loaded = load_or_create_goals(date, &config)?;

                // Verify completion status preserved
                for outcome in loaded.outcomes() {
                    for (i, action) in outcome.actions.iter().enumerate() {
                        assert_eq!(
                            action.completed, pattern[i],
                            "Pattern {} round {} failed for action {}",
                            pattern_idx, round, i
                        );
                    }
                }

                assert_eq!(goals, loaded);

                // Markdown round-trip
                let markdown = generate_markdown(&loaded);
                let parsed = parse_markdown(&markdown)?;
                assert_eq!(loaded, parsed);
            }
        }

        Ok(())
    }

    // =====================================
    // Test 6: Whitespace and Formatting Consistency
    // =====================================

    #[test]
    fn test_whitespace_preservation() -> Result<()> {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);

        // Test various whitespace scenarios
        goals.work.goal = Some("  Goal with leading and trailing spaces  ".to_string());
        goals.work.actions[0] =
            Action::from_markdown("Action with  multiple  internal  spaces".to_string(), true);
        goals.work.actions[1] =
            Action::from_markdown("\tAction with tabs\tand spaces\t".to_string(), false);
        goals.work.actions[2] =
            Action::from_markdown("Action\nwith\nline\nbreaks".to_string(), true);

        goals.health.goal = Some("Goal\r\nwith\r\nCRLF".to_string());
        goals.health.actions[0] = Action::from_markdown("Action with trailing\n".to_string(), true);
        goals.health.actions[1] = Action::from_markdown("\nAction with leading".to_string(), false);
        goals.health.actions[2] = Action::from_markdown("Normal action".to_string(), false);

        // Test markdown round-trips preserve meaningful whitespace
        let original_markdown = generate_markdown(&goals);

        for round in 1..=5 {
            let parsed = parse_markdown(&original_markdown)?;
            let regenerated_markdown = generate_markdown(&parsed);

            // Note: Some whitespace normalization is expected in markdown processing
            // The test verifies that the data model remains consistent
            assert_eq!(
                goals.work.goal, parsed.work.goal,
                "Round {} goal whitespace changed",
                round
            );

            // Action text should be preserved exactly as stored in the model
            for (outcome_idx, outcome) in parsed.outcomes().iter().enumerate() {
                for (action_idx, action) in outcome.actions.iter().enumerate() {
                    let expected_action = &goals.outcomes()[outcome_idx].actions[action_idx];
                    assert_eq!(
                        expected_action.text, action.text,
                        "Round {} outcome {} action {} text changed",
                        round, outcome_idx, action_idx
                    );
                    assert_eq!(
                        expected_action.completed, action.completed,
                        "Round {} outcome {} action {} completion changed",
                        round, outcome_idx, action_idx
                    );
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_cross_platform_line_endings() -> Result<()> {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        // Test content with different line ending styles
        let test_cases = vec![
            ("Unix LF", "Line 1\nLine 2\nLine 3"),
            ("Windows CRLF", "Line 1\r\nLine 2\r\nLine 3"),
            ("Old Mac CR", "Line 1\rLine 2\rLine 3"),
            ("Mixed", "Line 1\nLine 2\r\nLine 3\rLine 4"),
        ];

        for (case_name, content) in test_cases {
            let mut goals = DailyGoals::new(date);
            goals.work.goal = Some(format!("Goal with {} line endings", case_name));
            goals.work.actions[0] = Action::from_markdown(content.to_string(), true);

            // Markdown round-trip should handle line endings gracefully
            let markdown = generate_markdown(&goals);
            let parsed = parse_markdown(&markdown)?;

            // The exact line ending preservation might be normalized,
            // but the content structure should remain intact
            assert_eq!(goals.work.goal, parsed.work.goal);
            assert_eq!(
                goals.work.actions[0].completed,
                parsed.work.actions[0].completed
            );

            // Content should be parseable regardless of line endings
            assert!(
                !parsed.work.actions[0].text.is_empty(),
                "Content lost for case: {}",
                case_name
            );
        }

        Ok(())
    }

    // =====================================
    // Test 7: Rapid Modification Cycles
    // =====================================

    #[test]
    fn test_rapid_modification_cycles() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
            data_root: temp_dir.path().to_string_lossy().to_string(),
        };

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);

        // Initial setup
        goals.work.goal = Some("Evolving goal".to_string());
        goals.work.actions[0] = Action::from_markdown("Dynamic action".to_string(), false);

        // Simulate rapid modification cycles
        for cycle in 1..=100 {
            // Modify data
            goals.day_number = Some(cycle);
            goals.work.goal = Some(format!("Goal updated {} times", cycle));
            goals.work.actions[0].text = format!("Action iteration {}", cycle);
            goals.work.actions[0].completed = cycle % 2 == 0;

            // Save and reload
            write_goals_file(&goals, &config)?;
            let loaded = load_or_create_goals(date, &config)?;

            // Verify integrity
            assert_eq!(goals, loaded, "Cycle {} failed", cycle);

            // Markdown round-trip every 10th cycle
            if cycle % 10 == 0 {
                let markdown = generate_markdown(&loaded);
                let parsed = parse_markdown(&markdown)?;
                assert_eq!(
                    loaded, parsed,
                    "Markdown round-trip failed at cycle {}",
                    cycle
                );
            }

            // Update for next cycle
            goals = loaded;
        }

        Ok(())
    }

    #[test]
    fn test_incremental_data_growth() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
            data_root: temp_dir.path().to_string_lossy().to_string(),
        };

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);

        // Start with minimal data and gradually add more
        for iteration in 1..=50 {
            // Incrementally build up content
            goals.day_number = Some(iteration);

            // Add to work goal
            let current_work_goal = goals.work.goal.unwrap_or_default();
            goals.work.goal = Some(format!("{} Step {}", current_work_goal, iteration));

            // Extend action text
            for action in &mut goals.work.actions {
                action.text = format!("{} +{}", action.text, iteration);
                if iteration % 3 == 0 {
                    action.completed = !action.completed;
                }
            }

            // Add health content every 5 iterations
            if iteration % 5 == 0 {
                goals.health.goal = Some(format!("Health goal at iteration {}", iteration));
                goals.health.actions[0].text = format!("Health action {}", iteration);
            }

            // Add family content every 7 iterations
            if iteration % 7 == 0 {
                goals.family.goal = Some(format!("Family goal at iteration {}", iteration));
                goals.family.actions[0].text = format!("Family action {}", iteration);
            }

            // Full round-trip test
            write_goals_file(&goals, &config)?;
            let loaded = load_or_create_goals(date, &config)?;
            assert_eq!(goals, loaded, "Iteration {} failed", iteration);

            // Markdown stability test every 10 iterations
            if iteration % 10 == 0 {
                let markdown = generate_markdown(&loaded);
                let parsed = parse_markdown(&markdown)?;
                assert_eq!(
                    loaded, parsed,
                    "Markdown parsing failed at iteration {}",
                    iteration
                );

                // Verify content growth
                assert!(
                    markdown.len() > iteration * 10,
                    "Content not growing as expected"
                );
            }
        }

        Ok(())
    }

    // =====================================
    // Test 8: Error Recovery and Partial Data
    // =====================================

    #[test]
    fn test_partial_data_recovery() -> Result<()> {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        // Test parsing markdown with missing sections
        let partial_markdown = r#"# January 15, 2025 - Day 42

## Work (Goal: Complete project)
- [x] Task 1
- [ ] Task 2

## Health
- [x] Exercise
"#;

        let parsed = parse_markdown(partial_markdown)?;
        assert_eq!(parsed.date, date);
        assert_eq!(parsed.day_number, Some(42));
        assert_eq!(parsed.work.goal, Some("Complete project".to_string()));
        assert!(parsed.work.actions[0].completed);
        assert!(!parsed.work.actions[1].completed);

        // Work should have partial data, others should be default
        assert_eq!(parsed.work.actions[2].text, ""); // Third action missing
        assert_eq!(parsed.health.goal, None); // No goal specified
        assert!(parsed.health.actions[0].completed); // First action present
        assert_eq!(parsed.health.actions[1].text, ""); // Second action missing

        // Family section completely missing - should be defaults
        assert_eq!(parsed.family.goal, None);
        assert_eq!(parsed.family.actions[0].text, "");

        // Test that partial data can be saved and loaded
        let regenerated = generate_markdown(&parsed);
        let reparsed = parse_markdown(&regenerated)?;
        assert_eq!(parsed, reparsed);

        Ok(())
    }

    #[test]
    fn test_malformed_input_handling() -> Result<()> {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        // Test various malformed inputs that should not crash
        let malformed_cases = vec![
            // Missing date
            "## Work\n- [x] Task",
            // Invalid date format
            "# Not a date\n## Work\n- [x] Task",
            // Invalid action format
            "# January 15, 2025\n## Work\n- [invalid] Task",
            // Malformed headers
            "# January 15, 2025\n### Work\n- [x] Task",
            // Extra content
            "# January 15, 2025\n## Work\n- [x] Task\nExtra line\n## NotAnOutcome\n- [x] Should be ignored",
        ];

        for (i, malformed) in malformed_cases.iter().enumerate() {
            match parse_markdown(malformed) {
                Ok(goals) => {
                    // If parsing succeeds, ensure the result is valid
                    assert_eq!(
                        goals.outcomes().len(),
                        3,
                        "Case {} should have 3 outcomes",
                        i
                    );

                    // Should be able to regenerate without errors
                    let regenerated = generate_markdown(&goals);
                    let reparsed = parse_markdown(&regenerated)?;
                    assert_eq!(goals, reparsed, "Case {} regeneration failed", i);
                }
                Err(e) => {
                    // If parsing fails, that's acceptable for malformed input
                    println!("Case {} failed as expected: {}", i, e);
                }
            }
        }

        Ok(())
    }

    // =====================================
    // Test 9: Performance Under Load
    // =====================================

    #[test]
    fn test_performance_round_trip_cycles() -> Result<()> {
        use std::time::Instant;

        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
            data_root: temp_dir.path().to_string_lossy().to_string(),
        };

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);

        // Set up realistic content
        goals.work.goal = Some("Performance test goal".to_string());
        for i in 0..3 {
            goals.work.actions[i] = Action::from_markdown(
                format!(
                    "Performance test action {} with moderate length content",
                    i + 1
                ),
                i % 2 == 0,
            );
        }

        let start_time = Instant::now();
        const CYCLES: usize = 1000;

        // Perform many round-trip cycles
        for cycle in 1..=CYCLES {
            // Modify slightly each cycle
            goals.day_number = Some(cycle as u32);
            goals.work.actions[0].text = format!("Action updated {} times", cycle);

            // Full round-trip: save -> load -> markdown -> parse
            write_goals_file(&goals, &config)?;
            let loaded = load_or_create_goals(date, &config)?;
            let markdown = generate_markdown(&loaded);
            let parsed = parse_markdown(&markdown)?;

            // Verify integrity maintained
            assert_eq!(loaded, parsed, "Cycle {} integrity check failed", cycle);

            goals = parsed; // Use parsed version for next cycle
        }

        let duration = start_time.elapsed();
        println!("Completed {} round-trip cycles in {:?}", CYCLES, duration);

        // Performance target: should complete within reasonable time
        assert!(
            duration.as_secs() < 60,
            "Performance test took too long: {:?}",
            duration
        );

        Ok(())
    }

    // =====================================
    // Test 10: Comprehensive Integration Test
    // =====================================

    #[test]
    fn test_comprehensive_round_trip_integration() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
            data_root: temp_dir.path().to_string_lossy().to_string(),
        };

        // Test data representing real-world usage
        let test_data = vec![
            // Day 1: Basic goals
            (
                1,
                "First day",
                vec![
                    (
                        "Complete project setup",
                        vec!["Create repo", "Setup CI/CD", "Write README"],
                    ),
                    (
                        "Start fitness routine",
                        vec!["Morning jog", "Buy gym membership", "Plan workout schedule"],
                    ),
                    (
                        "Family time",
                        vec!["Call parents", "Plan weekend", "Help with homework"],
                    ),
                ],
            ),
            // Day 2: More complex goals with unicode
            (
                2,
                "Unicode day 🌟",
                vec![
                    (
                        "项目进展 📈",
                        vec!["编写代码 💻", "测试功能 🧪", "部署系统 🚀"],
                    ),
                    (
                        "健康管理 💪",
                        vec!["晨跑 5公里 🏃", "健康饮食 🥗", "充足睡眠 😴"],
                    ),
                    (
                        "家庭时光 👨‍👩‍👧‍👦",
                        vec!["与父母通话 📞", "家庭聚餐 🍽️", "看电影 🎬"],
                    ),
                ],
            ),
            // Day 3: Edge cases
            (
                3,
                "Edge cases",
                vec![
                    (
                        "Goal with [brackets] and (parentheses)",
                        vec![
                            "Task with - [x] fake checkbox",
                            "Task with ## header",
                            "Normal task",
                        ],
                    ),
                    ("", vec!["", "Single char: X", "Symbols: © ® ™"]),
                    (
                        "Very long goal with lots of content",
                        vec![
                            "Short",
                            "Medium length task with details",
                            "Very long task with detailed description",
                        ],
                    ),
                ],
            ),
        ];

        let mut all_results = HashMap::new();

        // Create and test each day
        for (day, day_name, outcomes_data) in test_data {
            let date = NaiveDate::from_ymd_opt(2025, 1, day).unwrap();
            let mut goals = DailyGoals::new(date);
            goals.day_number = Some(day as u32);

            // Set up goals data
            let mut outcomes = [&mut goals.work, &mut goals.health, &mut goals.family];
            for (outcome, (goal_text, actions_data)) in
                outcomes.iter_mut().zip(outcomes_data.iter())
            {
                outcome.goal = if goal_text.is_empty() {
                    None
                } else {
                    Some(goal_text.to_string())
                };
                for (i, action_text) in actions_data.iter().enumerate() {
                    if i < 3 {
                        outcome.actions[i] = Action::from_markdown(
                            action_text.to_string(),
                            (day as usize + i) % 3 == 0,
                        );
                    }
                }
            }

            // Perform comprehensive round-trip testing
            // Step 1: Save to file
            write_goals_file(&goals, &config)?;

            // Step 2: Load from file
            let loaded = load_or_create_goals(date, &config)?;
            assert_eq!(goals, loaded, "Day {} file I/O failed", day);

            // Step 3: Convert to markdown
            let markdown = generate_markdown(&loaded);

            // Step 4: Parse from markdown
            let parsed = parse_markdown(&markdown)?;
            assert_eq!(loaded, parsed, "Day {} markdown round-trip failed", day);

            // Step 5: Save parsed version
            write_goals_file(&parsed, &config)?;

            // Step 6: Load again to ensure consistency
            let reloaded = load_or_create_goals(date, &config)?;
            assert_eq!(parsed, reloaded, "Day {} consistency check failed", day);

            // Step 7: Multiple markdown cycles
            let mut current = reloaded;
            for cycle in 1..=5 {
                let md = generate_markdown(&current);
                let p = parse_markdown(&md)?;
                assert_eq!(current, p, "Day {} cycle {} failed", day, cycle);
                current = p;
            }

            all_results.insert(day, current);
        }

        // Cross-day verification: ensure all days remain accessible and correct
        for (day, expected_goals) in all_results {
            let date = NaiveDate::from_ymd_opt(2025, 1, day).unwrap();
            let loaded = load_or_create_goals(date, &config)?;
            assert_eq!(
                expected_goals, loaded,
                "Cross-day verification failed for day {}",
                day
            );
        }

        // Final verification: ensure file system consistency
        let goals_dir = std::path::Path::new(&config.goals_dir);
        let mut file_count = 0;
        for entry in std::fs::read_dir(goals_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "md") {
                file_count += 1;

                // Verify each file is parseable
                let content = std::fs::read_to_string(entry.path())?;
                let parsed = parse_markdown(&content)?;

                // Verify round-trip integrity
                let regenerated = generate_markdown(&parsed);
                let reparsed = parse_markdown(&regenerated)?;
                assert_eq!(
                    parsed,
                    reparsed,
                    "File {} failed round-trip",
                    entry.path().display()
                );
            }
        }

        assert_eq!(file_count, 3, "Expected 3 goal files, found {}", file_count);

        Ok(())
    }
}

/// Utility functions for test data generation and validation
#[cfg(test)]
mod test_utils {
    use super::*;

    /// Generate test goals with specific patterns for validation
    pub fn create_test_goals_with_pattern(date: NaiveDate, pattern: &str) -> DailyGoals {
        let mut goals = DailyGoals::new(date);

        match pattern {
            "empty" => {
                // All defaults, nothing to change
            }
            "minimal" => {
                goals.work.actions[0].text = "X".to_string();
            }
            "unicode" => {
                goals.work.goal = Some("目标 🎯".to_string());
                goals.work.actions[0] = Action::from_markdown("任务 ✅".to_string(), true);
            }
            "special_chars" => {
                goals.work.goal = Some("Goal with [brackets] and (parentheses)".to_string());
                goals.work.actions[0] =
                    Action::from_markdown("Task with - [x] and ## symbols".to_string(), false);
            }
            "long_content" => {
                let long_text = "Long content ".repeat(100);
                goals.work.goal = Some(long_text.clone());
                goals.work.actions[0] = Action::from_markdown(long_text, true);
            }
            _ => {
                panic!("Unknown test pattern: {}", pattern);
            }
        }

        goals
    }

    /// Verify that two DailyGoals are identical in all aspects
    pub fn verify_goals_identical(goals1: &DailyGoals, goals2: &DailyGoals, context: &str) {
        assert_eq!(goals1.date, goals2.date, "{}: Date mismatch", context);
        assert_eq!(
            goals1.day_number, goals2.day_number,
            "{}: Day number mismatch",
            context
        );

        for (i, (outcome1, outcome2)) in goals1
            .outcomes()
            .iter()
            .zip(goals2.outcomes().iter())
            .enumerate()
        {
            assert_eq!(
                outcome1.goal, outcome2.goal,
                "{}: Outcome {} goal mismatch",
                context, i
            );

            for (j, (action1, action2)) in outcome1
                .actions
                .iter()
                .zip(outcome2.actions.iter())
                .enumerate()
            {
                assert_eq!(
                    action1.text, action2.text,
                    "{}: Outcome {} action {} text mismatch",
                    context, i, j
                );
                assert_eq!(
                    action1.completed, action2.completed,
                    "{}: Outcome {} action {} completion mismatch",
                    context, i, j
                );
            }
        }
    }

    /// Perform a complete round-trip test and return timing information
    pub fn timed_round_trip_test(
        goals: &DailyGoals,
        config: &Config,
    ) -> Result<(DailyGoals, std::time::Duration)> {
        use std::time::Instant;

        let start = Instant::now();

        // Save to file
        write_goals_file(goals, config)?;

        // Load from file
        let loaded = load_or_create_goals(goals.date, config)?;

        // Convert to markdown
        let markdown = generate_markdown(&loaded);

        // Parse from markdown
        let parsed = parse_markdown(&markdown)?;

        let duration = start.elapsed();

        // Verify integrity
        verify_goals_identical(goals, &loaded, "File I/O");
        verify_goals_identical(&loaded, &parsed, "Markdown conversion");

        Ok((parsed, duration))
    }
}
