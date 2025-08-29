use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use focusfive::data::{load_review, save_review};
use focusfive::models::{Config, Decision, Review, ReviewData, ReviewPeriod};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_review_serialization() -> Result<()> {
    let mut review = Review {
        id: "review-123".to_string(),
        date: NaiveDate::from_ymd_opt(2025, 8, 31).unwrap(),
        period: ReviewPeriod::Weekly,
        notes: Some("Good week overall, met most targets.".to_string()),
        score_1_to_5: 4,
        decisions: vec![
            Decision {
                summary: "Double outreach efforts".to_string(),
                objective_id: Some("obj-abc".to_string()),
                indicator_id: None,
                rationale: Some("Leads lagging behind target.".to_string()),
            },
            Decision {
                summary: "Reduce meeting frequency".to_string(),
                objective_id: None,
                indicator_id: Some("ind-xyz".to_string()),
                rationale: Some("Too much time in meetings.".to_string()),
            },
        ],
    };

    // Test JSON serialization
    let review_data = ReviewData {
        version: 1,
        review: review.clone(),
    };
    
    let json = serde_json::to_string_pretty(&review_data)?;
    assert!(json.contains("\"version\": 1"));
    assert!(json.contains("\"review-123\""));
    assert!(json.contains("\"2025-08-31\""));
    assert!(json.contains("\"Weekly\""));
    assert!(json.contains("\"score_1_to_5\": 4"));
    assert!(json.contains("Double outreach efforts"));
    assert!(json.contains("Leads lagging behind target"));

    // Test deserialization
    let deserialized: ReviewData = serde_json::from_str(&json)?;
    assert_eq!(deserialized.version, 1);
    assert_eq!(deserialized.review.id, "review-123");
    assert_eq!(deserialized.review.date, NaiveDate::from_ymd_opt(2025, 8, 31).unwrap());
    assert_eq!(deserialized.review.score_1_to_5, 4);
    assert_eq!(deserialized.review.decisions.len(), 2);

    // Test other period types
    review.period = ReviewPeriod::Monthly;
    let json2 = serde_json::to_string(&review)?;
    assert!(json2.contains("\"Monthly\""));

    review.period = ReviewPeriod::Quarterly;
    let json3 = serde_json::to_string(&review)?;
    assert!(json3.contains("\"Quarterly\""));

    Ok(())
}

#[test]
fn test_iso_week_file_naming() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Test various dates and their ISO week representations
    let test_cases = vec![
        // Date -> (Year, Week)
        (NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), (2025, 1)),   // Wed, W01
        (NaiveDate::from_ymd_opt(2025, 1, 6).unwrap(), (2025, 2)),   // Mon, W02
        (NaiveDate::from_ymd_opt(2025, 8, 31).unwrap(), (2025, 35)), // Sun, W35
        (NaiveDate::from_ymd_opt(2025, 12, 29).unwrap(), (2026, 1)), // Mon, W01 of next year
    ];

    for (date, expected_week) in test_cases {
        let review = Review::new(date);
        
        // Get ISO week from the date
        let iso_week = date.iso_week();
        let week_tuple = (iso_week.year(), iso_week.week());
        assert_eq!(week_tuple, expected_week);
        
        // Save with ISO week
        let path = save_review(week_tuple, &review, &config)?;
        
        // Check filename format
        let filename = path.file_name().unwrap().to_string_lossy();
        let expected_filename = format!("{}-W{:02}.json", expected_week.0, expected_week.1);
        assert_eq!(filename, expected_filename);
        
        // Verify file was created in reviews directory
        assert!(path.exists());
        assert!(path.parent().unwrap().ends_with("reviews"));
    }

    Ok(())
}

#[test]
fn test_review_round_trip() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;
    
    // Create a review with all fields populated
    let date = NaiveDate::from_ymd_opt(2025, 8, 31).unwrap();
    let mut review = Review::new(date);
    review.notes = Some("Weekly retrospective notes".to_string());
    review.score_1_to_5 = 5;
    review.decisions = vec![
        Decision {
            summary: "Increase focus on testing".to_string(),
            objective_id: Some("obj-123".to_string()),
            indicator_id: Some("ind-456".to_string()),
            rationale: Some("Quality metrics declining".to_string()),
        },
    ];

    // Get ISO week for the date
    let iso_week = date.iso_week();
    let week_tuple = (iso_week.year(), iso_week.week());

    // Save the review
    let save_path = save_review(week_tuple, &review, &config)?;
    assert!(save_path.exists());

    // Load it back
    let loaded = load_review(week_tuple, &config)?;
    assert!(loaded.is_some());
    
    let loaded_review = loaded.unwrap();
    assert_eq!(loaded_review.id, review.id);
    assert_eq!(loaded_review.date, review.date);
    assert_eq!(loaded_review.period, ReviewPeriod::Weekly);
    assert_eq!(loaded_review.notes, review.notes);
    assert_eq!(loaded_review.score_1_to_5, 5);
    assert_eq!(loaded_review.decisions.len(), 1);
    assert_eq!(loaded_review.decisions[0].summary, "Increase focus on testing");

    Ok(())
}

#[test]
fn test_load_nonexistent_review() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Try to load a review that doesn't exist
    let week_tuple = (2025, 40);
    let loaded = load_review(week_tuple, &config)?;
    assert!(loaded.is_none());

    Ok(())
}

#[test]
fn test_review_creation_helper() -> Result<()> {
    let date = NaiveDate::from_ymd_opt(2025, 8, 31).unwrap();
    let review = Review::new(date);

    assert!(!review.id.is_empty());
    assert_eq!(review.date, date);
    assert_eq!(review.period, ReviewPeriod::Weekly);
    assert!(review.notes.is_none());
    assert_eq!(review.score_1_to_5, 3); // Default middle score
    assert_eq!(review.decisions.len(), 0);

    // Verify UUID format
    assert!(review.id.len() >= 32);

    Ok(())
}

#[test]
fn test_multiple_weeks_storage() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Create reviews for multiple weeks
    let weeks = vec![
        (NaiveDate::from_ymd_opt(2025, 8, 10).unwrap(), "Week 32 review"),
        (NaiveDate::from_ymd_opt(2025, 8, 17).unwrap(), "Week 33 review"),
        (NaiveDate::from_ymd_opt(2025, 8, 24).unwrap(), "Week 34 review"),
        (NaiveDate::from_ymd_opt(2025, 8, 31).unwrap(), "Week 35 review"),
    ];

    for (date, note) in &weeks {
        let mut review = Review::new(*date);
        review.notes = Some(note.to_string());
        
        let iso_week = date.iso_week();
        let week_tuple = (iso_week.year(), iso_week.week());
        
        save_review(week_tuple, &review, &config)?;
    }

    // Verify all reviews can be loaded back
    for (date, expected_note) in &weeks {
        let iso_week = date.iso_week();
        let week_tuple = (iso_week.year(), iso_week.week());
        
        let loaded = load_review(week_tuple, &config)?;
        assert!(loaded.is_some());
        
        let review = loaded.unwrap();
        assert_eq!(review.notes, Some(expected_note.to_string()));
    }

    // Check that the reviews directory contains 4 files
    let reviews_dir = std::path::Path::new(&config.data_root).join("reviews");
    let entries = fs::read_dir(reviews_dir)?;
    let file_count = entries.count();
    assert_eq!(file_count, 4);

    Ok(())
}

#[test]
fn test_review_score_validation() -> Result<()> {
    // Test that scores are in valid range
    let mut review = Review::new(NaiveDate::from_ymd_opt(2025, 8, 31).unwrap());
    
    // Valid scores
    for score in 1..=5 {
        review.score_1_to_5 = score;
        let json = serde_json::to_string(&review)?;
        let deserialized: Review = serde_json::from_str(&json)?;
        assert_eq!(deserialized.score_1_to_5, score);
    }

    Ok(())
}

#[test]
fn test_decision_with_optional_fields() -> Result<()> {
    // Test decisions with various combinations of optional fields
    let decisions = vec![
        Decision {
            summary: "All fields populated".to_string(),
            objective_id: Some("obj-1".to_string()),
            indicator_id: Some("ind-1".to_string()),
            rationale: Some("Full rationale".to_string()),
        },
        Decision {
            summary: "Only summary".to_string(),
            objective_id: None,
            indicator_id: None,
            rationale: None,
        },
        Decision {
            summary: "With objective only".to_string(),
            objective_id: Some("obj-2".to_string()),
            indicator_id: None,
            rationale: None,
        },
        Decision {
            summary: "With indicator only".to_string(),
            objective_id: None,
            indicator_id: Some("ind-2".to_string()),
            rationale: None,
        },
    ];

    for decision in &decisions {
        let json = serde_json::to_string(&decision)?;
        let deserialized: Decision = serde_json::from_str(&json)?;
        assert_eq!(deserialized.summary, decision.summary);
        assert_eq!(deserialized.objective_id, decision.objective_id);
        assert_eq!(deserialized.indicator_id, decision.indicator_id);
        assert_eq!(deserialized.rationale, decision.rationale);
    }

    Ok(())
}