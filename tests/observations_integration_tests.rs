use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use focusfive::data::{
    append_observation, load_or_create_indicators, read_observations_range, save_indicators,
};
use focusfive::models::{
    Config, IndicatorDef, IndicatorKind, IndicatorUnit, IndicatorsData, Observation,
};
use tempfile::TempDir;

#[test]
fn test_two_weeks_observations_with_7day_window() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Create an indicator first
    let indicator = IndicatorDef::new(
        "Daily Sales".to_string(),
        IndicatorKind::Leading,
        IndicatorUnit::Dollars,
    );
    let indicator_id = indicator.id.clone();

    let indicators = IndicatorsData {
        version: 1,
        indicators: vec![indicator],
    };
    save_indicators(&indicators, &config)?;

    // Generate 2 weeks of observations (14 days)
    let start_date = NaiveDate::from_ymd_opt(2025, 8, 15).unwrap();
    for day_offset in 0..14 {
        let date = start_date
            .checked_add_signed(chrono::Duration::days(day_offset))
            .unwrap();

        // Vary the values for testing
        let value = 1000.0 + (day_offset as f64 * 50.0);

        let obs = Observation::new(indicator_id.clone(), date, value, IndicatorUnit::Dollars);

        append_observation(&obs, &config)?;
    }

    // Read 7-day window from the middle
    let window_start = NaiveDate::from_ymd_opt(2025, 8, 19).unwrap();
    let window_end = NaiveDate::from_ymd_opt(2025, 8, 25).unwrap();

    let window_obs = read_observations_range(window_start, window_end, &config)?;

    // Should have exactly 7 observations
    assert_eq!(window_obs.len(), 7);

    // Verify dates are correct
    assert_eq!(
        window_obs[0].when,
        NaiveDate::from_ymd_opt(2025, 8, 19).unwrap()
    );
    assert_eq!(
        window_obs[6].when,
        NaiveDate::from_ymd_opt(2025, 8, 25).unwrap()
    );

    // Verify values (day 4 = 1200, day 5 = 1250, ..., day 10 = 1500)
    assert_eq!(window_obs[0].value, 1200.0); // Day 4 (Aug 19)
    assert_eq!(window_obs[3].value, 1350.0); // Day 7 (Aug 22)
    assert_eq!(window_obs[6].value, 1500.0); // Day 10 (Aug 25)

    Ok(())
}

#[test]
fn test_multiple_indicators_observations() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Create multiple indicators
    let ind1 = IndicatorDef::new(
        "Website Traffic".to_string(),
        IndicatorKind::Leading,
        IndicatorUnit::Count,
    );
    let ind1_id = ind1.id.clone();

    let ind2 = IndicatorDef::new(
        "Conversion Rate".to_string(),
        IndicatorKind::Lagging,
        IndicatorUnit::Percent,
    );
    let ind2_id = ind2.id.clone();

    let ind3 = IndicatorDef::new(
        "Support Tickets".to_string(),
        IndicatorKind::Leading,
        IndicatorUnit::Count,
    );
    let ind3_id = ind3.id.clone();

    let indicators = IndicatorsData {
        version: 1,
        indicators: vec![ind1, ind2, ind3],
    };
    save_indicators(&indicators, &config)?;

    // Add observations for each indicator
    let dates = vec![
        NaiveDate::from_ymd_opt(2025, 8, 28).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 29).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 30).unwrap(),
    ];

    for date in &dates {
        // Website traffic
        let obs1 = Observation::new(
            ind1_id.clone(),
            *date,
            1000.0 + date.day() as f64 * 10.0,
            IndicatorUnit::Count,
        );
        append_observation(&obs1, &config)?;

        // Conversion rate
        let obs2 = Observation::new(
            ind2_id.clone(),
            *date,
            2.5 + date.day() as f64 * 0.1,
            IndicatorUnit::Percent,
        );
        append_observation(&obs2, &config)?;

        // Support tickets
        let obs3 = Observation::new(
            ind3_id.clone(),
            *date,
            5.0 + date.day() as f64,
            IndicatorUnit::Count,
        );
        append_observation(&obs3, &config)?;
    }

    // Read all observations
    let all_obs = read_observations_range(dates[0], dates[dates.len() - 1], &config)?;

    // Should have 9 observations (3 indicators × 3 days)
    assert_eq!(all_obs.len(), 9);

    // Group by indicator and verify
    let traffic_obs: Vec<_> = all_obs
        .iter()
        .filter(|o| o.indicator_id == ind1_id)
        .collect();
    assert_eq!(traffic_obs.len(), 3);

    let conversion_obs: Vec<_> = all_obs
        .iter()
        .filter(|o| o.indicator_id == ind2_id)
        .collect();
    assert_eq!(conversion_obs.len(), 3);

    let ticket_obs: Vec<_> = all_obs
        .iter()
        .filter(|o| o.indicator_id == ind3_id)
        .collect();
    assert_eq!(ticket_obs.len(), 3);

    Ok(())
}

#[test]
fn test_observations_with_notes_and_links() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Create indicator
    let indicator = IndicatorDef::new(
        "Code Reviews Completed".to_string(),
        IndicatorKind::Leading,
        IndicatorUnit::Count,
    );
    let indicator_id = indicator.id.clone();

    let indicators = IndicatorsData {
        version: 1,
        indicators: vec![indicator],
    };
    save_indicators(&indicators, &config)?;

    // Create observations with varying metadata
    let mut obs1 = Observation::new(
        indicator_id.clone(),
        NaiveDate::from_ymd_opt(2025, 8, 28).unwrap(),
        3.0,
        IndicatorUnit::Count,
    );
    obs1.action_id = Some("action-abc".to_string());
    obs1.note = Some("Reviewed 3 PRs this morning".to_string());
    append_observation(&obs1, &config)?;

    let mut obs2 = Observation::new(
        indicator_id.clone(),
        NaiveDate::from_ymd_opt(2025, 8, 29).unwrap(),
        5.0,
        IndicatorUnit::Count,
    );
    obs2.action_id = Some("action-def".to_string());
    obs2.note = None;
    append_observation(&obs2, &config)?;

    let obs3 = Observation::new(
        indicator_id,
        NaiveDate::from_ymd_opt(2025, 8, 30).unwrap(),
        2.0,
        IndicatorUnit::Count,
    );
    // No action_id or note
    append_observation(&obs3, &config)?;

    // Read back and verify metadata preserved
    let all_obs = read_observations_range(
        NaiveDate::from_ymd_opt(2025, 8, 28).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 30).unwrap(),
        &config,
    )?;

    assert_eq!(all_obs.len(), 3);

    // First observation has both action_id and note
    assert_eq!(all_obs[0].action_id, Some("action-abc".to_string()));
    assert_eq!(
        all_obs[0].note,
        Some("Reviewed 3 PRs this morning".to_string())
    );

    // Second has only action_id
    assert_eq!(all_obs[1].action_id, Some("action-def".to_string()));
    assert!(all_obs[1].note.is_none());

    // Third has neither
    assert!(all_obs[2].action_id.is_none());
    assert!(all_obs[2].note.is_none());

    Ok(())
}

#[test]
fn test_large_observation_dataset() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Create indicator
    let indicator = IndicatorDef::new(
        "Hourly Metrics".to_string(),
        IndicatorKind::Leading,
        IndicatorUnit::Count,
    );
    let indicator_id = indicator.id.clone();

    let indicators = IndicatorsData {
        version: 1,
        indicators: vec![indicator],
    };
    save_indicators(&indicators, &config)?;

    // Generate 365 days of observations
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    for day in 0..365 {
        let date = start_date
            .checked_add_signed(chrono::Duration::days(day))
            .unwrap();

        let obs = Observation::new(
            indicator_id.clone(),
            date,
            (day as f64) * 2.0,
            IndicatorUnit::Count,
        );

        append_observation(&obs, &config)?;
    }

    // Read January only (31 days)
    let jan_obs = read_observations_range(
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
        &config,
    )?;
    assert_eq!(jan_obs.len(), 31);

    // Read Q1 (90 days)
    let q1_obs = read_observations_range(
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
        &config,
    )?;
    assert_eq!(q1_obs.len(), 90);

    // Read single day from middle of year
    let single_day = read_observations_range(
        NaiveDate::from_ymd_opt(2025, 7, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 7, 1).unwrap(),
        &config,
    )?;
    assert_eq!(single_day.len(), 1);

    // July 1 is day 182 (31+28+31+30+31+30+1)
    // So the value should be 181 * 2 = 362
    assert_eq!(single_day[0].value, 362.0);

    Ok(())
}

#[test]
fn test_empty_date_ranges() -> Result<()> {
    let temp = TempDir::new()?;
    std::env::set_var("HOME", temp.path());

    let config = Config::new()?;

    // Create some observations
    let indicator_id = "test-ind".to_string();
    for day in 10..20 {
        let obs = Observation::new(
            indicator_id.clone(),
            NaiveDate::from_ymd_opt(2025, 8, day).unwrap(),
            day as f64,
            IndicatorUnit::Count,
        );
        append_observation(&obs, &config)?;
    }

    // Query before all data
    let before = read_observations_range(
        NaiveDate::from_ymd_opt(2025, 8, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 9).unwrap(),
        &config,
    )?;
    assert_eq!(before.len(), 0);

    // Query after all data
    let after = read_observations_range(
        NaiveDate::from_ymd_opt(2025, 8, 21).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 31).unwrap(),
        &config,
    )?;
    assert_eq!(after.len(), 0);

    // Query with inverted range (should return empty)
    let inverted = read_observations_range(
        NaiveDate::from_ymd_opt(2025, 8, 20).unwrap(),
        NaiveDate::from_ymd_opt(2025, 8, 10).unwrap(),
        &config,
    )?;
    assert_eq!(inverted.len(), 0);

    Ok(())
}
