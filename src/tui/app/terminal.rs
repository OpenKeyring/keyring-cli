//! Terminal lifecycle management for TUI application
//!
//! Contains terminal initialization, restoration, and the main event loop.

use super::types::{TuiError, TuiResult};
use super::Screen;
use crate::error::{KeyringError, Result};
use crate::tui::traits::{Action, HandleResult, Interactive, ScreenType};
use crossterm::event::{self, KeyCode, KeyEventKind};
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

    let needs_setup = !crate::onboarding::is_initialized(&keystore_path);
    if needs_setup {
        // User not initialized - show wizard for first-time setup
        app.wizard_state = Some(
            crate::tui::screens::wizard::WizardState::new()
                .with_keystore_path(keystore_path.clone()),
        );
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
                    // Confirm dialog takes priority over all screens
                    if app.confirm_dialog.is_some() {
                        handle_confirm_dialog_key(&mut app, key);
                    } else {
                        // Route to current screen
                        match app.current_screen {
                            Screen::Wizard => {
                                app.handle_wizard_key_event(key);
                            }
                            Screen::Unlock => {
                                app.handle_unlock_key_event(key);
                            }
                            Screen::Main => {
                                let result = app
                                    .main_screen
                                    .handle_key_with_state(key, &mut app.app_state);
                                match result {
                                    HandleResult::Action(action) => {
                                        app.handle_main_screen_action(action);
                                    }
                                    HandleResult::Ignored => {
                                        if key.code == KeyCode::Char('q') {
                                            app.quit();
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Screen::NewPassword => {
                                let result = app.new_password_screen.handle_key(key);
                                app.handle_new_password_result(result);
                            }
                            Screen::EditPassword => {
                                let result = app.edit_password_screen.handle_key(key);
                                app.handle_edit_password_result(result);
                            }
                            Screen::Trash => {
                                let result = app
                                    .trash_screen
                                    .handle_key_with_state(key, &mut app.app_state);
                                match result {
                                    HandleResult::Action(Action::CloseScreen) => {
                                        app.navigate_to(Screen::Main);
                                    }
                                    HandleResult::Action(Action::OpenScreen(
                                        ScreenType::ConfirmDialog(action),
                                    )) => {
                                        app.show_confirm_dialog(action);
                                    }
                                    _ => {}
                                }
                            }
                            Screen::Settings => {
                                if key.code == KeyCode::Esc {
                                    app.navigate_to(Screen::Main);
                                } else {
                                    handle_settings_key(&mut app, key);
                                }
                            }
                            Screen::Help => {
                                if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                                    app.navigate_to(Screen::Main);
                                }
                            }
                            Screen::Sync => {
                                if key.code == KeyCode::Esc {
                                    app.navigate_to(Screen::Main);
                                }
                            }
                            _ => {
                                // Any other screen: Esc returns to Main
                                if key.code == KeyCode::Esc {
                                    app.navigate_to(Screen::Main);
                                }
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

    // If wizard was needed but not completed, show message
    if needs_setup && !crate::onboarding::is_initialized(&keystore_path) {
        println!("Setup not completed. Run 'ok' again to restart the setup wizard.");
    }

    Ok(())
}

/// Handle key events when confirm dialog is active
fn handle_confirm_dialog_key(app: &mut super::TuiApp, key: crossterm::event::KeyEvent) {
    if let Some(dialog) = &mut app.confirm_dialog {
        let result = dialog.handle_key(key);
        match result {
            HandleResult::Action(Action::ConfirmDialog(action)) => {
                app.confirm_dialog = None;
                app.handle_confirmed_action(action);
            }
            HandleResult::Action(Action::CloseScreen) => {
                app.confirm_dialog = None;
            }
            _ => {}
        }
    }
}

/// Handle key events for the settings screen
fn handle_settings_key(app: &mut super::TuiApp, key: crossterm::event::KeyEvent) {
    // Settings screen basic navigation - delegate to screen if it has a handler
    // For MVP, settings is view-only; Esc handled by caller
    let _ = (app, key);
}
