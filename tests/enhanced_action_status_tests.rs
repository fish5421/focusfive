use chrono::Utc;
use focusfive::models::{Action, ActionOrigin, ActionStatus};

#[cfg(test)]
mod action_status_tests {
    use super::*;

    #[test]
    fn test_action_new_has_correct_defaults() {
        let action = Action::new("Test action".to_string());

        assert_eq!(action.status, ActionStatus::Planned);
        assert_eq!(action.origin, ActionOrigin::Manual);
        assert_eq!(action.completed, false); // Should be false for Planned status
        assert_eq!(action.text, "Test action");
        assert!(!action.id.is_empty()); // Should have a UUID

        // Check that timestamps are recent (within last second)
        let now = Utc::now();
        let time_diff = now.signed_duration_since(action.created);
        assert!(time_diff.num_seconds() < 2);

        let time_diff = now.signed_duration_since(action.modified);
        assert!(time_diff.num_seconds() < 2);
    }

    #[test]
    fn test_action_new_empty_has_correct_defaults() {
        let action = Action::new_empty();

        assert_eq!(action.status, ActionStatus::Planned);
        assert_eq!(action.origin, ActionOrigin::Manual);
        assert_eq!(action.completed, false);
        assert_eq!(action.text, "");
        assert!(!action.id.is_empty());
    }

    #[test]
    fn test_action_new_with_origin() {
        let action = Action::new_with_origin("Template action".to_string(), ActionOrigin::Template);

        assert_eq!(action.status, ActionStatus::Planned);
        assert_eq!(action.origin, ActionOrigin::Template);
        assert_eq!(action.completed, false);
        assert_eq!(action.text, "Template action");
    }

    #[test]
    fn test_action_from_markdown_completed() {
        let action = Action::from_markdown("Completed task".to_string(), true);

        assert_eq!(action.status, ActionStatus::Done);
        assert_eq!(action.origin, ActionOrigin::Manual);
        assert_eq!(action.completed, true);
        assert_eq!(action.text, "Completed task");
    }

    #[test]
    fn test_action_from_markdown_incomplete() {
        let action = Action::from_markdown("Incomplete task".to_string(), false);

        assert_eq!(action.status, ActionStatus::Planned);
        assert_eq!(action.origin, ActionOrigin::Manual);
        assert_eq!(action.completed, false);
        assert_eq!(action.text, "Incomplete task");
    }

    #[test]
    fn test_status_cycling() {
        let mut action = Action::new("Test".to_string());

        // Start at Planned
        assert_eq!(action.status, ActionStatus::Planned);
        assert_eq!(action.completed, false);

        // Cycle to InProgress
        action.cycle_status();
        assert_eq!(action.status, ActionStatus::InProgress);
        assert_eq!(action.completed, false);

        // Cycle to Done
        action.cycle_status();
        assert_eq!(action.status, ActionStatus::Done);
        assert_eq!(action.completed, true); // Should be true now!

        // Cycle to Skipped
        action.cycle_status();
        assert_eq!(action.status, ActionStatus::Skipped);
        assert_eq!(action.completed, false); // Should be false again

        // Cycle to Blocked
        action.cycle_status();
        assert_eq!(action.status, ActionStatus::Blocked);
        assert_eq!(action.completed, false);

        // Cycle back to Planned
        action.cycle_status();
        assert_eq!(action.status, ActionStatus::Planned);
        assert_eq!(action.completed, false);
    }

    #[test]
    fn test_set_status_syncs_completed() {
        let mut action = Action::new("Test".to_string());

        // Set to Done - completed should be true
        action.set_status(ActionStatus::Done);
        assert_eq!(action.status, ActionStatus::Done);
        assert_eq!(action.completed, true);

        // Set to InProgress - completed should be false
        action.set_status(ActionStatus::InProgress);
        assert_eq!(action.status, ActionStatus::InProgress);
        assert_eq!(action.completed, false);

        // Set to Skipped - completed should be false
        action.set_status(ActionStatus::Skipped);
        assert_eq!(action.status, ActionStatus::Skipped);
        assert_eq!(action.completed, false);

        // Set to Blocked - completed should be false
        action.set_status(ActionStatus::Blocked);
        assert_eq!(action.status, ActionStatus::Blocked);
        assert_eq!(action.completed, false);

        // Set to Planned - completed should be false
        action.set_status(ActionStatus::Planned);
        assert_eq!(action.status, ActionStatus::Planned);
        assert_eq!(action.completed, false);
    }

    #[test]
    fn test_status_char_display() {
        let _action = Action::new("Test".to_string());

        // Test all status characters using Debug format since Display is not implemented
        assert_eq!(format!("{:?}", ActionStatus::Planned), "Planned");
        assert_eq!(format!("{:?}", ActionStatus::InProgress), "InProgress");
        assert_eq!(format!("{:?}", ActionStatus::Done), "Done");
        assert_eq!(format!("{:?}", ActionStatus::Skipped), "Skipped");
        assert_eq!(format!("{:?}", ActionStatus::Blocked), "Blocked");
    }

    #[test]
    fn test_modified_timestamp_updates() {
        let mut action = Action::new("Test".to_string());
        let original_modified = action.modified;

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Cycle status should update modified timestamp
        action.cycle_status();
        assert!(action.modified > original_modified);

        let second_modified = action.modified;
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Set status should also update modified timestamp
        action.set_status(ActionStatus::Blocked);
        assert!(action.modified > second_modified);
    }

    #[test]
    fn test_backward_compatibility_with_completed_field() {
        // Test that completed field is properly synced for all statuses
        let mut action = Action::new("Test".to_string());

        // Only Done status should set completed = true
        for status in [
            ActionStatus::Planned,
            ActionStatus::InProgress,
            ActionStatus::Skipped,
            ActionStatus::Blocked,
        ] {
            action.set_status(status);
            assert_eq!(
                action.completed, false,
                "Status {:?} should have completed=false",
                status
            );
        }

        action.set_status(ActionStatus::Done);
        assert_eq!(
            action.completed, true,
            "Status Done should have completed=true"
        );
    }

    #[test]
    fn test_text_truncation_with_new_fields() {
        let long_text = "a".repeat(600); // Exceeds MAX_ACTION_LENGTH (500)
        let action = Action::new(long_text);

        assert_eq!(action.text.len(), 500);
        assert_eq!(action.status, ActionStatus::Planned);
        assert_eq!(action.origin, ActionOrigin::Manual);
        assert_eq!(action.completed, false);
    }

    #[test]
    fn test_serialization_deserialization() {
        use serde_json;

        let mut action = Action::new("Test action".to_string());
        action.set_status(ActionStatus::InProgress);
        action.origin = ActionOrigin::Template;

        // Serialize
        let serialized = serde_json::to_string(&action).expect("Should serialize");

        // Deserialize
        let deserialized: Action = serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(deserialized.text, "Test action");
        assert_eq!(deserialized.status, ActionStatus::InProgress);
        assert_eq!(deserialized.origin, ActionOrigin::Template);
        assert_eq!(deserialized.completed, false); // InProgress should be false
        assert_eq!(deserialized.id, action.id);
        assert_eq!(deserialized.created, action.created);
        assert_eq!(deserialized.modified, action.modified);
    }

    #[test]
    fn test_action_origins_are_preserved() {
        let manual_action = Action::new_with_origin("Manual".to_string(), ActionOrigin::Manual);
        let template_action =
            Action::new_with_origin("Template".to_string(), ActionOrigin::Template);
        let carryover_action =
            Action::new_with_origin("CarryOver".to_string(), ActionOrigin::CarryOver);

        assert_eq!(manual_action.origin, ActionOrigin::Manual);
        assert_eq!(template_action.origin, ActionOrigin::Template);
        assert_eq!(carryover_action.origin, ActionOrigin::CarryOver);

        // Status cycling shouldn't change origin
        let mut test_action = template_action;
        test_action.cycle_status();
        test_action.cycle_status();
        assert_eq!(test_action.origin, ActionOrigin::Template);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use focusfive::models::{Outcome, OutcomeType};

    #[test]
    fn test_outcome_handles_enhanced_actions() {
        let outcome = Outcome::new(OutcomeType::Work);

        // All actions should start as empty with default status/origin
        for action in &outcome.actions {
            assert_eq!(action.status, ActionStatus::Planned);
            assert_eq!(action.origin, ActionOrigin::Manual);
            assert_eq!(action.completed, false);
            assert!(action.text.is_empty());
        }
    }
}
