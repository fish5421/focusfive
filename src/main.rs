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
            data_root: "./FocusFive".to_string(),
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

                    // Save if needed - Goals first as they are the primary data
                    if app.needs_save {
                        match data::write_goals_file(&app.goals, config) {
                            Ok(_) => {
                                app.needs_save = false;
                                // Update streak after successful save
                                app.update_streak();
                                // Also mark metadata for save when goals are saved
                                app.day_meta.reconcile_with_goals(&app.goals);
                                app.meta_needs_save = true;
                            }
                            Err(e) => {
                                app.set_error(format!("Failed to save: {}", e));
                                app.needs_save = false; // Reset to avoid infinite retry
                            }
                        }
                    }
                    
                    // Save metadata if needed
                    if app.meta_needs_save {
                        match data::save_day_meta(app.goals.date, &app.day_meta, config) {
                            Ok(_) => {
                                app.meta_needs_save = false;
                            }
                            Err(e) => {
                                // Metadata save failure is non-critical, just log it
                                eprintln!("Warning: Failed to save metadata: {}", e);
                                app.meta_needs_save = false;
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

                    // Save templates if needed
                    if app.templates_needs_save {
                        match data::save_templates(&app.templates, config) {
                            Ok(_) => {
                                app.templates_needs_save = false;
                            }
                            Err(e) => {
                                app.set_error(format!("Failed to save templates: {}", e));
                                app.templates_needs_save = false; // Reset to avoid infinite retry
                            }
                        }
                    }

                    // Save objectives if needed (using atomic_write internally)
                    if app.objectives_needs_save {
                        match data::save_objectives(&app.objectives, config) {
                            Ok(_) => {
                                app.objectives_needs_save = false;
                            }
                            Err(e) => {
                                app.set_error(format!("Failed to save objectives: {}", e));
                                app.objectives_needs_save = false; // Reset to avoid infinite retry
                            }
                        }
                    }

                    // Save indicators if needed (using atomic_write internally)
                    if app.indicators_needs_save {
                        match data::save_indicators(&app.indicators, config) {
                            Ok(_) => {
                                app.indicators_needs_save = false;
                            }
                            Err(e) => {
                                app.set_error(format!("Failed to save indicators: {}", e));
                                app.indicators_needs_save = false; // Reset to avoid infinite retry
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
                            // Also save metadata
                            app.day_meta.reconcile_with_goals(&app.goals);
                            if let Err(e) = data::save_day_meta(app.goals.date, &app.day_meta, config) {
                                eprintln!("Warning: Failed to save metadata: {}", e);
                            }
                        }
                        if app.meta_needs_save {
                            if let Err(e) = data::save_day_meta(app.goals.date, &app.day_meta, config) {
                                eprintln!("Warning: Failed to save metadata: {}", e);
                            }
                        }
                        if app.vision_needs_save {
                            if let Err(e) = data::save_vision(&app.vision, config) {
                                eprintln!("Warning: Failed to save vision: {}", e);
                            }
                        }
                        if app.templates_needs_save {
                            if let Err(e) = data::save_templates(&app.templates, config) {
                                eprintln!("Warning: Failed to save templates: {}", e);
                            }
                        }
                        if app.objectives_needs_save {
                            if let Err(e) = data::save_objectives(&app.objectives, config) {
                                eprintln!("Warning: Failed to save objectives: {}", e);
                            }
                        }
                        if app.indicators_needs_save {
                            if let Err(e) = data::save_indicators(&app.indicators, config) {
                                eprintln!("Warning: Failed to save indicators: {}", e);
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
