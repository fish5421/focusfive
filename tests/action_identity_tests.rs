use anyhow::Result;
use chrono::{NaiveDate, Utc};
use focusfive::data::{generate_markdown, parse_markdown};
use focusfive::models::{Action, DailyGoals, DayMeta};

#[test]
fn test_action_new_generates_id() -> Result<()> {
    let action = Action::new("Test action".to_string());

    // Should have a UUID
    assert!(!action.id.is_empty());
    assert!(action.id.len() >= 32); // UUIDs are at least 32 chars

    // Should have text
    assert_eq!(action.text, "Test action");

    // Should not be completed by default
    assert!(!action.completed);

    // Should have timestamps
    assert!(action.created <= Utc::now());
    assert!(action.modified <= Utc::now());

    Ok(())
}

#[test]
fn test_action_new_empty() -> Result<()> {
    let action = Action::new_empty();

    // Should have a UUID
    assert!(!action.id.is_empty());

    // Should have empty text
    assert!(action.text.is_empty());

    // Should not be completed
    assert!(!action.completed);

    Ok(())
}

#[test]
fn test_action_from_markdown() -> Result<()> {
    let action_complete = Action::from_markdown("Completed task".to_string(), true);
    let action_incomplete = Action::from_markdown("Pending task".to_string(), false);

    // Both should have unique IDs
    assert!(!action_complete.id.is_empty());
    assert!(!action_incomplete.id.is_empty());
    assert_ne!(action_complete.id, action_incomplete.id);

    // Should preserve completion status
    assert!(action_complete.completed);
    assert!(!action_incomplete.completed);

    // Should have correct text
    assert_eq!(action_complete.text, "Completed task");
    assert_eq!(action_incomplete.text, "Pending task");

    Ok(())
}

#[test]
fn test_action_text_truncation() -> Result<()> {
    // Create a string longer than MAX_ACTION_LENGTH (500 chars)
    let long_text = "x".repeat(600);
    let action = Action::new(long_text.clone());

    // Should be truncated to MAX_ACTION_LENGTH
    assert_eq!(action.text.len(), 500);
    assert!(action.text.starts_with("xxxx"));

    Ok(())
}

#[test]
fn test_action_ids_unique() -> Result<()> {
    let action1 = Action::new("Task 1".to_string());
    let action2 = Action::new("Task 2".to_string());
    let action3 = Action::new_empty();

    // All should have unique IDs
    assert_ne!(action1.id, action2.id);
    assert_ne!(action2.id, action3.id);
    assert_ne!(action1.id, action3.id);

    Ok(())
}

#[test]
fn test_markdown_roundtrip_preserves_completion() -> Result<()> {
    let date = NaiveDate::from_ymd_opt(2025, 8, 29).unwrap();
    let mut goals = DailyGoals::new(date);

    // Create actions with specific completion states
    goals.work.actions = vec![
        Action::from_markdown("Work task 1".to_string(), true),
        Action::from_markdown("Work task 2".to_string(), false),
        Action::from_markdown("Work task 3".to_string(), false),
    ];

    // Generate markdown and parse it back
    let markdown = generate_markdown(&goals);
    let parsed = parse_markdown(&markdown)?;

    // Note: IDs are NOT preserved through markdown (only through metadata)
    // Each parse creates new IDs, which is expected
    assert!(!parsed.work.actions[0].id.is_empty());
    assert!(!parsed.work.actions[1].id.is_empty());
    assert!(!parsed.work.actions[2].id.is_empty());

    // But completion status and text should be preserved
    assert!(parsed.work.actions[0].completed);
    assert!(!parsed.work.actions[1].completed);
    assert!(!parsed.work.actions[2].completed);

    assert_eq!(parsed.work.actions[0].text, "Work task 1");
    assert_eq!(parsed.work.actions[1].text, "Work task 2");
    assert_eq!(parsed.work.actions[2].text, "Work task 3");

    Ok(())
}

#[test]
fn test_metadata_reconciliation_uses_action_ids() -> Result<()> {
    let date = NaiveDate::from_ymd_opt(2025, 8, 29).unwrap();
    let mut goals = DailyGoals::new(date);

    // Create actions with known IDs
    let action1 = Action::from_markdown("Task 1".to_string(), true);
    let action2 = Action::from_markdown("Task 2".to_string(), false);
    let action3 = Action::from_markdown("Task 3".to_string(), false);

    let id1 = action1.id.clone();
    let id2 = action2.id.clone();
    let id3 = action3.id.clone();

    goals.work.actions = vec![action1, action2, action3];

    // Create metadata from goals
    let meta = DayMeta::from_goals(&goals);

    // Metadata should use the same IDs as actions
    assert_eq!(meta.work[0].id, id1);
    assert_eq!(meta.work[1].id, id2);
    assert_eq!(meta.work[2].id, id3);

    // Status should match completion
    assert_eq!(
        meta.work[0].status,
        focusfive::models::ActionMetaStatus::Done
    );
    assert_eq!(
        meta.work[1].status,
        focusfive::models::ActionMetaStatus::Planned
    );
    assert_eq!(
        meta.work[2].status,
        focusfive::models::ActionMetaStatus::Planned
    );

    Ok(())
}

#[test]
fn test_metadata_reconciliation_on_action_add() -> Result<()> {
    let date = NaiveDate::from_ymd_opt(2025, 8, 29).unwrap();
    let mut goals = DailyGoals::new(date);

    // Start with 2 actions
    goals.work.actions = vec![
        Action::from_markdown("Task 1".to_string(), false),
        Action::from_markdown("Task 2".to_string(), false),
    ];

    let mut meta = DayMeta::from_goals(&goals);
    assert_eq!(meta.work.len(), 2);

    // Add a third action
    goals
        .work
        .actions
        .push(Action::from_markdown("Task 3".to_string(), true));
    let new_action_id = goals.work.actions[2].id.clone();

    // Reconcile
    meta.reconcile_with_goals(&goals);

    // Should now have 3 metadata entries
    assert_eq!(meta.work.len(), 3);

    // New metadata should use the new action's ID
    assert_eq!(meta.work[2].id, new_action_id);

    // New metadata status should match completion
    assert_eq!(
        meta.work[2].status,
        focusfive::models::ActionMetaStatus::Done
    );

    Ok(())
}

#[test]
fn test_metadata_reconciliation_on_action_remove() -> Result<()> {
    let date = NaiveDate::from_ymd_opt(2025, 8, 29).unwrap();
    let mut goals = DailyGoals::new(date);

    // Start with 3 actions
    goals.work.actions = vec![
        Action::from_markdown("Task 1".to_string(), false),
        Action::from_markdown("Task 2".to_string(), false),
        Action::from_markdown("Task 3".to_string(), false),
    ];

    let mut meta = DayMeta::from_goals(&goals);
    assert_eq!(meta.work.len(), 3);

    // Remove the last action
    goals.work.actions.pop();

    // Reconcile
    meta.reconcile_with_goals(&goals);

    // Should now have 2 metadata entries
    assert_eq!(meta.work.len(), 2);

    Ok(())
}

#[test]
fn test_metadata_status_sync_with_completion() -> Result<()> {
    let date = NaiveDate::from_ymd_opt(2025, 8, 29).unwrap();
    let mut goals = DailyGoals::new(date);

    // Create action that's not completed
    goals.work.actions = vec![Action::from_markdown("Task 1".to_string(), false)];

    let mut meta = DayMeta::from_goals(&goals);
    assert_eq!(
        meta.work[0].status,
        focusfive::models::ActionMetaStatus::Planned
    );

    // Mark action as completed
    goals.work.actions[0].completed = true;

    // Reconcile
    meta.reconcile_with_goals(&goals);

    // Status should update to Done
    assert_eq!(
        meta.work[0].status,
        focusfive::models::ActionMetaStatus::Done
    );

    // Unmark completion
    goals.work.actions[0].completed = false;

    // Reconcile again
    meta.reconcile_with_goals(&goals);

    // Status should revert to Planned
    assert_eq!(
        meta.work[0].status,
        focusfive::models::ActionMetaStatus::Planned
    );

    Ok(())
}

#[test]
fn test_metadata_preserves_in_progress_status() -> Result<()> {
    let date = NaiveDate::from_ymd_opt(2025, 8, 29).unwrap();
    let mut goals = DailyGoals::new(date);

    goals.work.actions = vec![Action::from_markdown("Task 1".to_string(), false)];

    let mut meta = DayMeta::from_goals(&goals);

    // Manually set to InProgress
    meta.work[0].status = focusfive::models::ActionMetaStatus::InProgress;

    // Even if action is not completed, InProgress should be preserved
    meta.reconcile_with_goals(&goals);
    assert_eq!(
        meta.work[0].status,
        focusfive::models::ActionMetaStatus::InProgress
    );

    Ok(())
}

#[test]
fn test_metadata_preserves_blocked_status() -> Result<()> {
    let date = NaiveDate::from_ymd_opt(2025, 8, 29).unwrap();
    let mut goals = DailyGoals::new(date);

    goals.work.actions = vec![Action::from_markdown("Task 1".to_string(), false)];

    let mut meta = DayMeta::from_goals(&goals);

    // Manually set to Blocked
    meta.work[0].status = focusfive::models::ActionMetaStatus::Blocked;

    // Even if action is not completed, Blocked should be preserved
    meta.reconcile_with_goals(&goals);
    assert_eq!(
        meta.work[0].status,
        focusfive::models::ActionMetaStatus::Blocked
    );

    Ok(())
}
