use crate::models::{Action, ActionTemplates, Config, DailyGoals, FiveYearVision, Outcome};
use anyhow::{Context, Result};
use chrono::{Local, NaiveDate};
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Safely get capture group as string
fn get_capture<'a>(caps: &'a regex::Captures, index: usize) -> Result<&'a str> {
    caps.get(index)
        .map(|m| m.as_str())
        .context(format!("Missing capture group {}", index))
}

/// Safely parse capture group as type T
fn parse_capture<T: std::str::FromStr>(caps: &regex::Captures, index: usize) -> Result<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let value = get_capture(caps, index)?;
    value
        .parse()
        .with_context(|| format!("Failed to parse capture group {} as expected type", index))
}

/// Parse a markdown file into DailyGoals
pub fn parse_markdown(content: &str) -> Result<DailyGoals> {
    let lines: Vec<&str> = content.lines().collect();

    // Find the date header in first 10 lines (not just line 0)
    let (header_index, date) = find_date_header(&lines)?;

    let mut goals = DailyGoals::new(date);
    let mut current_outcome: Option<&mut Outcome> = None;
    let mut action_index = 0;

    // Extract day number from the header line we found
    if let Some(day_num) = extract_day_number(lines[header_index]) {
        goals.day_number = Some(day_num);
    }

    // Parse from header onwards, tracking line numbers for better errors
    for (line_num, line) in lines.iter().enumerate().skip(header_index + 1) {
        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Check for outcome headers (case-insensitive)
        let line_lower = line.to_lowercase();
        if line_lower.starts_with("## work") {
            goals.work.goal = extract_goal_from_header(line);
            current_outcome = Some(&mut goals.work);
            action_index = 0;
        } else if line_lower.starts_with("## health") {
            goals.health.goal = extract_goal_from_header(line);
            current_outcome = Some(&mut goals.health);
            action_index = 0;
        } else if line_lower.starts_with("## family") {
            goals.family.goal = extract_goal_from_header(line);
            current_outcome = Some(&mut goals.family);
            action_index = 0;
        } else if line.starts_with("- [") {
            // Parse action
            if let Some(outcome) = current_outcome.as_mut() {
                if outcome.actions.len() < 5 {  // Max 5 actions per outcome
                    let (completed, text) = parse_action_line(line).with_context(|| {
                        format!("Failed to parse action on line {}", line_num + 1)
                    })?;
                    
                    // For existing files with pre-allocated actions
                    if action_index < outcome.actions.len() {
                        outcome.actions[action_index] = Action { text, completed };
                    } else {
                        // For new actions beyond the default 3
                        outcome.actions.push(Action { text, completed });
                    }
                    action_index += 1;
                } else {
                    // Warning for more than 5 actions
                    eprintln!(
                        "Warning (line {}): More than 5 actions for {:?}, ignoring: {}",
                        line_num + 1,
                        outcome.outcome_type,
                        line
                    );
                }
            }
        }
    }

    Ok(goals)
}

/// Find the date header in the first few lines of the file
fn find_date_header(lines: &[&str]) -> Result<(usize, NaiveDate)> {
    // Search first 10 lines for a valid date header
    for (index, line) in lines.iter().take(10).enumerate() {
        if let Ok(date) = parse_date_header(line) {
            return Ok((index, date));
        }
    }

    anyhow::bail!("No valid date header found in first 10 lines. Expected format: # Month DD, YYYY")
}

/// Parse the date from the header line
fn parse_date_header(header: &str) -> Result<NaiveDate> {
    // Pattern: # Month DD, YYYY - Day N
    let re = Regex::new(r"#\s*(\w+)\s+(\d{1,2}),\s*(\d{4})")?;

    let caps = re
        .captures(header)
        .context(format!("Could not parse date from header: {}", header))?;

    let month_str = get_capture(&caps, 1)?;
    let day: u32 = parse_capture(&caps, 2)?;
    let year: i32 = parse_capture(&caps, 3)?;

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
        .context(format!("Invalid date: {}-{}-{}", year, month, day))
}

/// Extract day number from header if present
fn extract_day_number(header: &str) -> Option<u32> {
    let re = Regex::new(r"Day\s+(\d+)").ok()?;
    re.captures(header)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// Extract goal description from outcome header
fn extract_goal_from_header(header: &str) -> Option<String> {
    // Pattern: ## Outcome (Goal: description)
    let re = Regex::new(r"\(Goal:\s*([^)]+)\)").ok()?;
    re.captures(header)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Parse an action line into completion status and text
fn parse_action_line(line: &str) -> Result<(bool, String)> {
    if let Some(text) = line.strip_prefix("- [x]").or_else(|| line.strip_prefix("- [X]")) {
        Ok((true, text.trim().to_string()))
    } else if let Some(text) = line.strip_prefix("- [ ]") {
        Ok((false, text.trim().to_string()))
    } else {
        anyhow::bail!("Invalid action line format: {}", line)
    }
}

/// Read a daily goals file from disk
pub fn read_goals_file(path: &Path) -> Result<DailyGoals> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    parse_markdown(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown() {
        let markdown = r#"# January 15, 2025 - Day 12

## Work (Goal: Ship v1)
- [x] Call investors
- [ ] Prep deck
- [ ] Team standup

## Health (Goal: Run 5k)
- [x] Morning run
- [ ] Meal prep
- [ ] Sleep by 10pm

## Family (Goal: Be present)
- [ ] Call parents
- [x] Plan weekend
- [x] Homework help"#;

        let goals = parse_markdown(markdown).unwrap();

        assert_eq!(goals.date, NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
        assert_eq!(goals.day_number, Some(12));

        // Check Work outcome
        assert_eq!(goals.work.goal, Some("Ship v1".to_string()));
        assert!(goals.work.actions[0].completed);
        assert!(!goals.work.actions[1].completed);

        // Check Health outcome
        assert_eq!(goals.health.goal, Some("Run 5k".to_string()));
        assert!(goals.health.actions[0].completed);

        // Check Family outcome
        assert_eq!(goals.family.goal, Some("Be present".to_string()));
        assert!(goals.family.actions[1].completed);
        assert!(goals.family.actions[2].completed);
    }

    #[test]
    fn test_parse_action_line() {
        let (completed, text) = parse_action_line("- [x] Complete task").unwrap();
        assert!(completed);
        assert_eq!(text, "Complete task");

        let (completed, text) = parse_action_line("- [ ] Pending task").unwrap();
        assert!(!completed);
        assert_eq!(text, "Pending task");
    }

    #[test]
    fn test_round_trip() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let mut goals = DailyGoals::new(date);
        goals.day_number = Some(12);

        goals.work.goal = Some("Ship v1".to_string());
        goals.work.actions[0] = Action {
            text: "Call investors".to_string(),
            completed: true,
        };
        goals.work.actions[1] = Action {
            text: "Prep deck".to_string(),
            completed: false,
        };
        goals.work.actions[2] = Action {
            text: "Team standup".to_string(),
            completed: false,
        };

        let markdown = generate_markdown(&goals);
        let parsed = parse_markdown(&markdown).unwrap();

        assert_eq!(goals.date, parsed.date);
        assert_eq!(goals.work.actions[0].text, parsed.work.actions[0].text);
        assert_eq!(
            goals.work.actions[0].completed,
            parsed.work.actions[0].completed
        );
    }
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

/// Calculate the current streak of consecutive days with at least one completed task
pub fn calculate_streak(config: &Config) -> Result<u32> {
    let goals_dir = Path::new(&config.goals_dir);
    if !goals_dir.exists() {
        return Ok(0);
    }

    let mut streak = 0;
    let mut current_date = Local::now().date_naive();

    loop {
        let file_path = goals_dir.join(format!("{}.md", current_date.format("%Y-%m-%d")));

        if file_path.exists() {
            // Try to read and parse the file
            match read_goals_file(&file_path) {
                Ok(goals) => {
                    // Check if at least one action is completed
                    let has_completion = goals
                        .outcomes()
                        .iter()
                        .flat_map(|o| &o.actions)
                        .any(|a| a.completed && !a.text.is_empty());

                    if has_completion {
                        streak += 1;
                        current_date = current_date.pred_opt().unwrap_or(current_date);
                    } else {
                        break;
                    }
                }
                Err(_) => break, // File exists but can't be parsed
            }
        } else {
            break; // No file for this date
        }

        // Safety limit to prevent infinite loops
        if streak > 365 {
            break;
        }
    }

    Ok(streak)
}

/// Write goals to a file atomically
pub fn write_goals_file(goals: &DailyGoals, config: &Config) -> Result<PathBuf> {
    // Ensure goals directory exists
    let goals_dir = Path::new(&config.goals_dir);
    fs::create_dir_all(goals_dir)
        .with_context(|| format!("Failed to create goals directory: {}", goals_dir.display()))?;

    // Generate filename: YYYY-MM-DD.md
    let filename = format!("{}.md", goals.date.format("%Y-%m-%d"));
    let file_path = goals_dir.join(&filename);

    // Generate markdown content
    let content = generate_markdown(goals);

    // Write atomically using temp file + rename
    atomic_write(&file_path, content.as_bytes())?;

    Ok(file_path)
}

/// Atomically write to a file by writing to temp file then renaming
fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    // Create unique temp filename with timestamp and process ID
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();

    // Build temp path in same directory as target file
    let temp_filename = format!(
        ".{}.tmp.{}.{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("tmp"),
        timestamp,
        pid
    );

    let temp_path = path
        .parent()
        .map(|p| p.join(&temp_filename))
        .unwrap_or_else(|| PathBuf::from(&temp_filename));

    // Write to temp file
    let mut file = fs::File::create(&temp_path)
        .with_context(|| format!("Failed to create temp file: {}", temp_path.display()))?;
    file.write_all(content)
        .with_context(|| "Failed to write content")?;
    file.sync_all().with_context(|| "Failed to sync file")?;

    // Atomic rename
    fs::rename(&temp_path, path)
        .inspect_err(|_| {
            // Cleanup on failure
            let _ = fs::remove_file(&temp_path);
        })
        .with_context(|| format!("Failed to rename temp file to: {}", path.display()))?;

    Ok(())
}

/// Load existing goals for a date, or create new ones
pub fn load_or_create_goals(date: NaiveDate, config: &Config) -> Result<DailyGoals> {
    let goals_dir = Path::new(&config.goals_dir);
    let filename = format!("{}.md", date.format("%Y-%m-%d"));
    let file_path = goals_dir.join(filename);

    if file_path.exists() {
        read_goals_file(&file_path)
    } else {
        Ok(DailyGoals::new(date))
    }
}

/// Get yesterday's goals if they exist
pub fn get_yesterday_goals(today: NaiveDate, config: &Config) -> Result<Option<DailyGoals>> {
    let yesterday = today.pred_opt().context("Cannot get yesterday's date")?;

    let goals_dir = Path::new(&config.goals_dir);
    let filename = format!("{}.md", yesterday.format("%Y-%m-%d"));
    let file_path = goals_dir.join(filename);

    if file_path.exists() {
        Ok(Some(read_goals_file(&file_path)?))
    } else {
        Ok(None)
    }
}

/// Load or create the 5-year vision file
pub fn load_or_create_vision(config: &Config) -> Result<FiveYearVision> {
    let vision_path = Path::new(&config.goals_dir)
        .parent()
        .unwrap_or(Path::new(&config.goals_dir))
        .join("vision.json");

    if vision_path.exists() {
        let content = fs::read_to_string(&vision_path)
            .with_context(|| format!("Failed to read vision file: {}", vision_path.display()))?;
        serde_json::from_str(&content).with_context(|| "Failed to parse vision file")
    } else {
        Ok(FiveYearVision::new())
    }
}

/// Save the 5-year vision to file
pub fn save_vision(vision: &FiveYearVision, config: &Config) -> Result<()> {
    let vision_dir = Path::new(&config.goals_dir)
        .parent()
        .unwrap_or(Path::new(&config.goals_dir));

    // Ensure directory exists
    fs::create_dir_all(vision_dir).with_context(|| {
        format!(
            "Failed to create vision directory: {}",
            vision_dir.display()
        )
    })?;

    let vision_path = vision_dir.join("vision.json");
    let json_content =
        serde_json::to_string_pretty(vision).with_context(|| "Failed to serialize vision")?;

    // Use atomic write for safety
    atomic_write(&vision_path, json_content.as_bytes())?;

    Ok(())
}

/// Load or create the action templates file
pub fn load_or_create_templates(config: &Config) -> Result<ActionTemplates> {
    let templates_path = Path::new(&config.goals_dir)
        .parent()
        .unwrap_or(Path::new(&config.goals_dir))
        .join("templates.json");

    if templates_path.exists() {
        let content = fs::read_to_string(&templates_path).with_context(|| {
            format!(
                "Failed to read templates file: {}",
                templates_path.display()
            )
        })?;
        serde_json::from_str(&content).with_context(|| "Failed to parse templates file")
    } else {
        Ok(ActionTemplates::new())
    }
}

/// Save the action templates to file
pub fn save_templates(templates: &ActionTemplates, config: &Config) -> Result<()> {
    let templates_dir = Path::new(&config.goals_dir)
        .parent()
        .unwrap_or(Path::new(&config.goals_dir));

    // Ensure directory exists
    fs::create_dir_all(templates_dir).with_context(|| {
        format!(
            "Failed to create templates directory: {}",
            templates_dir.display()
        )
    })?;

    let templates_path = templates_dir.join("templates.json");
    let json_content =
        serde_json::to_string_pretty(templates).with_context(|| "Failed to serialize templates")?;

    // Use atomic write for safety
    atomic_write(&templates_path, json_content.as_bytes())?;

    Ok(())
}
