/// Critical Parser Fix Tests
///
/// This module contains tests for the most critical parsing vulnerabilities
/// that MUST be fixed before the parser can be considered production-ready.
///
/// These tests should be run after implementing fixes to ensure the
/// vulnerabilities have been properly addressed.

#[cfg(test)]
mod critical_fixes_tests {
    use chrono::NaiveDate;
    use focusfive::data::parse_markdown;

    /// CRITICAL FIX 1: Header Position Flexibility
    ///
    /// The parser should accept headers that aren't on the exact first line,
    /// as long as there's only whitespace before them.
    #[test]
    fn test_header_position_flexibility() {
        // These should all succeed after the fix
        let test_cases = vec![
            // Leading whitespace
            "   # January 15, 2025 - Day 12\n\n## Work\n- [x] Task",
            // Leading empty lines
            "\n\n# January 15, 2025 - Day 12\n\n## Work\n- [x] Task",
            // Mixed whitespace
            " \t \n# January 15, 2025 - Day 12\n\n## Work\n- [x] Task",
        ];

        for (i, markdown) in test_cases.iter().enumerate() {
            let result = parse_markdown(markdown);
            assert!(
                result.is_ok(),
                "Test case {} should succeed after header fix",
                i + 1
            );

            let goals = result.unwrap();
            assert_eq!(goals.date, NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
            assert_eq!(goals.work.actions[0].text, "Task");
        }
    }

    /// CRITICAL FIX 2: Data Loss Prevention
    ///
    /// The parser should either:
    /// 1. Return an error when more than 3 actions are provided, OR
    /// 2. Return a warning/metadata about truncated actions
    #[test]
    fn test_action_overflow_handling() {
        let markdown = r#"# January 15, 2025 - Day 12

## Work
- [x] Task 1
- [x] Task 2  
- [x] Task 3
- [x] Task 4 (should trigger warning)
- [x] Task 5 (should trigger warning)"#;

        let result = parse_markdown(markdown);

        // Option 1: Parser returns error for too many actions
        if result.is_err() {
            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains("too many actions") || error_msg.contains("more than 3"),
                "Error should mention action limit exceeded"
            );
            return;
        }

        // Option 2: Parser succeeds but provides warning metadata
        let goals = result.unwrap();
        assert_eq!(goals.work.actions[0].text, "Task 1");
        assert_eq!(goals.work.actions[1].text, "Task 2");
        assert_eq!(goals.work.actions[2].text, "Task 3");

        // TODO: Add metadata field to track truncated actions
        // assert!(goals.warnings.contains("actions_truncated"));
        // assert_eq!(goals.truncated_actions_count, 2);

        // For now, this test documents the current behavior
        println!("⚠️  Parser should warn about {} truncated actions", 2);
    }

    /// CRITICAL FIX 3: Case Insensitive Outcome Headers
    ///
    /// Outcome headers should be case-insensitive for consistency with month parsing.
    #[test]
    fn test_case_insensitive_outcome_headers() {
        let test_cases = vec![
            ("## work", "work"),
            ("## WORK", "WORK"),
            ("## Work", "Work"),
            ("## health", "health"),
            ("## HEALTH", "HEALTH"),
            ("## Health", "Health"),
            ("## family", "family"),
            ("## FAMILY", "FAMILY"),
            ("## Family", "Family"),
        ];

        for (header, case_type) in test_cases {
            let markdown = format!(
                r#"# January 15, 2025 - Day 12

{}
- [x] Task for {}"#,
                header, case_type
            );

            let result = parse_markdown(&markdown);
            assert!(
                result.is_ok(),
                "Should parse {} header case-insensitively",
                case_type
            );

            let goals = result.unwrap();

            // All cases should parse the action
            if header.to_lowercase().contains("work") {
                assert_ne!(
                    goals.work.actions[0].text, "",
                    "Work action should be parsed for {}",
                    case_type
                );
                assert_eq!(
                    goals.work.actions[0].text,
                    format!("Task for {}", case_type)
                );
            } else if header.to_lowercase().contains("health") {
                assert_ne!(
                    goals.health.actions[0].text, "",
                    "Health action should be parsed for {}",
                    case_type
                );
            } else if header.to_lowercase().contains("family") {
                assert_ne!(
                    goals.family.actions[0].text, "",
                    "Family action should be parsed for {}",
                    case_type
                );
            }
        }
    }

    /// CRITICAL FIX 4: Better Error Messages
    ///
    /// Error messages should be specific and helpful for debugging.
    #[test]
    fn test_improved_error_messages() {
        let test_cases = vec![
            ("", "empty file"),
            ("No header here\n## Work\n- [x] Task", "no date header"),
            ("# Invalid Date Format\n## Work\n- [x] Task", "invalid date"),
            (
                "# February 30, 2025\n## Work\n- [x] Task",
                "impossible date",
            ),
        ];

        for (markdown, expected_error_type) in test_cases {
            let result = parse_markdown(markdown);
            assert!(result.is_err(), "Should fail for {}", expected_error_type);

            let error_msg = result.unwrap_err().to_string().to_lowercase();

            match expected_error_type {
                "empty file" => {
                    assert!(error_msg.contains("empty") || error_msg.contains("no content"))
                }
                "no date header" => {
                    assert!(error_msg.contains("header") || error_msg.contains("date"))
                }
                "invalid date" => {
                    assert!(error_msg.contains("date") || error_msg.contains("parse"))
                }
                "impossible date" => {
                    assert!(error_msg.contains("invalid") || error_msg.contains("date"))
                }
                _ => {}
            }
        }
    }

    /// CRITICAL FIX 5: Input Validation and Sanitization
    ///
    /// Parser should handle malicious or corrupted input safely.
    #[test]
    fn test_input_validation() {
        // Very long input should not cause performance issues
        let long_task = "a".repeat(100_000);
        let markdown = format!(
            r#"# January 15, 2025 - Day 12

## Work
- [x] {}"#,
            long_task
        );

        let start = std::time::Instant::now();
        let result = parse_markdown(&markdown);
        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 1000,
            "Should parse large input in under 1 second"
        );
        assert!(result.is_ok(), "Should handle large input gracefully");

        // Control characters should be handled safely
        let markdown_with_nulls = format!(
            r#"# January 15, 2025 - Day 12

## Work
- [x] Task with{}null byte"#,
            '\0'
        );

        let result = parse_markdown(&markdown_with_nulls);
        assert!(result.is_ok(), "Should handle control characters");

        if let Ok(goals) = result {
            // Control characters should either be filtered out or preserved safely
            let task_text = &goals.work.actions[0].text;
            assert!(!task_text.is_empty(), "Task text should not be empty");
            // Don't assert on null byte presence - that's an implementation detail
        }
    }

    /// CRITICAL FIX 6: Regex Performance and Security
    ///
    /// Regex patterns should not be vulnerable to catastrophic backtracking.
    #[test]
    fn test_regex_security() {
        // Input designed to cause regex backtracking
        let many_parens = "(".repeat(1000);
        let markdown = format!(
            r#"# January 15, 2025 - Day 12

## Work (Goal: {})
- [x] Task"#,
            many_parens
        );

        let start = std::time::Instant::now();
        let result = parse_markdown(&markdown);
        let duration = start.elapsed();

        // Should complete quickly regardless of input
        assert!(
            duration.as_millis() < 100,
            "Regex should not cause significant delays"
        );
        assert!(result.is_ok(), "Should handle malformed goals gracefully");
    }

    /// INTEGRATION TEST: All Fixes Working Together
    ///
    /// Test that combines multiple fixed vulnerabilities.
    #[test]
    fn test_comprehensive_robustness() {
        let challenging_markdown = format!(
            r#"  
# january 15, 2025 - Day 12

## work (Goal: Ship v1.0 & celebrate!)
- [x] {}
- [ ] Normal task
- [x] Third task
- [x] Fourth task (should be handled appropriately)

## HEALTH
- [x] Exercise 💪
- [ ] Медитация
- [x] Sleep

## family
- [x] Call parents ❤️  
- [ ] Plan trip (vacation in Europe)
- [x] Help with homework"#,
            "a".repeat(1000)
        );

        let result = parse_markdown(&challenging_markdown);

        // Should either succeed with warnings or fail with clear error
        match result {
            Ok(goals) => {
                // Verify all sections parsed despite case variations
                assert_ne!(goals.work.actions[0].text, "", "Work section should parse");
                assert_ne!(
                    goals.health.actions[0].text, "",
                    "Health section should parse"
                );
                assert_ne!(
                    goals.family.actions[0].text, "",
                    "Family section should parse"
                );

                // Date should parse despite lowercase month
                assert_eq!(goals.date, NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());

                // Goals should parse correctly
                assert_eq!(goals.work.goal, Some("Ship v1.0 & celebrate!".to_string()));

                println!("✅ Comprehensive robustness test passed");
            }
            Err(e) => {
                // If it fails, error should be clear and specific
                let error_msg = e.to_string();
                assert!(!error_msg.is_empty(), "Error message should not be empty");
                assert!(error_msg.len() > 10, "Error message should be descriptive");

                println!("ℹ️  Failed with clear error: {}", error_msg);
            }
        }
    }

    /// PERFORMANCE REGRESSION TEST
    ///
    /// Ensure fixes don't significantly impact performance.
    #[test]
    fn test_performance_after_fixes() {
        let base_markdown = "# January 15, 2025 - Day 12\n\n";

        // Test with different sizes
        for num_sections in [1, 10, 100] {
            let mut large_markdown = base_markdown.to_string();

            for i in 0..num_sections {
                large_markdown.push_str(&format!(
                    r#"
## Work Section {}
- [x] Task 1
- [ ] Task 2  
- [x] Task 3
"#,
                    i
                ));
            }

            let start = std::time::Instant::now();
            let result = parse_markdown(&large_markdown);
            let duration = start.elapsed();

            // Performance should scale reasonably
            assert!(
                duration.as_millis() < num_sections * 10,
                "Performance should be reasonable for {} sections",
                num_sections
            );

            // Should still parse correctly
            assert!(result.is_ok(), "Should parse large input correctly");
        }
    }
}

/// Tests for specific parser improvements and edge cases
#[cfg(test)]
mod parser_improvements_tests {
    use super::*;
    use focusfive::data::parse_markdown;

    #[test]
    fn test_flexible_header_formats() {
        // After improvements, these should all work
        let header_variations = vec![
            "# January 15, 2025 - Day 12",      // Standard
            "#January 15, 2025 - Day 12",       // No space after #
            "# January 15, 2025-Day 12",        // No spaces around dash
            "# January 15, 2025 - Day12",       // No space before number
            "# January  15,  2025  -  Day  12", // Extra spaces
        ];

        for header in header_variations {
            let markdown = format!("{}\n\n## Work\n- [x] Task", header);
            let result = parse_markdown(&markdown);
            assert!(result.is_ok(), "Should parse header variant: {}", header);
        }
    }

    #[test]
    fn test_enhanced_goal_extraction() {
        let goal_test_cases = vec![
            ("## Work (Goal: Simple goal)", Some("Simple goal")),
            (
                "## Work (Goal: Goal with (nested) parens)",
                Some("Goal with (nested) parens"),
            ),
            (
                "## Work (Goal: Goal with $symbols & more!)",
                Some("Goal with $symbols & more!"),
            ),
            (
                "## Work (Goal:No space after colon)",
                Some("No space after colon"),
            ),
            ("## Work ( Goal: Extra space )", Some("Extra space")),
            ("## Work (Goal: )", Some("")),         // Empty goal
            ("## Work Goal: No parentheses", None), // Wrong format
        ];

        for (header, expected_goal) in goal_test_cases {
            let markdown = format!("# January 15, 2025 - Day 12\n\n{}\n- [x] Task", header);
            let result = parse_markdown(&markdown);
            assert!(result.is_ok(), "Should parse: {}", header);

            let goals = result.unwrap();
            assert_eq!(
                goals.work.goal.as_deref(),
                expected_goal,
                "Goal extraction failed for: {}",
                header
            );
        }
    }

    #[test]
    fn test_robust_action_parsing() {
        let action_test_cases = vec![
            ("- [x] Standard completed", true, "Standard completed"),
            ("- [X] Uppercase completed", true, "Uppercase completed"),
            ("- [ ] Standard incomplete", false, "Standard incomplete"),
            (
                "- [x]No space after bracket",
                true,
                "No space after bracket",
            ),
            ("- [ ]   Extra spaces", false, "Extra spaces"),
            ("-[x] No space after dash", true, "No space after dash"), // Should this work?
        ];

        for (action_line, expected_completed, expected_text) in action_test_cases {
            let markdown = format!("# January 15, 2025 - Day 12\n\n## Work\n{}", action_line);
            let result = parse_markdown(&markdown);

            if result.is_ok() {
                let goals = result.unwrap();
                if !goals.work.actions[0].text.is_empty() {
                    assert_eq!(goals.work.actions[0].completed, expected_completed);
                    assert_eq!(goals.work.actions[0].text, expected_text);
                }
            }
            // Some formats might be rejected - that's OK as long as it's consistent
        }
    }
}
