//! Terminal lifecycle management for TUI application
//!
//! Contains terminal initialization, restoration, and the main event loop.

use super::types::{TuiError, TuiResult};
use super::Screen;
use crate::error::{KeyringError, Result};
use crate::tui::components::ConfirmAction;
use crate::tui::traits::{Action, DatabaseService, HandleResult, ScreenType};
use crossterm::event::{self, KeyEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::time::Duration;

/// Initialize terminal for TUI mode
pub fn init_terminal() -> TuiResult<Terminal<CrosstermBackend<Stdout>>> {
    use crossterm::{
        event::EnableMouseCapture,
        execute,
        terminal::{enable_raw_mode, EnterAlternateScreen},
    };

    enable_raw_mode().map_err(|e| TuiError::InitFailed(e.to_string()))?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| TuiError::InitFailed(e.to_string()))?;

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend).map_err(|e| TuiError::InitFailed(e.to_string()))?;

    Ok(terminal)
}

/// Restore terminal after TUI mode
pub fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> TuiResult<()> {
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, LeaveAlternateScreen},
    };

    disable_raw_mode().map_err(|e| TuiError::RestoreFailed(e.to_string()))?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )
    .map_err(|e| TuiError::RestoreFailed(e.to_string()))?;

    terminal
        .show_cursor()
        .map_err(|e| TuiError::RestoreFailed(e.to_string()))?;

    Ok(())
}

/// Run the TUI application
pub fn run_tui() -> Result<()> {
    // Install panic hook FIRST to ensure terminal recovery on panic
    crate::tui::panic_hook::install_panic_hook();

    let mut terminal =
        init_terminal().map_err(|e| KeyringError::IoError(format!("Failed to init TUI: {}", e)))?;

    let mut app = super::TuiApp::new();

    // Check onboarding status - show wizard if keystore doesn't exist
    let keystore_path = crate::cli::config::ConfigManager::new()
        .map(|cm| cm.get_keystore_path())
        .unwrap_or_else(|_| {
            // Fallback to default path if config manager fails
            dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("open-keyring")
                .join("keystore.json")
        });
    if !crate::onboarding::is_initialized(&keystore_path) {
        // User not initialized - show wizard for first-time setup
        app.wizard_state =
            Some(crate::tui::screens::wizard::WizardState::new().with_keystore_path(keystore_path));
        app.current_screen = Screen::Wizard;
    } else {
        // User already initialized - show unlock screen to enter master password
        app.current_screen = Screen::Unlock;
    }

    // Main event loop
    while app.is_running() {
        terminal
            .draw(|f| app.render(f))
            .map_err(|e| KeyringError::IoError(format!("Failed to draw: {}", e)))?;

        // Cleanup expired notifications (3-second auto-dismiss for Info/Success)
        app.app_state.cleanup_notifications();

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))
            .map_err(|e| KeyringError::IoError(format!("Event poll failed: {}", e)))?
        {
            match event::read()
                .map_err(|e| KeyringError::IoError(format!("Event read failed: {}", e)))?
            {
                // Filter to only handle Press events to avoid duplicate key handling on Windows
                event::Event::Key(key) if key.kind == KeyEventKind::Press => {
                    use crossterm::event::KeyCode;

                    // Route wizard events
                    if app.current_screen == Screen::Wizard {
                        app.handle_wizard_key_event(key);
                    } else if app.current_screen == Screen::Unlock {
                        // Route unlock screen events
                        app.handle_unlock_key_event(key);
                    } else if app.current_screen == Screen::Trash {
                        // Route trash screen events
                        let result = app
                            .trash_screen
                            .handle_key_with_state(key, &mut app.app_state);
                        match result {
                            HandleResult::Action(Action::CloseScreen) => {
                                app.navigate_to(Screen::Main);
                            }
                            HandleResult::Action(Action::OpenScreen(ScreenType::ConfirmDialog(action))) => {
                                app.show_confirm_dialog(action);
                            }
                            _ => {}
                        }
                    } else if app.current_screen == Screen::Main {
                        // Route main screen events to MainScreen handler
                        match app
                            .main_screen
                            .handle_key_with_state(key, &mut app.app_state)
                        {
                            HandleResult::Consumed => {}
                            HandleResult::Ignored => {
                                // Fallback: handle global shortcuts
                                if key.code == KeyCode::Char('q') {
                                    app.quit();
                                }
                            }
                            HandleResult::Action(action) => {
                                handle_main_screen_action(&mut app, action);
                            }
                            HandleResult::NeedsRender => {}
                        }
                    } else {
                        // Check for keyboard shortcuts first (Ctrl keys)
                        if key.modifiers.contains(event::KeyModifiers::CONTROL) {
                            app.handle_key_event(key);
                        } else {
                            // Regular input handling
                            match key.code {
                                KeyCode::Char(c) => app.handle_char(c),
                                KeyCode::Backspace | KeyCode::Delete => app.handle_backspace(),
                                KeyCode::Enter => app.handle_char('\n'),
                                KeyCode::Esc
                                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                                {
                                    app.quit();
                                }
                                _ => {}
                            }
                        }
                    }
                }
                event::Event::Resize(_, _) => {
                    // Terminal resized - will be handled on next draw
                }
                _ => {}
            }
        }
    }

    restore_terminal(terminal)
        .map_err(|e| KeyringError::IoError(format!("Failed to restore terminal: {}", e)))?;

    Ok(())
}

/// Handle actions from the main screen
fn handle_main_screen_action(app: &mut super::TuiApp, action: Action) {
    match action {
        Action::Quit => {
            app.quit();
        }
        Action::OpenScreen(screen_type) => match screen_type {
            ScreenType::Help => {
                app.show_help();
            }
            ScreenType::Settings => {
                app.navigate_to(Screen::Settings);
            }
            ScreenType::NewPassword => {
                app.navigate_to(Screen::NewPassword);
            }
            ScreenType::EditPassword(id_str) => {
                // Get password data from cache
                if let Some(record) = app.app_state.get_password_by_str(&id_str).cloned() {
                    // Create EditPasswordScreen with existing data
                    // Convert String ID to Uuid
                    if let Ok(uuid) = uuid::Uuid::parse_str(&record.id) {
                        app.edit_password_screen = crate::tui::screens::EditPasswordScreen::new(
                            uuid,
                            &record.name,
                            record.username.as_deref(),
                            &record.password,
                            record.url.as_deref(),
                            record.notes.as_deref(),
                            &record.tags,
                            record.group_id.as_deref(),
                        );
                        app.navigate_to(Screen::EditPassword);
                    } else {
                        app.output_lines
                            .push(format!("Invalid UUID in record: {}", record.id));
                    }
                } else {
                    app.output_lines
                        .push(format!("Password not found: {}", id_str));
                }
            }
            _ => {
                // For other screens, show a placeholder message
                app.output_lines
                    .push(format!("Screen {:?} not yet implemented", screen_type));
            }
        },
        Action::ShowToast(message) => {
            app.output_lines.push(message);
        }
        Action::CloseScreen => {
            // Return to main screen
            app.navigate_to(Screen::Main);
        }
        Action::CopyToClipboard(content) => {
            app.output_lines.push(format!("Copied: {}", content));
        }
        Action::Refresh => {
            app.output_lines.push("Refreshed".to_string());
        }
        Action::ConfirmDialog(action) => {
            handle_confirm_dialog_action(app, action);
        }
        Action::None => {}
    }
}

/// Handle confirmed actions from dialogs
/// Note: MutexGuard across await is safe here because we're in block_in_place
#[allow(clippy::await_holding_lock)]
fn handle_confirm_dialog_action(app: &mut super::TuiApp, action: ConfirmAction) {
    match action {
        ConfirmAction::DeletePassword {
            password_id,
            password_name,
        } => {
            // Delete from database first
            let deleted = if let Some(db_service) = app.app_state.db_service() {
                let db = db_service.clone();
                let id = password_id.clone();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        if let Ok(service) = db.lock() {
                            service.delete_password(&id, true).await.is_ok()
                        } else {
                            false
                        }
                    })
                })
            } else {
                false
            };

            if deleted {
                app.app_state.remove_password_from_cache(&password_id);
                app.output_lines
                    .push(format!("Deleted \"{}\"", password_name));
            } else {
                app.output_lines
                    .push(format!("Failed to delete \"{}\"", password_name));
            }
        }
        ConfirmAction::PermanentDelete(id) => {
            let password_name = app
                .app_state
                .get_password_by_str(&id)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| id.clone());

            // Permanently delete from database first
            let deleted = if let Some(db_service) = app.app_state.db_service() {
                let db = db_service.clone();
                let id_clone = id.clone();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        if let Ok(service) = db.lock() {
                            service.permanently_delete(&id_clone).await.is_ok()
                        } else {
                            false
                        }
                    })
                })
            } else {
                false
            };

            if deleted {
                app.app_state.permanent_delete_password(&id);
                app.output_lines
                    .push(format!("Permanently deleted \"{}\"", password_name));
            } else {
                app.output_lines.push(format!(
                    "Failed to permanently delete \"{}\"",
                    password_name
                ));
            }
        }
        ConfirmAction::EmptyTrash => {
            // Get all deleted password IDs before emptying
            let deleted_ids: Vec<String> = app
                .app_state
                .all_passwords()
                .iter()
                .filter(|p| p.is_deleted)
                .map(|p| p.id.clone())
                .collect();

            let mut success_count = 0;
            let mut fail_count = 0;

            // Permanently delete each from database
            if let Some(db_service) = app.app_state.db_service() {
                let db = db_service.clone();
                for id in deleted_ids {
                    let db_clone = db.clone();
                    let result = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            if let Ok(service) = db_clone.lock() {
                                service.permanently_delete(&id).await.is_ok()
                            } else {
                                false
                            }
                        })
                    });
                    if result {
                        success_count += 1;
                    } else {
                        fail_count += 1;
                    }
                }
            }

            // Empty trash in cache
            app.app_state.empty_trash();

            if fail_count > 0 {
                app.output_lines.push(format!(
                    "Emptied trash ({} deleted, {} failed)",
                    success_count, fail_count
                ));
            } else {
                app.output_lines.push(format!(
                    "Emptied trash ({} passwords permanently deleted)",
                    success_count
                ));
            }
        }
        ConfirmAction::Generic => {
            app.output_lines.push("Action confirmed".to_string());
        }
    }
}
