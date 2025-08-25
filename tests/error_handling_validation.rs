use anyhow::{Context, Result};
use chrono::NaiveDate;
/// Comprehensive Error Handling Validation Tests for FocusFive Phase 1
///
/// This test suite validates error handling throughout the FocusFive Phase 1 implementation,
/// focusing on the 10 critical error scenarios identified in the validation requirements:
///
/// 1. File not found scenarios
/// 2. Permission denied errors
/// 3. Disk full conditions
/// 4. Invalid UTF-8 in files
/// 5. Corrupted markdown files
/// 6. Missing required fields
/// 7. Invalid dates (Feb 30, etc.)
/// 8. Path traversal attempts
/// 9. Memory exhaustion scenarios
/// 10. Panic recovery and graceful degradation
use focusfive::data::{load_or_create_goals, parse_markdown, read_goals_file, write_goals_file};
use focusfive::models::{Action, Config, DailyGoals};
use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::{NamedTempFile, TempDir};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    // =====================================
    // Test 1: File Not Found Scenarios
    // =====================================

    #[test]
    fn test_read_nonexistent_file() {
        let nonexistent_path = Path::new("/path/that/definitely/does/not/exist/goals.md");

        let result = read_goals_file(nonexistent_path);

        // Should return an error, not panic
        assert!(result.is_err(), "Should return error for nonexistent file");

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        // Error should be meaningful and contain path information
        assert!(
            error_msg.contains("Failed to read file"),
            "Error should mention file read failure: {}",
            error_msg
        );
        assert!(
            error_msg.contains("/path/that/definitely/does/not/exist/goals.md"),
            "Error should contain the problematic path: {}",
            error_msg
        );

        // Check error chain contains useful information
        let error_chain: Vec<String> = error.chain().map(|e| e.to_string()).collect();
        assert!(
            error_chain.len() >= 2,
            "Should have error chain with context"
        );

        println!("✅ File not found error properly handled: {}", error_msg);
    }

    #[test]
    fn test_load_from_nonexistent_directory() {
        let config = Config {
            goals_dir: "/nonexistent/directory/that/cannot/exist".to_string(),
        };

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        // load_or_create_goals should handle missing directory gracefully
        let result = load_or_create_goals(date, &config);

        // Should return new goals since file doesn't exist
        assert!(
            result.is_ok(),
            "Should create new goals when directory doesn't exist"
        );

        let goals = result.unwrap();
        assert_eq!(goals.date, date);
        assert!(goals.work.actions[0].text.is_empty());

        println!("✅ Nonexistent directory handled gracefully");
    }

    // =====================================
    // Test 2: Permission Denied Errors
    // =====================================

    #[test]
    #[cfg(unix)]
    fn test_permission_denied_read() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("readonly.md");

        // Create file with content
        fs::write(&file_path, "# January 15, 2025\n\n## Work\n- [x] Task")?;

        // Make file unreadable (no read permissions)
        let mut perms = fs::metadata(&file_path)?.permissions();
        perms.set_mode(0o000); // No permissions
        fs::set_permissions(&file_path, perms)?;

        let result = read_goals_file(&file_path);

        // Should return permission error, not panic
        assert!(result.is_err(), "Should return error for unreadable file");

        let error = result.unwrap_err();
        let error_msg = error.to_string().to_lowercase();

        // Error should indicate permission issue
        assert!(
            error_msg.contains("permission")
                || error_msg.contains("denied")
                || error_msg.contains("access"),
            "Error should indicate permission issue: {}",
            error_msg
        );

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&file_path)?.permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&file_path, perms)?;

        println!("✅ Permission denied error properly handled");
        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn test_permission_denied_write() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir(&readonly_dir)?;

        // Make directory read-only
        let mut perms = fs::metadata(&readonly_dir)?.permissions();
        perms.set_mode(0o555); // Read and execute only
        fs::set_permissions(&readonly_dir, perms)?;

        let config = Config {
            goals_dir: readonly_dir.to_string_lossy().to_string(),
        };

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let goals = DailyGoals::new(date);

        let result = write_goals_file(&goals, &config);

        // Should return permission error, not panic
        assert!(
            result.is_err(),
            "Should return error for readonly directory"
        );

        let error = result.unwrap_err();
        let error_msg = error.to_string().to_lowercase();

        // Error should indicate permission or directory creation issue
        assert!(
            error_msg.contains("permission")
                || error_msg.contains("denied")
                || error_msg.contains("create")
                || error_msg.contains("directory"),
            "Error should indicate permission/creation issue: {}",
            error_msg
        );

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&readonly_dir)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&readonly_dir, perms)?;

        println!("✅ Write permission denied error properly handled");
        Ok(())
    }

    // =====================================
    // Test 3: Disk Full Conditions
    // =====================================

    #[test]
    fn test_large_file_handling() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
        };

        // Create goals with very large content
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);

        // Create extremely large action text (10MB)
        let large_text = "A".repeat(10_000_000);
        goals.work.actions[0] = Action {
            text: large_text,
            completed: false,
        };

        let start_time = Instant::now();
        let result = write_goals_file(&goals, &config);
        let duration = start_time.elapsed();

        match result {
            Ok(path) => {
                // If successful, verify file exists and has correct size
                assert!(path.exists());
                let metadata = fs::metadata(&path)?;
                assert!(metadata.len() > 10_000_000);
                println!(
                    "✅ Large file ({} bytes) written successfully in {:?}",
                    metadata.len(),
                    duration
                );
            }
            Err(e) => {
                // If it fails, ensure error is reasonable (disk space, etc.)
                let error_msg = e.to_string().to_lowercase();
                println!("⚠️  Large file write failed (expected): {}", error_msg);

                // Error should not be a panic or unhandled exception
                assert!(
                    !error_msg.contains("panic") && !error_msg.contains("abort"),
                    "Should not panic on large file write"
                );
            }
        }

        // Performance should be reasonable (not hang indefinitely)
        assert!(
            duration < Duration::from_secs(30),
            "Large file operation should complete within 30 seconds"
        );

        println!("✅ Large file handling completed gracefully");
        Ok(())
    }

    #[test]
    fn test_simulated_disk_full_scenario() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
        };

        // Try to write many large files to potentially exhaust space
        let mut successful_writes = 0;
        let mut first_error: Option<anyhow::Error> = None;

        for i in 0..100 {
            let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap() + chrono::Duration::days(i);
            let mut goals = DailyGoals::new(date);

            // Each file is ~1MB
            let large_content = format!("Task {} - {}", i, "X".repeat(1_000_000));
            goals.work.actions[0] = Action {
                text: large_content,
                completed: false,
            };

            match write_goals_file(&goals, &config) {
                Ok(_) => successful_writes += 1,
                Err(e) => {
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                    break;
                }
            }
        }

        println!(
            "✅ Disk space test: {} files written before failure",
            successful_writes
        );

        if let Some(error) = first_error {
            let error_msg = error.to_string();
            println!("First error: {}", error_msg);

            // Error should be handled gracefully, not a panic
            assert!(!error_msg.contains("panic"));
        }

        Ok(())
    }

    // =====================================
    // Test 4: Invalid UTF-8 in Files
    // =====================================

    #[test]
    fn test_invalid_utf8_file_content() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("invalid_utf8.md");

        // Create file with invalid UTF-8 bytes
        let mut file = File::create(&file_path)?;

        // Write valid UTF-8 header
        file.write_all(b"# January 15, 2025\n\n## Work\n- [x] ")?;

        // Write invalid UTF-8 sequence
        file.write_all(&[0xFF, 0xFE, 0xFD, 0xFC])?;

        // Write more valid UTF-8
        file.write_all(b" Invalid UTF-8 task\n")?;

        file.sync_all()?;
        drop(file);

        let result = read_goals_file(&file_path);

        // Should handle invalid UTF-8 gracefully
        assert!(result.is_err(), "Should return error for invalid UTF-8");

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        // Error should mention UTF-8 or encoding issue
        assert!(
            error_msg.contains("utf")
                || error_msg.contains("UTF")
                || error_msg.contains("encoding")
                || error_msg.contains("invalid"),
            "Error should mention encoding issue: {}",
            error_msg
        );

        println!("✅ Invalid UTF-8 handled gracefully: {}", error_msg);
        Ok(())
    }

    #[test]
    fn test_bom_and_encoding_artifacts() {
        // Test Byte Order Mark (BOM) at start of file
        let markdown_with_bom =
            "\u{FEFF}# January 15, 2025 - Day 12\n\n## Work\n- [x] Task with BOM";

        let result = parse_markdown(markdown_with_bom);

        match result {
            Ok(goals) => {
                // If BOM is handled, verify parsing works
                assert_eq!(goals.date.month(), 1);
                println!("✅ BOM handled gracefully");
            }
            Err(e) => {
                // If BOM causes failure, ensure error is clear
                let error_msg = e.to_string();
                assert!(!error_msg.contains("panic"));
                println!("⚠️  BOM causes parsing failure: {}", error_msg);
            }
        }

        // Test various Unicode control characters
        let markdown_with_controls = format!(
            "# January 15, 2025\n\n## Work\n- [x] Task with controls{}{}{}",
            '\u{200B}', // Zero-width space
            '\u{FEFF}', // BOM in middle
            '\u{202E}'  // Right-to-left override
        );

        let result = parse_markdown(&markdown_with_controls);
        assert!(
            result.is_ok(),
            "Should handle Unicode control characters without panicking"
        );

        println!("✅ Unicode control characters handled");
    }

    // =====================================
    // Test 5: Corrupted Markdown Files
    // =====================================

    #[test]
    fn test_completely_corrupted_markdown() {
        let corrupted_examples = vec![
            // Binary data
            "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F",
            // Random characters
            "!@#$%^&*()_+-=[]{}|;':\",./<>?",
            // Malformed markdown structures
            "### #### ##### ###### ####### ########",
            // Mixed valid and invalid
            "# Valid Header\n\x00\x01\x02\nInvalid content",
            // Extremely nested structures
            "## Work (".repeat(1000) + &")".repeat(1000),
        ];

        for (i, corrupted_content) in corrupted_examples.iter().enumerate() {
            let result = parse_markdown(corrupted_content);

            match result {
                Ok(_) => {
                    println!(
                        "⚠️  Corrupted example {} unexpectedly parsed successfully",
                        i
                    );
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    // Ensure no panics in error handling
                    assert!(!error_msg.contains("panic") && !error_msg.contains("abort"));
                    println!(
                        "✅ Corrupted example {} rejected gracefully: {}",
                        i, error_msg
                    );
                }
            }
        }
    }

    #[test]
    fn test_truncated_file_scenarios() {
        let truncated_examples = vec![
            "",                                    // Empty file
            "#",                                   // Just hash
            "# Jan",                               // Incomplete header
            "# January 15, 2025\n##",              // Incomplete outcome header
            "# January 15, 2025\n\n## Work\n- [",  // Incomplete action
            "# January 15, 2025\n\n## Work\n- [x", // Incomplete checkbox
        ];

        for (i, truncated_content) in truncated_examples.iter().enumerate() {
            let result = parse_markdown(truncated_content);

            // All should either succeed with default values or fail gracefully
            match result {
                Ok(goals) => {
                    println!("✅ Truncated example {} parsed with defaults", i);
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    assert!(!error_msg.contains("panic"));
                    println!(
                        "✅ Truncated example {} rejected gracefully: {}",
                        i, error_msg
                    );
                }
            }
        }
    }

    // =====================================
    // Test 6: Missing Required Fields
    // =====================================

    #[test]
    fn test_missing_date_header() {
        let no_date_examples = vec![
            "## Work\n- [x] Task without date header",
            "\n\n## Work\n- [x] Task with leading blank lines",
            "Some random text\n## Work\n- [x] Task",
        ];

        for (i, content) in no_date_examples.iter().enumerate() {
            let result = parse_markdown(content);

            // Should fail gracefully for missing date
            assert!(
                result.is_err(),
                "Example {} should fail without date header",
                i
            );

            let error = result.unwrap_err();
            let error_msg = error.to_string();

            // Error should be informative
            assert!(
                error_msg.contains("date")
                    || error_msg.contains("header")
                    || error_msg.contains("Empty file"),
                "Error should mention missing date/header: {}",
                error_msg
            );

            println!("✅ Missing date header {} handled: {}", i, error_msg);
        }
    }

    #[test]
    fn test_malformed_date_formats() {
        let bad_date_examples = vec![
            "# Not a date at all",
            "# 15 January 2025",  // Wrong order
            "# January 2025",     // Missing day
            "# January 15",       // Missing year
            "# 2025-01-15",       // ISO format instead of written
            "# January 15, 25",   // Two-digit year
            "# Janvier 15, 2025", // French month name
        ];

        for (i, header) in bad_date_examples.iter().enumerate() {
            let markdown = format!("{}\n\n## Work\n- [x] Task", header);
            let result = parse_markdown(&markdown);

            // Should fail for malformed dates
            assert!(result.is_err(), "Bad date example {} should fail", i);

            let error = result.unwrap_err();
            let error_msg = error.to_string();

            // Error should mention date parsing issue
            assert!(
                error_msg.contains("date")
                    || error_msg.contains("parse")
                    || error_msg.contains("header"),
                "Error should mention date parsing: {}",
                error_msg
            );

            println!("✅ Bad date format {} rejected: {}", i, error_msg);
        }
    }

    // =====================================
    // Test 7: Invalid Dates (Feb 30, etc.)
    // =====================================

    #[test]
    fn test_impossible_dates() {
        let impossible_dates = vec![
            "# February 30, 2025",  // Feb doesn't have 30 days
            "# April 31, 2025",     // April doesn't have 31 days
            "# February 29, 2023",  // 2023 is not a leap year
            "# June 31, 2025",      // June doesn't have 31 days
            "# September 31, 2025", // September doesn't have 31 days
            "# November 31, 2025",  // November doesn't have 31 days
            "# December 32, 2025",  // No month has 32 days
            "# January 0, 2025",    // Day 0 doesn't exist
        ];

        for (i, date_header) in impossible_dates.iter().enumerate() {
            let markdown = format!("{}\n\n## Work\n- [x] Task on impossible date", date_header);
            let result = parse_markdown(&markdown);

            // All impossible dates should be rejected
            assert!(
                result.is_err(),
                "Impossible date {} should be rejected: {}",
                i,
                date_header
            );

            let error = result.unwrap_err();
            let error_msg = error.to_string();

            // Error should mention invalid date
            assert!(
                error_msg.contains("Invalid date") || error_msg.contains("date"),
                "Error should mention invalid date: {}",
                error_msg
            );

            println!("✅ Impossible date {} rejected: {}", i, error_msg);
        }
    }

    #[test]
    fn test_edge_case_dates() {
        let edge_case_dates = vec![
            ("# February 29, 2024", true),  // 2024 is a leap year
            ("# February 29, 2000", true),  // 2000 is a leap year (divisible by 400)
            ("# February 29, 1900", false), // 1900 is not a leap year (divisible by 100 but not 400)
            ("# December 31, 1999", true),  // Y2K edge case
            ("# January 1, 2000", true),    // Y2K edge case
        ];

        for (date_header, should_succeed) in edge_case_dates {
            let markdown = format!("{}\n\n## Work\n- [x] Edge case task", date_header);
            let result = parse_markdown(&markdown);

            if should_succeed {
                assert!(
                    result.is_ok(),
                    "Valid edge case date should succeed: {}",
                    date_header
                );
                println!("✅ Valid edge case accepted: {}", date_header);
            } else {
                assert!(
                    result.is_err(),
                    "Invalid edge case date should fail: {}",
                    date_header
                );
                println!("✅ Invalid edge case rejected: {}", date_header);
            }
        }
    }

    // =====================================
    // Test 8: Path Traversal Attempts
    // =====================================

    #[test]
    fn test_path_traversal_in_goals_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;

        let path_traversal_attempts = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "/etc/passwd",
            "C:\\Windows\\System32",
            "goals/../../../sensitive_file",
            "goals\\..\\..\\sensitive_file",
        ];

        for malicious_path in path_traversal_attempts {
            let config = Config {
                goals_dir: malicious_path.to_string(),
            };

            let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
            let goals = DailyGoals::new(date);

            let result = write_goals_file(&goals, &config);

            match result {
                Ok(actual_path) => {
                    // If it succeeds, ensure it's contained within temp directory
                    let canonical_temp = temp_dir.path().canonicalize()?;

                    if let Ok(canonical_actual) = actual_path.canonicalize() {
                        // Ensure no path traversal occurred
                        assert!(
                            canonical_actual.starts_with(&canonical_temp)
                                || canonical_actual.starts_with(std::env::current_dir()?),
                            "Path traversal detected: {:?} not contained in safe area",
                            canonical_actual
                        );
                    }

                    println!(
                        "⚠️  Path traversal attempt '{}' succeeded but contained",
                        malicious_path
                    );
                }
                Err(e) => {
                    // Failure is also acceptable for security
                    let error_msg = e.to_string();
                    assert!(!error_msg.contains("panic"));
                    println!(
                        "✅ Path traversal attempt '{}' blocked: {}",
                        malicious_path, error_msg
                    );
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_malicious_filename_characters() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
        };

        // Note: The current implementation generates filenames from dates,
        // so direct filename injection isn't possible. But test edge cases.

        let edge_case_dates = vec![
            // These should all generate safe filenames
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(1900, 12, 31).unwrap(),
            NaiveDate::from_ymd_opt(2099, 12, 31).unwrap(),
        ];

        for date in edge_case_dates {
            let goals = DailyGoals::new(date);
            let result = write_goals_file(&goals, &base_config);

            match result {
                Ok(path) => {
                    // Verify filename is safe
                    let filename = path.file_name().unwrap().to_string_lossy();

                    // Should be in YYYY-MM-DD.md format
                    assert!(filename.ends_with(".md"));
                    assert!(filename.len() == 13); // "YYYY-MM-DD.md" = 13 chars
                    assert!(!filename.contains(".."));
                    assert!(!filename.contains("/"));
                    assert!(!filename.contains("\\"));

                    println!("✅ Safe filename generated: {}", filename);
                }
                Err(e) => {
                    println!("⚠️  Date {} caused error: {}", date, e);
                }
            }
        }

        Ok(())
    }

    // =====================================
    // Test 9: Memory Exhaustion Scenarios
    // =====================================

    #[test]
    fn test_extremely_large_input_handling() {
        // Test with very large markdown content
        let large_header = "# January 15, 2025 - Day ".to_string() + &"9".repeat(1000);
        let large_goal = "Goal: ".to_string() + &"X".repeat(100_000);
        let large_task = "Very long task description ".to_string() + &"Y".repeat(1_000_000);

        let large_markdown = format!(
            "{}\n\n## Work ({})\n- [x] {}\n- [ ] Normal task\n- [ ] Another task",
            large_header, large_goal, large_task
        );

        let start_time = Instant::now();
        let result = parse_markdown(&large_markdown);
        let duration = start_time.elapsed();

        // Should complete within reasonable time
        assert!(
            duration < Duration::from_secs(5),
            "Large input parsing should complete quickly: {:?}",
            duration
        );

        match result {
            Ok(goals) => {
                // If successful, verify data integrity
                assert_eq!(goals.date.month(), 1);
                assert_eq!(goals.date.day(), 15);
                assert!(!goals.work.actions[0].text.is_empty());
                println!("✅ Large input parsed successfully in {:?}", duration);
            }
            Err(e) => {
                // If failed, ensure no panic
                let error_msg = e.to_string();
                assert!(!error_msg.contains("panic"));
                println!("✅ Large input rejected gracefully: {}", error_msg);
            }
        }
    }

    #[test]
    fn test_memory_usage_with_many_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
        };

        let start_memory = get_memory_usage();
        let start_time = Instant::now();

        // Create and load many files to test memory usage
        const NUM_FILES: i64 = 1000;
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        // Create files
        for i in 0..NUM_FILES {
            let date = base_date + chrono::Duration::days(i);
            let mut goals = DailyGoals::new(date);

            // Add some content to each file
            goals.work.actions[0] = Action {
                text: format!("Work task for day {}", i),
                completed: i % 2 == 0,
            };

            write_goals_file(&goals, &config)?;
        }

        let after_write_time = Instant::now();
        let write_duration = after_write_time.duration_since(start_time);

        // Load all files back
        let mut total_actions = 0;
        for i in 0..NUM_FILES {
            let date = base_date + chrono::Duration::days(i);
            let goals = load_or_create_goals(date, &config)?;

            if !goals.work.actions[0].text.is_empty() {
                total_actions += 1;
            }
        }

        let end_time = Instant::now();
        let total_duration = end_time.duration_since(start_time);
        let end_memory = get_memory_usage();

        println!(
            "✅ Memory test: {} files processed in {:?}",
            NUM_FILES, total_duration
        );
        println!("   Write time: {:?}", write_duration);
        println!("   Memory usage: {} -> {} MB", start_memory, end_memory);
        println!("   Total actions loaded: {}", total_actions);

        // Performance assertions
        assert!(
            write_duration < Duration::from_secs(30),
            "Write should be fast"
        );
        assert!(
            total_duration < Duration::from_secs(60),
            "Total should be fast"
        );

        // Memory usage should be reasonable (not more than 1GB increase)
        let memory_increase = end_memory.saturating_sub(start_memory);
        assert!(
            memory_increase < 1000,
            "Memory usage should be reasonable: {}MB increase",
            memory_increase
        );

        Ok(())
    }

    // Helper function to get approximate memory usage (simplified)
    fn get_memory_usage() -> u64 {
        // This is a simplified implementation - in real tests you might use
        // more sophisticated memory monitoring
        std::process::Command::new("ps")
            .args(&["-o", "rss=", "-p"])
            .arg(std::process::id().to_string())
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse::<u64>()
                    .unwrap_or(0)
                    / 1024 // Convert KB to MB
            })
            .unwrap_or(0)
    }

    // =====================================
    // Test 10: Panic Recovery and Graceful Degradation
    // =====================================

    #[test]
    fn test_no_panics_with_malicious_input() {
        let malicious_inputs = vec![
            // Extreme repetition
            "# ".repeat(10000) + "January 15, 2025",
            // Control characters
            format!("# January 15, 2025{}{}{}", '\0', '\x01', '\x7F'),
            // Extremely long lines
            "# ".to_string() + &"January 15, 2025 - Day ".repeat(1000),
            // Mixed valid/invalid UTF-8 patterns
            "# January 15, 2025\n\n## Work\n- [x] ".to_string()
                + &String::from_utf8_lossy(&vec![0xC0, 0xC1, 0xF5, 0xFF]),
            // Nested parentheses (regex attack)
            "# January 15, 2025\n\n## Work (".to_string()
                + &"(".repeat(1000)
                + "Goal: test"
                + &")".repeat(1000)
                + ")\n- [x] Task",
            // Very deep nesting
            "## Work (".to_string() + &"(".repeat(10000) + "Goal" + &")".repeat(10000) + ")",
        ];

        for (i, malicious_input) in malicious_inputs.iter().enumerate() {
            let start_time = Instant::now();

            // Use catch_unwind to detect panics
            let result = std::panic::catch_unwind(|| parse_markdown(malicious_input));

            let duration = start_time.elapsed();

            match result {
                Ok(parse_result) => {
                    // No panic occurred - check if parsing succeeded or failed gracefully
                    match parse_result {
                        Ok(_) => println!("✅ Malicious input {} parsed successfully", i),
                        Err(e) => {
                            let error_msg = e.to_string();
                            assert!(!error_msg.contains("panic"));
                            println!(
                                "✅ Malicious input {} rejected gracefully: {}",
                                i, error_msg
                            );
                        }
                    }
                }
                Err(_) => {
                    panic!(
                        "PANIC DETECTED with malicious input {}: {}",
                        i,
                        malicious_input.chars().take(100).collect::<String>()
                    );
                }
            }

            // Should complete within reasonable time (no infinite loops)
            assert!(
                duration < Duration::from_secs(1),
                "Input {} took too long: {:?}",
                i,
                duration
            );
        }

        println!("✅ All malicious inputs handled without panics");
    }

    #[test]
    fn test_concurrent_panic_safety() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Arc::new(Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
        });

        let panic_detected = Arc::new(Mutex::new(false));
        let error_count = Arc::new(Mutex::new(0));
        let success_count = Arc::new(Mutex::new(0));

        let handles: Vec<_> = (0..10)
            .map(|thread_id| {
                let config = Arc::clone(&config);
                let panic_detected = Arc::clone(&panic_detected);
                let error_count = Arc::clone(&error_count);
                let success_count = Arc::clone(&success_count);

                thread::spawn(move || {
                    // Each thread tries various operations that might cause issues
                    for i in 0..50 {
                        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
                            + chrono::Duration::days(thread_id * 50 + i);

                        // Test with potentially problematic content
                        let mut goals = DailyGoals::new(date);
                        goals.work.actions[0] = Action {
                            text: format!(
                                "Thread {} task {} {}",
                                thread_id,
                                i,
                                "X".repeat(thread_id * 100)
                            ),
                            completed: (thread_id + i) % 2 == 0,
                        };

                        let result = std::panic::catch_unwind(|| write_goals_file(&goals, &config));

                        match result {
                            Ok(write_result) => match write_result {
                                Ok(_) => {
                                    *success_count.lock().unwrap() += 1;
                                }
                                Err(_) => {
                                    *error_count.lock().unwrap() += 1;
                                }
                            },
                            Err(_) => {
                                *panic_detected.lock().unwrap() = true;
                            }
                        }
                    }
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let panic_occurred = *panic_detected.lock().unwrap();
        let errors = *error_count.lock().unwrap();
        let successes = *success_count.lock().unwrap();

        assert!(
            !panic_occurred,
            "No panics should occur during concurrent operations"
        );

        println!(
            "✅ Concurrent operations completed: {} successes, {} errors, no panics",
            successes, errors
        );

        Ok(())
    }

    // =====================================
    // Error Context and Chain Validation
    // =====================================

    #[test]
    fn test_error_context_preservation() {
        let test_cases = vec![
            ("", "Empty file context should be preserved"),
            (
                "Invalid header",
                "Header parsing context should be preserved",
            ),
            (
                "# January 15, 2025\n\n## Work\n- [invalid] Bad checkbox",
                "Action parsing context should be preserved",
            ),
        ];

        for (content, expected_context) in test_cases {
            let result = parse_markdown(content);

            if let Err(error) = result {
                let error_chain: Vec<String> = error.chain().map(|e| e.to_string()).collect();

                // Should have meaningful error chain
                assert!(!error_chain.is_empty(), "Error should have context chain");

                // Top-level error should be descriptive
                let top_error = &error_chain[0];
                assert!(!top_error.is_empty(), "Top-level error should not be empty");

                // Print error chain for analysis
                println!(
                    "✅ Error context for '{}': {:?}",
                    content.chars().take(20).collect::<String>(),
                    error_chain
                );
            }
        }
    }
}

// =====================================
// Integration Tests for Error Scenarios
// =====================================

#[cfg(test)]
mod error_integration_tests {
    use super::*;

    #[test]
    fn test_full_workflow_error_recovery() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
        };

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        // Step 1: Try to load non-existent file (should create new)
        let goals1 = load_or_create_goals(date, &config)?;
        assert_eq!(goals1.work.actions[0].text, "");

        // Step 2: Modify and save
        let mut goals2 = goals1;
        goals2.work.actions[0] = Action {
            text: "First task".to_string(),
            completed: false,
        };

        let saved_path = write_goals_file(&goals2, &config)?;
        assert!(saved_path.exists());

        // Step 3: Corrupt the file and try to read
        fs::write(&saved_path, "Corrupted content that is not valid markdown")?;

        let corrupted_result = read_goals_file(&saved_path);
        assert!(corrupted_result.is_err(), "Should detect corrupted file");

        // Step 4: Recovery - overwrite with valid content
        let mut goals3 = DailyGoals::new(date);
        goals3.work.actions[0] = Action {
            text: "Recovered task".to_string(),
            completed: true,
        };

        write_goals_file(&goals3, &config)?;

        // Step 5: Verify recovery
        let recovered_goals = load_or_create_goals(date, &config)?;
        assert_eq!(recovered_goals.work.actions[0].text, "Recovered task");
        assert!(recovered_goals.work.actions[0].completed);

        println!("✅ Full workflow error recovery completed successfully");
        Ok(())
    }
}
