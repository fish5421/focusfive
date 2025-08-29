use anyhow::Result;
use chrono::{Local, NaiveDate};
use focusfive::data::{
    append_observation, load_or_create_indicators, read_observations_range, save_indicators,
};
use focusfive::models::{
    Config, IndicatorDef, IndicatorDirection, IndicatorKind, IndicatorUnit, IndicatorsData,
    Observation, ObservationSource,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_indicator_serialization() -> Result<()> {
    // Test all enum variants and fields
    let mut indicators = IndicatorsData {
        version: 1,
        indicators: vec![
            IndicatorDef {
                id: "ind-1".to_string(),
                name: "Qualified Leads (7d)".to_string(),
                kind: IndicatorKind::Leading,
                unit: IndicatorUnit::Count,
                objective_id: Some("obj-123".to_string()),
                target: Some(50.0),
                direction: IndicatorDirection::HigherIsBetter,
                active: true,
                created: chrono::Utc::now(),
                modified: chrono::Utc::now(),
                lineage_of: None,
                notes: Some("Weekly sales leads".to_string()),
            },
            IndicatorDef {
                id: "ind-2".to_string(),
                name: "Customer Churn Rate".to_string(),
                kind: IndicatorKind::Lagging,
                unit: IndicatorUnit::Percent,
                objective_id: None,
                target: Some(5.0),
                direction: IndicatorDirection::LowerIsBetter,
                active: true,
                created: chrono::Utc::now(),
                modified: chrono::Utc::now(),
                lineage_of: Some("old-ind-2".to_string()),
                notes: None,
            },
            IndicatorDef {
                id: "ind-3".to_string(),
                name: "Daily Exercise".to_string(),
                kind: IndicatorKind::Leading,
                unit: IndicatorUnit::Minutes,
                objective_id: Some("health-obj".to_string()),
                target: Some(30.0),
                direction: IndicatorDirection::WithinRange,
                active: false,
                created: chrono::Utc::now(),
                modified: chrono::Utc::now(),
                lineage_of: None,
                notes: None,
            },
        ],
    };

    // Test JSON serialization
    let json = serde_json::to_string_pretty(&indicators)?;
    assert!(json.contains("\"version\": 1"));
    assert!(json.contains("\"Qualified Leads (7d)\""));
    assert!(json.contains("\"Leading\""));
    assert!(json.contains("\"Lagging\""));
    assert!(json.contains("\"type\": \"Count\""));
    assert!(json.contains("\"type\": \"Percent\""));
    assert!(json.contains("\"type\": \"Minutes\""));
    assert!(json.contains("\"HigherIsBetter\""));
    assert!(json.contains("\"LowerIsBetter\""));
    assert!(json.contains("\"WithinRange\""));

    // Test deserialization
    let deserialized: IndicatorsData = serde_json::from_str(&json)?;
    assert_eq!(deserialized.version, 1);
    assert_eq!(deserialized.indicators.len(), 3);
    assert_eq!(deserialized.indicators[0].name, "Qualified Leads (7d)");
    assert_eq!(deserialized.indicators[0].kind, IndicatorKind::Leading);
    assert_eq!(deserialized.indicators[1].direction, IndicatorDirection::LowerIsBetter);
    assert!(!deserialized.indicators[2].active);

    // Test Custom unit type
    indicators.indicators.push(IndicatorDef {
        id: "ind-4".to_string(),
        name: "Code Coverage".to_string(),
        kind: IndicatorKind::Lagging,
        unit: IndicatorUnit::Custom("Lines".to_string()),
        objective_id: None,
        target: Some(80.0),
        direction: IndicatorDirection::HigherIsBetter,
        active: true,
        created: chrono::Utc::now(),
        modified: chrono::Utc::now(),
        lineage_of: None,
        notes: None,
    });

    let json2 = serde_json::to_string(&indicators)?;
    assert!(json2.contains("\"type\":\"Custom\",\"value\":\"Lines\""));

    // Test Dollars unit
    let dollar_indicator = IndicatorDef::new(
        "Revenue".to_string(),
        IndicatorKind::Lagging,
        IndicatorUnit::Dollars,
    );
    assert_eq!(dollar_indicator.unit, IndicatorUnit::Dollars);

    Ok(())
}

#[test]
fn test_observation_serialization() -> Result<()> {
    let obs = Observation {
        id: "obs-1".to_string(),
        indicator_id: "ind-1".to_string(),
        when: NaiveDate::from_ymd_opt(2025, 8, 28).unwrap(),
        value: 7.0,
        unit: IndicatorUnit::Count,
        source: ObservationSource::Manual,
        action_id: Some("action-123".to_string()),
        note: Some("Morning count".to_string()),
        created: chrono::Utc::now(),
    };

    // Test JSON serialization (should be single line for NDJSON)
    let json = serde_json::to_string(&obs)?;
    assert!(!json.contains('\n')); // NDJSON requires single line
    assert!(json.contains("\"obs-1\""));
    assert!(json.contains("\"2025-08-28\""));
    assert!(json.contains("7.0"));
    assert!(json.contains("\"Manual\""));

    // Test deserialization
    let deserialized: Observation = serde_json::from_str(&json)?;
    assert_eq!(deserialized.id, "obs-1");
    assert_eq!(deserialized.value, 7.0);
    assert_eq!(deserialized.source, ObservationSource::Manual);

    // Test other source types
    let mut obs2 = obs.clone();
    obs2.source = ObservationSource::Automated;
    let json2 = serde_json::to_string(&obs2)?;
    assert!(json2.contains("\"Automated\""));

    obs2.source = ObservationSource::Import;
    let json3 = serde_json::to_string(&obs2)?;
    assert!(json3.contains("\"Import\""));

    Ok(())
}

#[test]
fn test_indicators_save_and_load() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Create test indicators
    let indicators = IndicatorsData {
        version: 1,
        indicators: vec![
            IndicatorDef::new(
                "Daily Steps".to_string(),
                IndicatorKind::Leading,
                IndicatorUnit::Count,
            ),
            IndicatorDef::new(
                "Weight".to_string(),
                IndicatorKind::Lagging,
                IndicatorUnit::Custom("kg".to_string()),
            ),
        ],
    };

    // Save indicators
    let save_path = save_indicators(&indicators, &config)?;
    assert!(save_path.exists());
    assert!(save_path.to_string_lossy().ends_with("indicators.json"));

    // Load indicators back
    let loaded = load_or_create_indicators(&config)?;
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.indicators.len(), 2);
    assert_eq!(loaded.indicators[0].name, "Daily Steps");
    assert_eq!(loaded.indicators[1].name, "Weight");

    Ok(())
}

#[test]
fn test_indicators_load_creates_default() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Load when file doesn't exist should return default
    let indicators = load_or_create_indicators(&config)?;
    assert_eq!(indicators.version, 1);
    assert_eq!(indicators.indicators.len(), 0);

    Ok(())
}

#[test]
fn test_observation_append_and_count() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;
    let observations_path = std::path::Path::new(&config.data_root).join("observations.ndjson");

    // Initially no file
    assert!(!observations_path.exists());

    // Append first observation
    let obs1 = Observation::new(
        "ind-1".to_string(),
        NaiveDate::from_ymd_opt(2025, 8, 28).unwrap(),
        10.0,
        IndicatorUnit::Count,
    );
    append_observation(&obs1, &config)?;

    // File should now exist
    assert!(observations_path.exists());

    // Count lines - should be 1
    let content = fs::read_to_string(&observations_path)?;
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 1);

    // Append second observation
    let obs2 = Observation::new(
        "ind-1".to_string(),
        NaiveDate::from_ymd_opt(2025, 8, 29).unwrap(),
        12.0,
        IndicatorUnit::Count,
    );
    append_observation(&obs2, &config)?;

    // Count lines - should be 2
    let content = fs::read_to_string(&observations_path)?;
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2);

    // Verify each line is valid JSON
    for line in lines {
        let parsed: Observation = serde_json::from_str(line)?;
        assert!(!parsed.id.is_empty());
    }

    Ok(())
}

#[test]
fn test_observation_read_range() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Create observations for different dates
    let dates = vec![
        NaiveDate::from_ymd_opt(2025, 8, 25).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 26).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 27).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 28).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 29).unwrap(),
    ];

    for (i, date) in dates.iter().enumerate() {
        let obs = Observation::new(
            "ind-1".to_string(),
            *date,
            (i + 1) as f64,
            IndicatorUnit::Count,
        );
        append_observation(&obs, &config)?;
    }

    // Read full range
    let all_obs = read_observations_range(
        NaiveDate::from_ymd_opt(2025, 8, 25).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 29).unwrap(),
        &config,
    )?;
    assert_eq!(all_obs.len(), 5);

    // Read partial range (middle 3 days)
    let partial_obs = read_observations_range(
        NaiveDate::from_ymd_opt(2025, 8, 26).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 28).unwrap(),
        &config,
    )?;
    assert_eq!(partial_obs.len(), 3);
    assert_eq!(partial_obs[0].value, 2.0);
    assert_eq!(partial_obs[1].value, 3.0);
    assert_eq!(partial_obs[2].value, 4.0);

    // Read single day
    let single_obs = read_observations_range(
        NaiveDate::from_ymd_opt(2025, 8, 27).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 27).unwrap(),
        &config,
    )?;
    assert_eq!(single_obs.len(), 1);
    assert_eq!(single_obs[0].value, 3.0);

    // Read range with no data
    let no_obs = read_observations_range(
        NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 9, 30).unwrap(),
        &config,
    )?;
    assert_eq!(no_obs.len(), 0);

    Ok(())
}

#[test]
fn test_observation_read_nonexistent_file() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Reading from non-existent file should return empty vec
    let obs = read_observations_range(
        NaiveDate::from_ymd_opt(2025, 8, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 31).unwrap(),
        &config,
    )?;
    assert_eq!(obs.len(), 0);

    Ok(())
}

#[test]
fn test_indicator_creation_helper() -> Result<()> {
    let ind = IndicatorDef::new(
        "Test Indicator".to_string(),
        IndicatorKind::Leading,
        IndicatorUnit::Minutes,
    );

    assert!(!ind.id.is_empty());
    assert_eq!(ind.name, "Test Indicator");
    assert_eq!(ind.kind, IndicatorKind::Leading);
    assert_eq!(ind.unit, IndicatorUnit::Minutes);
    assert!(ind.objective_id.is_none());
    assert!(ind.target.is_none());
    assert_eq!(ind.direction, IndicatorDirection::HigherIsBetter);
    assert!(ind.active);
    assert!(ind.lineage_of.is_none());
    assert!(ind.notes.is_none());

    // Verify UUID format
    assert!(ind.id.len() >= 32);

    Ok(())
}

#[test]
fn test_observation_creation_helper() -> Result<()> {
    let obs = Observation::new(
        "ind-123".to_string(),
        Local::now().date_naive(),
        42.5,
        IndicatorUnit::Percent,
    );

    assert!(!obs.id.is_empty());
    assert_eq!(obs.indicator_id, "ind-123");
    assert_eq!(obs.value, 42.5);
    assert_eq!(obs.unit, IndicatorUnit::Percent);
    assert_eq!(obs.source, ObservationSource::Manual);
    assert!(obs.action_id.is_none());
    assert!(obs.note.is_none());

    // Verify UUID format
    assert!(obs.id.len() >= 32);

    Ok(())
}