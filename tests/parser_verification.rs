use chrono::NaiveDate;
use focusfive::data;
use focusfive::models::Config;

#[test]
fn test_parser_with_real_file() {
    // Test with the sample goals file we created
    let test_content = r#"# January 17, 2025

## Work (Goal: Complete Phase 2)
- [ ] Design TUI layout with ratatui
- [ ] Implement keyboard navigation
- [ ] Connect UI to data layer

## Health (Goal: Stay active)
- [x] Morning walk
- [ ] Healthy lunch
- [ ] Evening stretches

## Family (Goal: Quality time)
- [x] Breakfast together
- [ ] Game night
- [ ] Weekend planning"#;

    // Create a temp file
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("2025-01-17.md");
    std::fs::write(&file_path, test_content).unwrap();

    // Parse it
    let goals = data::parse_markdown(&std::fs::read_to_string(&file_path).unwrap()).unwrap();

    // Verify date
    assert_eq!(goals.date, NaiveDate::from_ymd_opt(2025, 1, 17).unwrap());

    // Verify Work outcome
    assert_eq!(goals.work.goal, Some("Complete Phase 2".to_string()));
    assert_eq!(goals.work.actions[0].text, "Design TUI layout with ratatui");
    assert_eq!(goals.work.actions[0].completed, false);
    assert_eq!(goals.work.actions[1].text, "Implement keyboard navigation");
    assert_eq!(goals.work.actions[1].completed, false);
    assert_eq!(goals.work.actions[2].text, "Connect UI to data layer");
    assert_eq!(goals.work.actions[2].completed, false);

    // Verify Health outcome
    assert_eq!(goals.health.goal, Some("Stay active".to_string()));
    assert_eq!(goals.health.actions[0].text, "Morning walk");
    assert_eq!(goals.health.actions[0].completed, true);
    assert_eq!(goals.health.actions[1].text, "Healthy lunch");
    assert_eq!(goals.health.actions[1].completed, false);
    assert_eq!(goals.health.actions[2].text, "Evening stretches");
    assert_eq!(goals.health.actions[2].completed, false);

    // Verify Family outcome
    assert_eq!(goals.family.goal, Some("Quality time".to_string()));
    assert_eq!(goals.family.actions[0].text, "Breakfast together");
    assert_eq!(goals.family.actions[0].completed, true);
    assert_eq!(goals.family.actions[1].text, "Game night");
    assert_eq!(goals.family.actions[1].completed, false);
    assert_eq!(goals.family.actions[2].text, "Weekend planning");
    assert_eq!(goals.family.actions[2].completed, false);

    // Verify completion count
    let mut completed = 0;
    for outcome in [&goals.work, &goals.health, &goals.family] {
        for action in &outcome.actions {
            if action.completed {
                completed += 1;
            }
        }
    }
    assert_eq!(completed, 2); // Morning walk and Breakfast together

    println!("✅ Parser verification passed!");
}
