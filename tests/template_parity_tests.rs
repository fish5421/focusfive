use chrono::Local;
use focusfive::models::{Action, ActionOrigin, ActionStatus, ActionTemplates, Config, DailyGoals, Outcome, OutcomeType};
use focusfive::data::{load_or_create_templates, save_templates};

#[cfg(test)]
mod template_parity_tests {
    use super::*;

    #[test]
    fn test_template_with_5_actions_creation_and_storage() {
        let mut templates = ActionTemplates::new();
        
        // Create a template with 5 actions
        let template_actions = vec![
            "Action 1".to_string(),
            "Action 2".to_string(), 
            "Action 3".to_string(),
            "Action 4".to_string(),
            "Action 5".to_string(),
        ];
        
        templates.add_template("Five Action Template".to_string(), template_actions.clone());
        
        // Verify template was stored correctly
        let stored_template = templates.get_template("Five Action Template").unwrap();
        assert_eq!(stored_template.len(), 5);
        assert_eq!(*stored_template, template_actions);
    }

    #[test]
    fn test_template_truncation_at_5_actions() {
        let mut templates = ActionTemplates::new();
        
        // Try to create a template with 7 actions
        let template_actions = vec![
            "Action 1".to_string(),
            "Action 2".to_string(),
            "Action 3".to_string(),
            "Action 4".to_string(),
            "Action 5".to_string(),
            "Action 6".to_string(),
            "Action 7".to_string(),
        ];
        
        templates.add_template("Seven Action Template".to_string(), template_actions);
        
        // Verify template was truncated to 5 actions
        let stored_template = templates.get_template("Seven Action Template").unwrap();
        assert_eq!(stored_template.len(), 5);
        assert_eq!(stored_template[4], "Action 5");
    }

    #[test]
    fn test_apply_5_action_template_to_outcome_with_2_existing_actions() {
        let mut outcome = Outcome::new(OutcomeType::Work);
        
        // Set up outcome with 2 existing actions
        outcome.actions[0].text = "Existing Action 1".to_string();
        outcome.actions[1].text = "Existing Action 2".to_string();
        // Third action remains empty
        
        // Create template with 5 actions
        let mut templates = ActionTemplates::new();
        let template_actions = vec![
            "Template Action 1".to_string(),
            "Template Action 2".to_string(),
            "Template Action 3".to_string(),
            "Template Action 4".to_string(),
            "Template Action 5".to_string(),
        ];
        templates.add_template("Five Actions".to_string(), template_actions);
        
        // Simulate the template application logic
        let template = templates.get_template("Five Actions").unwrap();
        
        // Ensure outcome has enough slots (simulating add_action() calls)
        while outcome.actions.len() < template.len() && outcome.actions.len() < 5 {
            outcome.add_action().unwrap();
        }
        
        // Apply template to empty slots only
        for (i, action_text) in template.iter().enumerate() {
            if i < outcome.actions.len() && outcome.actions[i].text.is_empty() {
                outcome.actions[i].text = action_text.clone();
                outcome.actions[i].origin = ActionOrigin::Template;
                outcome.actions[i].set_status(ActionStatus::Planned);
            }
        }
        
        // Verify final state: should have 5 actions total
        assert_eq!(outcome.actions.len(), 5);
        
        // Verify existing actions weren't overwritten
        assert_eq!(outcome.actions[0].text, "Existing Action 1");
        assert_eq!(outcome.actions[1].text, "Existing Action 2");
        assert_eq!(outcome.actions[0].origin, ActionOrigin::Manual);
        assert_eq!(outcome.actions[1].origin, ActionOrigin::Manual);
        
        // Verify template actions were applied to empty slots
        assert_eq!(outcome.actions[2].text, "Template Action 3");
        assert_eq!(outcome.actions[3].text, "Template Action 4"); 
        assert_eq!(outcome.actions[4].text, "Template Action 5");
        assert_eq!(outcome.actions[2].origin, ActionOrigin::Template);
        assert_eq!(outcome.actions[3].origin, ActionOrigin::Template);
        assert_eq!(outcome.actions[4].origin, ActionOrigin::Template);
        
        // Verify all template actions have correct status
        assert_eq!(outcome.actions[2].status, ActionStatus::Planned);
        assert_eq!(outcome.actions[3].status, ActionStatus::Planned);
        assert_eq!(outcome.actions[4].status, ActionStatus::Planned);
    }

    #[test]
    fn test_template_serialization_with_5_actions() {
        let mut templates = ActionTemplates::new();
        
        // Create templates with varying action counts
        templates.add_template("Small".to_string(), vec!["Action 1".to_string()]);
        templates.add_template("Medium".to_string(), vec![
            "Action 1".to_string(),
            "Action 2".to_string(),
            "Action 3".to_string(),
        ]);
        templates.add_template("Large".to_string(), vec![
            "Action 1".to_string(),
            "Action 2".to_string(),
            "Action 3".to_string(),
            "Action 4".to_string(),
            "Action 5".to_string(),
        ]);
        
        // Serialize and deserialize
        let json = serde_json::to_string(&templates).unwrap();
        let deserialized: ActionTemplates = serde_json::from_str(&json).unwrap();
        
        // Verify all templates preserved correctly
        assert_eq!(deserialized.get_template("Small").unwrap().len(), 1);
        assert_eq!(deserialized.get_template("Medium").unwrap().len(), 3);
        assert_eq!(deserialized.get_template("Large").unwrap().len(), 5);
        
        // Verify action order preserved
        let large_template = deserialized.get_template("Large").unwrap();
        assert_eq!(large_template[0], "Action 1");
        assert_eq!(large_template[4], "Action 5");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::Path;
    
    fn create_test_config() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            goals_dir: temp_dir.path().join("goals").to_string_lossy().to_string(),
            data_root: temp_dir.path().to_string_lossy().to_string(),
        };
        (config, temp_dir)
    }

    #[test]
    fn test_morning_phase_template_application_integration() {
        let (config, _temp_dir) = create_test_config();
        
        // Create a 5-action template and save it
        let mut templates = ActionTemplates::new();
        let morning_routine = vec![
            "Review daily priorities".to_string(),
            "Check calendar for meetings".to_string(),
            "Prepare materials for first task".to_string(),
            "Set focus intention".to_string(),
            "Clear workspace".to_string(),
        ];
        templates.add_template("Morning Routine".to_string(), morning_routine.clone());
        
        // Save templates to disk
        save_templates(&templates, &config).unwrap();
        
        // Load templates back (simulating app startup)
        let loaded_templates = load_or_create_templates(&config).unwrap();
        let loaded_template = loaded_templates.get_template("Morning Routine").unwrap();
        
        // Verify template loaded correctly
        assert_eq!(loaded_template.len(), 5);
        assert_eq!(*loaded_template, morning_routine);
        
        // Create a daily goals with minimal existing actions
        let mut goals = DailyGoals::new(Local::now().date_naive());
        goals.work.actions[0].text = "Existing priority task".to_string();
        // Actions 1 and 2 are empty
        
        // Apply template to work outcome (simulating Morning phase)
        let work_outcome = &mut goals.work;
        
        // Ensure enough action slots
        while work_outcome.actions.len() < loaded_template.len() && work_outcome.actions.len() < 5 {
            work_outcome.add_action().unwrap();
        }
        
        // Apply template actions in order (skip existing non-empty actions)
        for (i, action_text) in loaded_template.iter().enumerate() {
            if i < work_outcome.actions.len() && work_outcome.actions[i].text.is_empty() {
                work_outcome.actions[i].text = action_text.clone();
                work_outcome.actions[i].origin = ActionOrigin::Template;
                work_outcome.actions[i].set_status(ActionStatus::Planned);
            }
        }
        
        // Verify final state
        assert_eq!(work_outcome.actions.len(), 5);
        
        // Verify existing action wasn't overwritten
        assert_eq!(work_outcome.actions[0].text, "Existing priority task");
        assert_eq!(work_outcome.actions[0].origin, ActionOrigin::Manual);
        
        // Verify template actions applied in correct order
        assert_eq!(work_outcome.actions[1].text, "Check calendar for meetings");
        assert_eq!(work_outcome.actions[2].text, "Prepare materials for first task"); 
        assert_eq!(work_outcome.actions[3].text, "Set focus intention");
        assert_eq!(work_outcome.actions[4].text, "Clear workspace");
        
        // Verify all template actions have correct metadata
        for i in 1..5 {
            assert_eq!(work_outcome.actions[i].origin, ActionOrigin::Template);
            assert_eq!(work_outcome.actions[i].status, ActionStatus::Planned);
        }
    }
    
    #[test]
    fn test_template_application_with_action_limit_reached() {
        let mut outcome = Outcome::new(OutcomeType::Health);
        
        // Fill outcome to maximum capacity (5 actions)
        for i in 0..5 {
            if i >= outcome.actions.len() {
                outcome.add_action().unwrap();
            }
            outcome.actions[i].text = format!("Existing Action {}", i + 1);
        }
        
        // Try to apply a 3-action template
        let mut templates = ActionTemplates::new();
        templates.add_template("Health Routine".to_string(), vec![
            "Template Action 1".to_string(),
            "Template Action 2".to_string(),
            "Template Action 3".to_string(),
        ]);
        
        let template = templates.get_template("Health Routine").unwrap();
        
        // Simulate template application (should not overwrite existing actions)
        let original_actions: Vec<String> = outcome.actions.iter()
            .map(|a| a.text.clone())
            .collect();
        
        for (i, action_text) in template.iter().enumerate() {
            if i < outcome.actions.len() && outcome.actions[i].text.is_empty() {
                outcome.actions[i].text = action_text.clone();
                outcome.actions[i].origin = ActionOrigin::Template;
            }
        }
        
        // Verify no actions were overwritten (all were non-empty)
        assert_eq!(outcome.actions.len(), 5);
        for (i, original_text) in original_actions.iter().enumerate() {
            assert_eq!(outcome.actions[i].text, *original_text);
            assert_eq!(outcome.actions[i].origin, ActionOrigin::Manual);
        }
    }
}