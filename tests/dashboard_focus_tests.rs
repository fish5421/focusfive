use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use chrono::{Duration, Local, NaiveDate, Utc};
use crossterm::event::KeyCode;
use focusfive::models::{
    Config, IndicatorDef, IndicatorDirection, IndicatorKind, IndicatorUnit, IndicatorsData,
    Observation, ObservationSource,
};
use focusfive::ui::app::{App, ModalState};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use tempfile::tempdir;
use uuid::Uuid;

/// Copy a dashboard markdown fixture into a temporary FocusFive layout and return the config.
fn setup_markdown_fixture(fixture: &str) -> (Config, tempfile::TempDir, NaiveDate, PathBuf) {
    let temp = tempdir().expect("temp dir");
    let goals_dir = temp.path().join("goals");
    let data_root = temp.path().join("data");
    fs::create_dir_all(&goals_dir).expect("goals dir");
    fs::create_dir_all(&data_root).expect("data dir");

    let today = Local::now().date_naive();
    let fixture_src = PathBuf::from("tests/fixtures/dashboard").join(fixture);
    let fixture_dst = goals_dir.join(format!("{}.md", today.format("%Y-%m-%d")));
    fs::copy(&fixture_src, &fixture_dst).expect("copy markdown fixture");

    let config = Config {
        goals_dir: goals_dir.to_string_lossy().to_string(),
        data_root: data_root.to_string_lossy().to_string(),
    };

    (config, temp, today, data_root)
}

/// Render the dashboard into a vector of lines for inspection.
#[allow(deprecated)]
fn render_dashboard(app: &mut App) -> Vec<String> {
    let backend = TestBackend::new(200, 50);
    let mut terminal = Terminal::new(backend).expect("terminal");

    app.show_dashboard = true;

    terminal
        .draw(|frame| {
            app.render(frame);
        })
        .expect("render dashboard");

    let backend = terminal.backend_mut();
    let buffer = backend.buffer().clone();
    let area = buffer.area;

    (0..area.height)
        .map(|y| {
            let mut line = String::new();
            for x in 0..area.width {
                let cell = buffer.get(x, y);
                line.push_str(cell.symbol());
            }
            line
        })
        .collect()
}

#[test]
fn dashboard_header_uses_markdown_day_number() -> Result<()> {
    let (config, _guard, _today, _data_root) = setup_markdown_fixture("day_zero_target.md");

    let mut app = App::new(config)?;
    let lines = render_dashboard(&mut app);

    let screen = lines.join("\n");
    assert!(
        screen.contains("FOCUSFIVE GOAL TRACKING SYSTEM"),
        "dashboard header missing title:\n{}",
        screen
    );
    assert!(
        screen.contains("Day 207"),
        "expected day number from markdown:\n{}",
        screen
    );

    Ok(())
}

#[test]
fn alternative_signals_respects_zero_targets_from_markdown_context() -> Result<()> {
    let (config, _guard, today, data_root) = setup_markdown_fixture("day_zero_target.md");

    // Write indicator definitions with a zero target leading indicator.
    let indicator = IndicatorDef {
        id: "lead-zero".to_string(),
        name: "Incident Backlog".to_string(),
        kind: IndicatorKind::Leading,
        unit: IndicatorUnit::Percent,
        objective_id: None,
        target: Some(0.0),
        direction: IndicatorDirection::LowerIsBetter,
        active: true,
        created: Utc::now(),
        modified: Utc::now(),
        lineage_of: None,
        notes: None,
    };

    let indicators_path = data_root.join("indicators.json");
    let indicators = IndicatorsData {
        version: 1,
        indicators: vec![indicator.clone()],
    };
    fs::write(
        &indicators_path,
        serde_json::to_string_pretty(&indicators).unwrap(),
    )
    .expect("write indicators");

    // Write observations with a prior and current value.
    let observations_path = data_root.join("observations.ndjson");
    let mut observations_file = File::create(&observations_path).expect("create observations");

    let previous = Observation {
        id: Uuid::new_v4().to_string(),
        indicator_id: indicator.id.clone(),
        when: today - Duration::days(1),
        value: 2.0,
        unit: IndicatorUnit::Percent,
        source: ObservationSource::Manual,
        action_id: None,
        note: None,
        created: Utc::now(),
    };

    let latest = Observation {
        id: Uuid::new_v4().to_string(),
        indicator_id: indicator.id.clone(),
        when: today,
        value: 1.0,
        unit: IndicatorUnit::Percent,
        source: ObservationSource::Manual,
        action_id: None,
        note: None,
        created: Utc::now(),
    };

    writeln!(
        observations_file,
        "{}",
        serde_json::to_string(&previous).unwrap()
    )
    .expect("write previous observation");
    writeln!(
        observations_file,
        "{}",
        serde_json::to_string(&latest).unwrap()
    )
    .expect("write latest observation");

    // Recreate the app now that data files exist.
    let mut app = App::new(config)?;
    let lines = render_dashboard(&mut app);
    let screen = lines.join("\n");

    // Headline signals should surface the indicator with correct strength and delta.
    assert!(
        screen.contains("Incident Backlog"),
        "signal list missing indicator name:\n{}",
        screen
    );
    assert!(
        screen.contains("Wt 100.0%"),
        "signal list missing normalized weight:\n{}",
        screen
    );
    assert!(
        screen.contains("Δ-1.0"),
        "expected negative delta for lower-is-better target:\n{}",
        screen
    );
    assert!(
        screen.contains("  0.0%"),
        "expected zero strength for off-target value:\n{}",
        screen
    );
    assert!(
        screen.contains("░░░░░░░░░░"),
        "empty strength bar should render when off target:\n{}",
        screen
    );
    Ok(())
}

#[test]
fn dashboard_signal_update_appends_observation() -> Result<()> {
    let (config, _guard, today, data_root) = setup_markdown_fixture("day_zero_target.md");

    let indicator = IndicatorDef {
        id: "lead-zero".to_string(),
        name: "Incident Backlog".to_string(),
        kind: IndicatorKind::Leading,
        unit: IndicatorUnit::Percent,
        objective_id: None,
        target: Some(0.0),
        direction: IndicatorDirection::LowerIsBetter,
        active: true,
        created: Utc::now(),
        modified: Utc::now(),
        lineage_of: None,
        notes: None,
    };

    let indicators_path = data_root.join("indicators.json");
    let indicators = IndicatorsData {
        version: 1,
        indicators: vec![indicator.clone()],
    };
    fs::write(
        &indicators_path,
        serde_json::to_string_pretty(&indicators).unwrap(),
    )?;

    let observations_path = data_root.join("observations.ndjson");
    let mut observations_file = File::create(&observations_path)?;

    let previous = Observation {
        id: Uuid::new_v4().to_string(),
        indicator_id: indicator.id.clone(),
        when: today - Duration::days(1),
        value: 2.0,
        unit: IndicatorUnit::Percent,
        source: ObservationSource::Manual,
        action_id: None,
        note: None,
        created: Utc::now(),
    };

    let latest = Observation {
        id: Uuid::new_v4().to_string(),
        indicator_id: indicator.id.clone(),
        when: today,
        value: 1.0,
        unit: IndicatorUnit::Percent,
        source: ObservationSource::Manual,
        action_id: None,
        note: None,
        created: Utc::now(),
    };

    writeln!(
        observations_file,
        "{}",
        serde_json::to_string(&previous).unwrap()
    )?;
    writeln!(
        observations_file,
        "{}",
        serde_json::to_string(&latest).unwrap()
    )?;

    let mut app = App::new(config)?;

    app.handle_key(KeyCode::Char('d'))?;
    render_dashboard(&mut app);
    app.handle_key(KeyCode::Char('l'))?;
    app.handle_key(KeyCode::Char('l'))?;
    app.handle_key(KeyCode::Char('l'))?;
    render_dashboard(&mut app);

    app.handle_key(KeyCode::Char('i'))?;
    assert!(matches!(app.modal, Some(ModalState::IndicatorUpdate(_))));

    app.handle_key(KeyCode::Char('c'))?;
    app.handle_key(KeyCode::Char('0'))?;
    app.handle_key(KeyCode::Enter)?;

    assert!(app.modal.is_none());

    let contents = fs::read_to_string(&observations_path)?;
    let last_line = contents
        .lines()
        .last()
        .expect("observation entry should exist");
    let observation: Observation = serde_json::from_str(last_line)?;
    assert_eq!(observation.value, 0.0);

    Ok(())
}
