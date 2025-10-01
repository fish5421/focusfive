use crate::models::{Config, DailyGoals, OutcomeType};
use chrono::{Duration, NaiveDate};

pub struct Statistics {
    pub daily_completion: f64,
    pub weekly_trend: Vec<f64>,
    pub monthly_trend: Vec<f64>,
    pub outcome_percentages: (f64, f64, f64), // work, health, family
}

impl Statistics {
    pub fn calculate(config: &Config, current_date: NaiveDate) -> Self {
        let mut weekly_trend = Vec::new();
        let mut monthly_trend = Vec::new();

        // Load last 7 days for weekly trend
        for i in (0..7).rev() {
            let date = current_date - Duration::days(i);
            if let Ok(goals) = crate::data::load_or_create_goals(date, config) {
                weekly_trend.push(calculate_completion_percentage(&goals));
            } else {
                // If no data for that day, assume 0% completion
                weekly_trend.push(0.0);
            }
        }

        // Load last 30 days for monthly trend
        for i in (0..30).rev() {
            let date = current_date - Duration::days(i);
            if let Ok(goals) = crate::data::load_or_create_goals(date, config) {
                monthly_trend.push(calculate_completion_percentage(&goals));
            } else {
                // If no data for that day, assume 0% completion
                monthly_trend.push(0.0);
            }
        }

        // Calculate daily completion for today
        let daily_completion =
            if let Ok(goals) = crate::data::load_or_create_goals(current_date, config) {
                calculate_completion_percentage(&goals)
            } else {
                0.0
            };

        // Calculate outcome percentages for today
        let outcome_percentages =
            if let Ok(goals) = crate::data::load_or_create_goals(current_date, config) {
                let work_pct = calculate_outcome_percentage(&goals, OutcomeType::Work);
                let health_pct = calculate_outcome_percentage(&goals, OutcomeType::Health);
                let family_pct = calculate_outcome_percentage(&goals, OutcomeType::Family);
                (work_pct, health_pct, family_pct)
            } else {
                (0.0, 0.0, 0.0)
            };

        Self {
            daily_completion,
            weekly_trend,
            monthly_trend,
            outcome_percentages,
        }
    }

    // Calculate statistics from already loaded goals (more efficient for live updates)
    pub fn from_current_goals(goals: &DailyGoals, config: &Config) -> Self {
        let daily_completion = calculate_completion_percentage(goals);
        let outcome_percentages = (
            calculate_outcome_percentage(goals, OutcomeType::Work),
            calculate_outcome_percentage(goals, OutcomeType::Health),
            calculate_outcome_percentage(goals, OutcomeType::Family),
        );

        // Still need to load historical data for trends
        let mut weekly_trend = Vec::new();
        let mut monthly_trend = Vec::new();

        // Load last 7 days for weekly trend
        for i in (0..7).rev() {
            let date = goals.date - Duration::days(i);
            if let Ok(historical_goals) = crate::data::load_or_create_goals(date, config) {
                weekly_trend.push(calculate_completion_percentage(&historical_goals));
            } else {
                weekly_trend.push(0.0);
            }
        }

        // Load last 30 days for monthly trend
        for i in (0..30).rev() {
            let date = goals.date - Duration::days(i);
            if let Ok(historical_goals) = crate::data::load_or_create_goals(date, config) {
                monthly_trend.push(calculate_completion_percentage(&historical_goals));
            } else {
                monthly_trend.push(0.0);
            }
        }

        Self {
            daily_completion,
            weekly_trend,
            monthly_trend,
            outcome_percentages,
        }
    }
}

fn calculate_completion_percentage(goals: &DailyGoals) -> f64 {
    let total = 9; // 3 outcomes * 3 actions
    let completed = goals.work.actions.iter().filter(|a| a.completed).count()
        + goals.health.actions.iter().filter(|a| a.completed).count()
        + goals.family.actions.iter().filter(|a| a.completed).count();

    (completed as f64 / total as f64) * 100.0
}

fn calculate_outcome_percentage(goals: &DailyGoals, outcome_type: OutcomeType) -> f64 {
    let outcome = match outcome_type {
        OutcomeType::Work => &goals.work,
        OutcomeType::Health => &goals.health,
        OutcomeType::Family => &goals.family,
    };

    let total = 3;
    let completed = outcome.actions.iter().filter(|a| a.completed).count();

    (completed as f64 / total as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Action, Outcome};

    fn create_test_action(text: &str, completed: bool) -> Action {
        Action {
            id: uuid::Uuid::new_v4().to_string(),
            text: text.to_string(),
            completed,
            status: if completed {
                crate::models::ActionStatus::Done
            } else {
                crate::models::ActionStatus::Planned
            },
            origin: crate::models::ActionOrigin::Manual,
            objective_id: None,
            objective_ids: vec![],
            created: chrono::Utc::now(),
            modified: chrono::Utc::now(),
            completed_at: if completed {
                Some(chrono::Utc::now())
            } else {
                None
            },
        }
    }

    #[test]
    fn test_completion_percentage() {
        let goals = DailyGoals {
            date: chrono::Local::now().date_naive(),
            day_number: Some(1),
            work: Outcome {
                outcome_type: OutcomeType::Work,
                goal: None,
                reflection: None,
                actions: vec![
                    create_test_action("Task 1", true),
                    create_test_action("Task 2", false),
                    create_test_action("Task 3", true),
                ],
            },
            health: Outcome {
                outcome_type: OutcomeType::Health,
                goal: None,
                reflection: None,
                actions: vec![
                    create_test_action("Task 1", true),
                    create_test_action("Task 2", true),
                    create_test_action("Task 3", true),
                ],
            },
            family: Outcome {
                outcome_type: OutcomeType::Family,
                goal: None,
                reflection: None,
                actions: vec![
                    create_test_action("Task 1", false),
                    create_test_action("Task 2", false),
                    create_test_action("Task 3", false),
                ],
            },
        };

        let percentage = calculate_completion_percentage(&goals);
        // 5 out of 9 tasks completed = 55.55%
        assert!((percentage - 55.55).abs() < 0.1);
    }

    #[test]
    fn test_outcome_percentage() {
        let goals = DailyGoals {
            date: chrono::Local::now().date_naive(),
            day_number: Some(1),
            work: Outcome {
                outcome_type: OutcomeType::Work,
                goal: None,
                reflection: None,
                actions: vec![
                    create_test_action("Task 1", true),
                    create_test_action("Task 2", false),
                    create_test_action("Task 3", true),
                ],
            },
            health: Outcome {
                outcome_type: OutcomeType::Health,
                goal: None,
                reflection: None,
                actions: vec![
                    create_test_action("Task 1", true),
                    create_test_action("Task 2", true),
                    create_test_action("Task 3", true),
                ],
            },
            family: Outcome {
                outcome_type: OutcomeType::Family,
                goal: None,
                reflection: None,
                actions: vec![
                    create_test_action("Task 1", false),
                    create_test_action("Task 2", false),
                    create_test_action("Task 3", false),
                ],
            },
        };

        let work_pct = calculate_outcome_percentage(&goals, OutcomeType::Work);
        let health_pct = calculate_outcome_percentage(&goals, OutcomeType::Health);
        let family_pct = calculate_outcome_percentage(&goals, OutcomeType::Family);

        assert!((work_pct - 66.66).abs() < 0.1); // 2/3 completed
        assert!((health_pct - 100.0).abs() < 0.1); // 3/3 completed
        assert!((family_pct - 0.0).abs() < 0.1); // 0/3 completed
    }
}
