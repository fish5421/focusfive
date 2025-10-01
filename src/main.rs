mod data;
mod models;
mod ui;
mod ui_state;
mod widgets;

use ui::{init_terminal, restore_terminal, run_app, App};

fn main() -> anyhow::Result<()> {
    let config = models::Config::new().unwrap_or_else(|e| {
        eprintln!("Warning: {}. Using fallback.", e);
        models::Config {
            goals_dir: "./FocusFive/goals".to_string(),
            data_root: "./FocusFive".to_string(),
        }
    });

    let mut terminal = init_terminal()?;
    let app = App::new(config)?;

    let result = run_app(&mut terminal, app);

    restore_terminal(&mut terminal)?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}
