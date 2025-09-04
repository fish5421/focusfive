use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::models::{DailyGoals, OutcomeType};

/// Schema version for all data files
const SCHEMA_VERSION: u32 = 1;

/// Enhanced Action with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionMeta {
    pub id: Uuid,
    pub text: String,
    pub completed: bool,
    pub status: ActionStatus,
    pub effort_minutes: Option<u32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub outcome_type: OutcomeType,
    pub action_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionStatus {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
    Deferred,
}

/// Per-day metadata sidecar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayMetadata {
    pub version: u32,
    pub date: NaiveDate,
    pub day_number: Option<u32>,
    pub actions: Vec<ActionMeta>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Objective with optional Key Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub outcome_type: OutcomeType,
    pub target_date: Option<NaiveDate>,
    pub key_results: Vec<KeyResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyResult {
    pub id: Uuid,
    pub description: String,
    pub target_value: f64,
    pub current_value: f64,
    pub unit: Option<String>,
}

/// Indicator definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indicator {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub outcome_type: OutcomeType,
    pub unit: Option<String>,
    pub target_value: Option<f64>,
    pub frequency: MeasurementFrequency,
    pub created_at: DateTime<Utc>,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeasurementFrequency {
    Daily,
    Weekly,
    Monthly,
}

/// Observation (measurement of an indicator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: Uuid,
    pub indicator_id: Uuid,
    pub value: f64,
    pub notes: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Review (weekly/monthly retrospective)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub version: u32,
    pub id: Uuid,
    pub period_type: ReviewPeriod,
    pub period_identifier: String, // e.g., "2025-W35" or "2025-08"
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub wins: Vec<String>,
    pub challenges: Vec<String>,
    pub learnings: Vec<String>,
    pub next_actions: Vec<String>,
    pub completion_stats: HashMap<OutcomeType, CompletionSummary>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewPeriod {
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionSummary {
    pub total_actions: u32,
    pub completed_actions: u32,
    pub completion_rate: f64,
}

/// Data storage configuration
pub struct DataStorage {
    pub data_root: PathBuf,
    pub goals_dir: PathBuf,
    pub meta_dir: PathBuf,
    pub reviews_dir: PathBuf,
}

impl DataStorage {
    /// Create new storage configuration from Config
    pub fn new(config: &crate::models::Config) -> Result<Self> {
        let data_root = PathBuf::from(&config.data_root);

        // Ensure data_root exists
        fs::create_dir_all(&data_root)
            .with_context(|| format!("Failed to create data root directory: {:?}", data_root))?;

        let goals_dir = PathBuf::from(&config.goals_dir);
        let meta_dir = data_root.join("meta");
        let reviews_dir = data_root.join("reviews");

        // Ensure subdirectories exist
        fs::create_dir_all(&goals_dir)
            .with_context(|| format!("Failed to create goals directory: {:?}", goals_dir))?;
        fs::create_dir_all(&meta_dir)
            .with_context(|| format!("Failed to create meta directory: {:?}", meta_dir))?;
        fs::create_dir_all(&reviews_dir)
            .with_context(|| format!("Failed to create reviews directory: {:?}", reviews_dir))?;

        Ok(Self {
            data_root,
            goals_dir,
            meta_dir,
            reviews_dir,
        })
    }

    /// Save day metadata
    pub fn save_day_metadata(&self, meta: &DayMetadata) -> Result<()> {
        let filename = format!("{}.meta.json", meta.date.format("%Y-%m-%d"));
        let path = self.meta_dir.join(filename);

        let json =
            serde_json::to_string_pretty(meta).context("Failed to serialize day metadata")?;

        atomic_write(&path, json.as_bytes())?;
        Ok(())
    }

    /// Load day metadata
    pub fn load_day_metadata(&self, date: NaiveDate) -> Result<Option<DayMetadata>> {
        let filename = format!("{}.meta.json", date.format("%Y-%m-%d"));
        let path = self.meta_dir.join(filename);

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read metadata file: {:?}", path))?;

        let meta: DayMetadata =
            serde_json::from_str(&content).context("Failed to parse day metadata")?;

        Ok(Some(meta))
    }

    /// Save objectives
    pub fn save_objectives(&self, objectives: &[Objective]) -> Result<()> {
        let path = self.data_root.join("objectives.json");

        let container = ObjectivesContainer {
            version: SCHEMA_VERSION,
            objectives: objectives.to_vec(),
        };

        let json =
            serde_json::to_string_pretty(&container).context("Failed to serialize objectives")?;

        atomic_write(&path, json.as_bytes())?;
        Ok(())
    }

    /// Load objectives
    pub fn load_objectives(&self) -> Result<Vec<Objective>> {
        let path = self.data_root.join("objectives.json");

        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read objectives file: {:?}", path))?;

        let container: ObjectivesContainer =
            serde_json::from_str(&content).context("Failed to parse objectives")?;

        Ok(container.objectives)
    }

    /// Save indicators
    pub fn save_indicators(&self, indicators: &[Indicator]) -> Result<()> {
        let path = self.data_root.join("indicators.json");

        let container = IndicatorsContainer {
            version: SCHEMA_VERSION,
            indicators: indicators.to_vec(),
        };

        let json =
            serde_json::to_string_pretty(&container).context("Failed to serialize indicators")?;

        atomic_write(&path, json.as_bytes())?;
        Ok(())
    }

    /// Append observation to NDJSON file
    pub fn append_observation(&self, observation: &Observation) -> Result<()> {
        let path = self.data_root.join("observations.ndjson");

        // Serialize to single line
        let json = serde_json::to_string(observation).context("Failed to serialize observation")?;

        // Append with newline
        append_to_file(&path, format!("{}\n", json).as_bytes())?;
        Ok(())
    }

    /// Load observations (optionally filtered by indicator)
    pub fn load_observations(&self, indicator_id: Option<Uuid>) -> Result<Vec<Observation>> {
        let path = self.data_root.join("observations.ndjson");

        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read observations file: {:?}", path))?;

        let mut observations = Vec::new();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let obs: Observation = serde_json::from_str(line)
                .with_context(|| format!("Failed to parse observation: {}", line))?;

            if let Some(id) = indicator_id {
                if obs.indicator_id == id {
                    observations.push(obs);
                }
            } else {
                observations.push(obs);
            }
        }

        Ok(observations)
    }

    /// Save review
    pub fn save_review(&self, review: &Review) -> Result<()> {
        let filename = format!("{}.json", review.period_identifier);
        let path = self.reviews_dir.join(filename);

        let json = serde_json::to_string_pretty(review).context("Failed to serialize review")?;

        atomic_write(&path, json.as_bytes())?;
        Ok(())
    }

    /// Create day metadata from DailyGoals (for migration/compatibility)
    pub fn create_day_metadata_from_goals(&self, goals: &DailyGoals) -> DayMetadata {
        let mut actions = Vec::new();
        let now = Utc::now();

        // Process each outcome's actions
        for (outcome_idx, outcome) in [&goals.work, &goals.health, &goals.family]
            .iter()
            .enumerate()
        {
            let outcome_type = match outcome_idx {
                0 => OutcomeType::Work,
                1 => OutcomeType::Health,
                2 => OutcomeType::Family,
                _ => unreachable!(),
            };

            for (action_idx, action) in outcome.actions.iter().enumerate() {
                let status = if action.completed {
                    ActionStatus::Completed
                } else {
                    ActionStatus::NotStarted
                };

                let meta = ActionMeta {
                    id: Uuid::new_v4(),
                    text: action.text.clone(),
                    completed: action.completed,
                    status,
                    effort_minutes: None,
                    notes: None,
                    created_at: now,
                    completed_at: if action.completed { Some(now) } else { None },
                    outcome_type,
                    action_index: action_idx,
                };

                actions.push(meta);
            }
        }

        DayMetadata {
            version: SCHEMA_VERSION,
            date: goals.date,
            day_number: goals.day_number,
            actions,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }
}

// Container types for versioned JSON files
#[derive(Debug, Serialize, Deserialize)]
struct ObjectivesContainer {
    version: u32,
    objectives: Vec<Objective>,
}

#[derive(Debug, Serialize, Deserialize)]
struct IndicatorsContainer {
    version: u32,
    indicators: Vec<Indicator>,
}

/// Atomically write to a file
fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();

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
        .with_context(|| format!("Failed to create temp file: {:?}", temp_path))?;
    file.write_all(content)?;
    file.sync_all()?;

    // Atomic rename
    fs::rename(&temp_path, path)
        .inspect_err(|_| {
            let _ = fs::remove_file(&temp_path);
        })
        .with_context(|| format!("Failed to rename temp file to: {:?}", path))?;

    Ok(())
}

/// Append to file (for NDJSON)
fn append_to_file(path: &Path, content: &[u8]) -> Result<()> {
    use std::fs::OpenOptions;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Failed to open file for appending: {:?}", path))?;

    file.write_all(content)?;
    file.sync_all()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_data_storage_creation() {
        let temp = TempDir::new().unwrap();
        std::env::set_var("HOME", temp.path());

        let config = crate::models::Config::new().unwrap();
        let storage = DataStorage::new(&config).unwrap();

        assert!(storage.goals_dir.exists());
        assert!(storage.meta_dir.exists());
        assert!(storage.reviews_dir.exists());
    }

    #[test]
    fn test_save_load_day_metadata() {
        let temp = TempDir::new().unwrap();
        std::env::set_var("HOME", temp.path());

        let config = crate::models::Config::new().unwrap();
        let storage = DataStorage::new(&config).unwrap();
        let date = NaiveDate::from_ymd_opt(2025, 8, 28).unwrap();

        let meta = DayMetadata {
            version: SCHEMA_VERSION,
            date,
            day_number: Some(5),
            actions: vec![],
            notes: Some("Test notes".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        storage.save_day_metadata(&meta).unwrap();

        let loaded = storage.load_day_metadata(date).unwrap().unwrap();
        assert_eq!(loaded.date, date);
        assert_eq!(loaded.day_number, Some(5));
        assert_eq!(loaded.notes, Some("Test notes".to_string()));
    }

    #[test]
    fn test_observations_ndjson() {
        let temp = TempDir::new().unwrap();
        std::env::set_var("HOME", temp.path());

        let config = crate::models::Config::new().unwrap();
        let storage = DataStorage::new(&config).unwrap();
        let indicator_id = Uuid::new_v4();

        let obs1 = Observation {
            id: Uuid::new_v4(),
            indicator_id,
            value: 7.5,
            notes: Some("First observation".to_string()),
            observed_at: Utc::now(),
            created_at: Utc::now(),
        };

        let obs2 = Observation {
            id: Uuid::new_v4(),
            indicator_id,
            value: 8.0,
            notes: Some("Second observation".to_string()),
            observed_at: Utc::now(),
            created_at: Utc::now(),
        };

        storage.append_observation(&obs1).unwrap();
        storage.append_observation(&obs2).unwrap();

        let loaded = storage.load_observations(Some(indicator_id)).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].value, 7.5);
        assert_eq!(loaded[1].value, 8.0);
    }
}
