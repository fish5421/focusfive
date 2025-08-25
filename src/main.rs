mod app;
mod data;
mod models;
mod ui;

use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use models::Config;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

fn main() -> Result<()> {
    let config = Config::new().unwrap_or_else(|e| {
        eprintln!("Warning: Could not determine home directory: {}", e);
        eprintln!("Using current directory for goals");
        Config {
            goals_dir: "./FocusFive/goals".to_string(),
        }
    });

    let today = Local::now().date_naive();

    // Load or create today's goals
    let goals = data::load_or_create_goals(today, &config)?;

    // Load or create 5-year vision
    let vision = data::load_or_create_vision(&config)?;

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = app::App::new(goals, config.clone(), vision);

    // Run the app
    let res = run_app(&mut terminal, &mut app, &config);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut app::App,
    config: &Config,
) -> Result<()> {
    loop {
        // Draw the UI
        terminal.draw(|f| ui::render_app(f, app))?;

        // Handle input
        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => {
                    app.handle_key(key)?;

                    // Save if needed
                    if app.needs_save {
                        match data::write_goals_file(&app.goals, config) {
                            Ok(_) => {
                                app.needs_save = false;
                                // Update streak after successful save
                                app.update_streak();
                            }
                            Err(e) => {
                                app.set_error(format!("Failed to save: {}", e));
                                app.needs_save = false; // Reset to avoid infinite retry
                            }
                        }
                    }

                    // Save vision if needed
                    if app.vision_needs_save {
                        match data::save_vision(&app.vision, config) {
                            Ok(_) => {
                                app.vision_needs_save = false;
                            }
                            Err(e) => {
                                app.set_error(format!("Failed to save vision: {}", e));
                                app.vision_needs_save = false; // Reset to avoid infinite retry
                            }
                        }
                    }

                    // Check if we should quit
                    if app.should_quit {
                        // Save any pending changes before quitting
                        if app.needs_save {
                            if let Err(e) = data::write_goals_file(&app.goals, config) {
                                eprintln!("Warning: Failed to save changes: {}", e);
                            }
                        }
                        if app.vision_needs_save {
                            if let Err(e) = data::save_vision(&app.vision, config) {
                                eprintln!("Warning: Failed to save vision: {}", e);
                            }
                        }
                        break;
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal was resized, just redraw on next iteration
                }
                _ => {}
            }
        }
    }

    Ok(())
}
