use crate::models::{Action, Config, DailyGoals, Outcome, OutcomeType};
use anyhow::{Context, Result};
use chrono::NaiveDate;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Thread-safe file locks for atomic operations
lazy_static::lazy_static! {
    static ref FILE_LOCKS: Mutex<HashMap<PathBuf, Arc<Mutex<()>>>> = 
        Mutex::new(HashMap::new());
}

/// Parse a markdown file into DailyGoals
pub fn parse_markdown(content: &str) -> Result<DailyGoals> {
    let lines: Vec<&str> = content.lines().collect();
    
    // Parse the date from the header
    let date = parse_date_header(lines.get(0).context("Empty file")?)?;
    
    let mut goals = DailyGoals::new(date);
    let mut current_outcome: Option<&mut Outcome> = None;
    let mut action_index = 0;
    
    // Parse line by line
    for line in lines.iter().skip(1) {
        let line = line.trim();
        
        // Skip empty lines
        if line.is_empty() {
            continue;
        }
        
        // Check for outcome headers
        if line.starts_with("## Work") {
            current_outcome = Some(&mut goals.work);
            goals.work.goal = extract_goal_from_header(line);
            action_index = 0;
        } else if line.starts_with("## Health") {
            current_outcome = Some(&mut goals.health);
            goals.health.goal = extract_goal_from_header(line);
            action_index = 0;
        } else if line.starts_with("## Family") {
            current_outcome = Some(&mut goals.family);
            goals.family.goal = extract_goal_from_header(line);
            action_index = 0;
        } else if line.starts_with("- [") {
            // Parse action
            if let Some(outcome) = current_outcome.as_mut() {
                if action_index < 3 {
                    let (completed, text) = parse_action_line(line)?;
                    outcome.actions[action_index] = Action {
                        text,
                        completed,
                    };
                    action_index += 1;
                }
            }
        }
    }
    
    // Extract day number if present
    if let Some(day_num) = extract_day_number(lines.get(0).unwrap_or(&"")) {
        goals.day_number = Some(day_num);
    }
    
    Ok(goals)
}

/// Parse the date from the header line
fn parse_date_header(header: &str) -> Result<NaiveDate> {
    // Pattern: # Month DD, YYYY - Day N
    let re = Regex::new(r"#\s*(\w+)\s+(\d{1,2}),\s*(\d{4})")?;
    
    if let Some(caps) = re.captures(header) {
        let month_str = &caps[1];
        let day: u32 = caps[2].parse()?;
        let year: i32 = caps[3].parse()?;
        
        let month = match month_str.to_lowercase().as_str() {
            "january" | "jan" => 1,
            "february" | "feb" => 2,
            "march" | "mar" => 3,
            "april" | "apr" => 4,
            "may" => 5,
            "june" | "jun" => 6,
            "july" | "jul" => 7,
            "august" | "aug" => 8,
            "september" | "sep" => 9,
            "october" | "oct" => 10,
            "november" | "nov" => 11,
            "december" | "dec" => 12,
            _ => anyhow::bail!("Invalid month: {}", month_str),
        };
        
        NaiveDate::from_ymd_opt(year, month, day)
            .context("Invalid date")
    } else {
        anyhow::bail!("Could not parse date from header: {}", header)
    }
}

/// Extract day number from header if present
fn extract_day_number(header: &str) -> Option<u32> {
    let re = Regex::new(r"Day\s+(\d+)").ok()?;
    re.captures(header)
        .and_then(|caps| caps[1].parse().ok())
}

/// Extract goal description from outcome header
fn extract_goal_from_header(header: &str) -> Option<String> {
    // Pattern: ## Outcome (Goal: description)
    let re = Regex::new(r"\(Goal:\s*([^)]+)\)").ok()?;
    re.captures(header)
        .map(|caps| caps[1].to_string())
}

/// Parse an action line into completion status and text
fn parse_action_line(line: &str) -> Result<(bool, String)> {
    if line.starts_with("- [x]") || line.starts_with("- [X]") {
        Ok((true, line[5..].trim().to_string()))
    } else if line.starts_with("- [ ]") {
        Ok((false, line[5..].trim().to_string()))
    } else {
        anyhow::bail!("Invalid action line format: {}", line)
    }
}

/// Validate path for security and safety
fn validate_path(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();
    
    // Check for null bytes
    if path_str.contains('\0') {
        anyhow::bail!("Path contains null byte");
    }
    
    // Check for control characters that could cause issues
    if path_str.chars().any(|c| c.is_control() && c != '\t') {
        anyhow::bail!("Path contains invalid control characters");
    }
    
    // Check path length (conservative limit)
    if path_str.len() > 255 {
        anyhow::bail!("Path too long: {} characters", path_str.len());
    }
    
    // Ensure we're not trying to write outside allowed areas
    if path_str.contains("..") {
        anyhow::bail!("Path traversal detected");
    }
    
    Ok(())
}

/// Generate a unique temporary file name
fn generate_temp_path(target_path: &Path) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    
    let thread_id = std::thread::current().id();
    let process_id = std::process::id();
    
    target_path.with_extension(&format!("tmp.{}.{:?}.{}", process_id, thread_id, timestamp))
}

/// Clean up old temporary files
fn cleanup_old_temp_files(directory: &Path) -> Result<()> {
    if !directory.exists() {
        return Ok(());
    }
    
    let one_hour_ago = SystemTime::now() - Duration::from_secs(3600);
    
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        
        // Check if it's a temp file
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            if filename_str.ends_with(".tmp") || filename_str.contains(".tmp.") {
                // Check if it's old enough to clean up
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if modified < one_hour_ago {
                            let _ = fs::remove_file(&path); // Ignore errors for cleanup
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Thread-safe atomic write with proper error handling and cleanup
fn atomic_write_safe(path: &Path, content: &[u8]) -> Result<()> {
    // Validate path first
    validate_path(path)?;
    
    let canonical_path = path.to_path_buf();
    
    // Get or create file-specific lock
    let file_lock = {
        let mut locks = FILE_LOCKS.lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire file locks mutex"))?;
        locks.entry(canonical_path.clone())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    };
    
    // Acquire file-specific lock
    let _guard = file_lock.lock()
        .map_err(|_| anyhow::anyhow!("Failed to acquire file lock"))?;
    
    // Clean up old temp files in the directory
    if let Some(parent) = path.parent() {
        cleanup_old_temp_files(parent)?;
    }
    
    // Generate unique temp file path
    let temp_path = generate_temp_path(path);
    
    // Ensure we clean up temp file on any error
    let cleanup_temp = || {
        let _ = fs::remove_file(&temp_path);
    };
    
    // Write to temp file
    let mut file = fs::File::create(&temp_path)
        .with_context(|| format!("Failed to create temp file: {}", temp_path.display()))?;
    
    // Write content
    if let Err(e) = file.write_all(content) {
        cleanup_temp();
        return Err(e).with_context(|| "Failed to write content");
    }
    
    // Ensure data reaches disk
    if let Err(e) = file.sync_all() {
        cleanup_temp();
        return Err(e).with_context(|| "Failed to sync file");
    }
    
    // Drop file handle before rename
    drop(file);
    
    // Atomic rename
    if let Err(e) = fs::rename(&temp_path, path) {
        cleanup_temp();
        return Err(e).with_context(|| format!("Failed to rename temp file to: {}", path.display()));
    }
    
    // Set appropriate permissions on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mut perms) = fs::metadata(path).map(|m| m.permissions()) {
            perms.set_mode(0o644); // rw-r--r--
            let _ = fs::set_permissions(path, perms); // Best effort
        }
    }
    
    Ok(())
}

/// Read a daily goals file from disk
pub fn read_goals_file(path: &Path) -> Result<DailyGoals> {
    validate_path(path)?;
    
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    parse_markdown(&content)
}

/// Generate markdown content from DailyGoals
pub fn generate_markdown(goals: &DailyGoals) -> String {
    let mut content = String::new();
    
    // Header with date and optional day number
    let date_str = goals.date.format("%B %d, %Y");
    if let Some(day) = goals.day_number {
        content.push_str(&format!("# {} - Day {}\n\n", date_str, day));
    } else {
        content.push_str(&format!("# {}\n\n", date_str));
    }
    
    // Generate each outcome section
    for outcome in goals.outcomes() {
        generate_outcome_section(&mut content, outcome);
        content.push('\n');
    }
    
    content
}

/// Generate markdown for a single outcome section
fn generate_outcome_section(content: &mut String, outcome: &Outcome) {
    // Header with optional goal
    let header = if let Some(goal) = &outcome.goal {
        format!("## {} (Goal: {})\n", outcome.outcome_type.as_str(), goal)
    } else {
        format!("## {}\n", outcome.outcome_type.as_str())
    };
    content.push_str(&header);
    
    // Actions
    for action in &outcome.actions {
        let checkbox = if action.completed { "[x]" } else { "[ ]" };
        content.push_str(&format!("- {} {}\n", checkbox, action.text));
    }
}

/// Write goals to a file atomically with safety checks
pub fn write_goals_file(goals: &DailyGoals, config: &Config) -> Result<PathBuf> {
    // Validate config path
    let goals_dir = Path::new(&config.goals_dir);
    validate_path(goals_dir)?;
    
    // Ensure goals directory exists
    fs::create_dir_all(goals_dir)
        .with_context(|| format!("Failed to create goals directory: {}", goals_dir.display()))?;
    
    // Generate filename: YYYY-MM-DD.md
    let filename = format!("{}.md", goals.date.format("%Y-%m-%d"));
    let file_path = goals_dir.join(&filename);
    
    // Validate final file path
    validate_path(&file_path)?;
    
    // Generate markdown content
    let content = generate_markdown(goals);
    
    // Write atomically using safe implementation
    atomic_write_safe(&file_path, content.as_bytes())?;
    
    Ok(file_path)
}

/// Load existing goals for a date, or create new ones
pub fn load_or_create_goals(date: NaiveDate, config: &Config) -> Result<DailyGoals> {
    let goals_dir = Path::new(&config.goals_dir);
    validate_path(goals_dir)?;
    
    let filename = format!("{}.md", date.format("%Y-%m-%d"));
    let file_path = goals_dir.join(filename);
    
    if file_path.exists() {
        read_goals_file(&file_path)
    } else {
        Ok(DailyGoals::new(date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    use tempfile::TempDir;
    
    #[test]
    fn test_safe_atomic_write() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let target_path = temp_dir.path().join("test_file.md");
        let content = b"Test content for safe atomic write";

        let result = atomic_write_safe(&target_path, content);
        assert!(result.is_ok());
        
        // Verify file exists and has correct content
        assert!(target_path.exists());
        let written_content = fs::read(&target_path)?;
        assert_eq!(written_content, content);
        
        Ok(())
    }
    
    #[test]
    fn test_concurrent_safe_writes() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let target_path = Arc::new(temp_dir.path().join("concurrent_safe.md"));
        
        let mut handles = vec![];
        
        // Launch 10 threads writing concurrently
        for i in 0..10 {
            let path = Arc::clone(&target_path);
            let handle = thread::spawn(move || {
                let content = format!("Content from thread {}", i);
                atomic_write_safe(&path, content.as_bytes())
            });
            handles.push(handle);
        }
        
        // Wait for all threads and check for errors
        let mut errors = 0;
        for handle in handles {
            match handle.join() {
                Ok(Ok(_)) => {}, // Success
                Ok(Err(_)) => errors += 1,
                Err(_) => errors += 1,
            }
        }
        
        // All operations should succeed (no collisions)
        assert_eq!(errors, 0, "Concurrent writes had {} errors", errors);
        
        // File should exist and contain valid content
        assert!(target_path.exists());
        let final_content = fs::read_to_string(&*target_path)?;
        assert!(final_content.starts_with("Content from thread"));
        
        Ok(())
    }
    
    #[test]
    fn test_path_validation() {
        // Valid paths should pass
        assert!(validate_path(Path::new("valid/path/file.md")).is_ok());
        assert!(validate_path(Path::new("file.md")).is_ok());
        
        // Invalid paths should fail
        assert!(validate_path(Path::new("path\0with\0null")).is_err());
        assert!(validate_path(Path::new("../../../etc/passwd")).is_err());
        assert!(validate_path(Path::new(&"a".repeat(300))).is_err());
    }
    
    #[test]
    fn test_temp_file_uniqueness() {
        let target = Path::new("/tmp/test.md");
        
        // Generate multiple temp paths - they should all be unique
        let temp1 = generate_temp_path(target);
        let temp2 = generate_temp_path(target);
        let temp3 = generate_temp_path(target);
        
        assert_ne!(temp1, temp2);
        assert_ne!(temp2, temp3);
        assert_ne!(temp1, temp3);
        
        // All should be based on the original path
        assert!(temp1.to_string_lossy().contains("test"));
        assert!(temp2.to_string_lossy().contains("test"));
        assert!(temp3.to_string_lossy().contains("test"));
    }
    
    #[test]
    fn test_cleanup_old_temp_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path();
        
        // Create some temp files
        let old_temp = dir_path.join("old_file.tmp");
        let new_temp = dir_path.join("new_file.tmp.123");
        let regular_file = dir_path.join("regular.md");
        
        fs::write(&old_temp, "old temp content")?;
        fs::write(&new_temp, "new temp content")?;
        fs::write(&regular_file, "regular content")?;
        
        // Run cleanup (won't actually remove new files in test, but shouldn't error)
        let result = cleanup_old_temp_files(dir_path);
        assert!(result.is_ok());
        
        // Regular file should still exist
        assert!(regular_file.exists());
        
        Ok(())
    }
    
    #[test]
    fn test_full_safe_workflow() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
        };
        
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);
        
        goals.work.goal = Some("Complete safe implementation".to_string());
        goals.work.actions[0] = Action {
            text: "Fix atomic write issues".to_string(),
            completed: true,
        };
        
        // Write goals safely
        let saved_path = write_goals_file(&goals, &config)?;
        assert!(saved_path.exists());
        
        // Load back and verify
        let loaded_goals = load_or_create_goals(date, &config)?;
        assert_eq!(loaded_goals.date, goals.date);
        assert_eq!(loaded_goals.work.goal, goals.work.goal);
        assert_eq!(loaded_goals.work.actions[0].text, goals.work.actions[0].text);
        assert_eq!(loaded_goals.work.actions[0].completed, goals.work.actions[0].completed);
        
        Ok(())
    }
}