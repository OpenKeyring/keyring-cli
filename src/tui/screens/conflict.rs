//! Conflict Resolution Screen
//!
//! TUI screen for resolving sync conflicts between local and remote records.

use crate::sync::conflict::{Conflict, ConflictResolution};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListState, Paragraph, Wrap},
    Frame,
};

/// Conflict resolution screen
#[derive(Debug, Clone)]
pub struct ConflictResolutionScreen {
    /// List of conflicts to resolve
    conflicts: Vec<Conflict>,
    /// Currently selected conflict index
    selected_index: usize,
    /// List state for scrolling
    list_state: ListState,
    /// Resolution choices for each conflict
    resolutions: Vec<Option<ConflictResolution>>,
}

impl ConflictResolutionScreen {
    /// Creates a new conflict resolution screen
    pub fn new(conflicts: Vec<Conflict>) -> Self {
        let resolutions = vec![None; conflicts.len()];
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            conflicts,
            selected_index: 0,
            list_state,
            resolutions,
        }
    }

    /// Returns the list of conflicts
    pub fn get_conflicts(&self) -> &[Conflict] {
        &self.conflicts
    }

    /// Returns the currently selected conflict index
    pub fn get_selected_index(&self) -> usize {
        self.selected_index
    }

    /// Returns the resolution choices
    pub fn get_resolutions(&self) -> &[Option<ConflictResolution>] {
        &self.resolutions
    }

    /// Handles Down arrow (move to next conflict)
    pub fn handle_down(&mut self) {
        if !self.conflicts.is_empty() && self.selected_index < self.conflicts.len() - 1 {
            self.selected_index += 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    /// Handles Up arrow (move to previous conflict)
    pub fn handle_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    /// Handles key press for resolution selection
    pub fn handle_char(&mut self, c: char) {
        if !self.conflicts.is_empty() && self.selected_index < self.resolutions.len() {
            let resolution = match c {
                'l' | 'L' => Some(ConflictResolution::Local),
                'r' | 'R' => Some(ConflictResolution::Remote),
                'n' | 'N' => Some(ConflictResolution::Newer),
                'o' | 'O' => Some(ConflictResolution::Older),
                'i' | 'I' => Some(ConflictResolution::Interactive),
                _ => return,
            };
            self.resolutions[self.selected_index] = resolution;
        }
    }

    /// Handles Enter key (confirm resolutions)
    pub fn has_unresolved_conflicts(&self) -> bool {
        self.resolutions.iter().any(|r| r.is_none())
    }

    /// Returns all resolved conflicts
    pub fn get_resolved_conflicts(&self) -> Vec<Conflict> {
        self.conflicts
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                self.resolutions.get(i).and_then(|r| {
                    r.as_ref().map(|resolution| {
                        let mut conflict = c.clone();
                        conflict.resolution = Some(resolution.clone());
                        conflict
                    })
                })
            })
            .collect()
    }

    /// Renders the conflict resolution screen
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Title
        let title = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "冲突解决 / Conflict Resolution",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("共 {} 个冲突需要解决 / {} conflicts to resolve", self.conflicts.len(), self.conflicts.len()),
                Style::default().fg(Color::Yellow),
            )),
        ]))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    ratatui::layout::Constraint::Length(4), // Title
                    ratatui::layout::Constraint::Min(0),    // Conflict list
                    ratatui::layout::Constraint::Length(3), // Footer
                ]
                .as_ref(),
            )
            .split(area);

        frame.render_widget(title, chunks[0]);

        // Conflict list
        let conflict_items: Vec<Line> = self
            .conflicts
            .iter()
            .enumerate()
            .map(|(i, conflict)| {
                let is_selected = i == self.selected_index;
                let resolution = self.resolutions.get(i).and_then(|r| r.as_ref());

                let record_info = if let (Some(local), Some(remote)) =
                    (&conflict.local_record, &conflict.remote_record)
                {
                    format!("v{} local vs v{} remote", local.version, remote.version)
                } else if conflict.local_record.is_some() {
                    "local only".to_string()
                } else if conflict.remote_record.is_some() {
                    "remote only".to_string()
                } else {
                    "empty".to_string()
                };

                let resolution_text = match resolution {
                    Some(ConflictResolution::Local) => "[Local]",
                    Some(ConflictResolution::Remote) => "[Remote]",
                    Some(ConflictResolution::Newer) => "[Newer]",
                    Some(ConflictResolution::Older) => "[Older]",
                    Some(ConflictResolution::Interactive) => "[Interactive]",
                    Some(ConflictResolution::Merge) => "[Merge]",
                    None => "[Unresolved]",
                };

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                Line::from(vec![
                    Span::styled(format!("{}. ", i + 1), style),
                    Span::styled(&conflict.id[..8], style),
                    Span::styled(" - ", style),
                    Span::styled(record_info, style),
                    Span::styled(" ", style),
                    Span::styled(
                        resolution_text,
                        Style::default()
                            .fg(if resolution.is_some() {
                                Color::Green
                            } else {
                                Color::Red
                            }),
                    ),
                ])
            })
            .collect();

        let list = List::new(conflict_items)
            .block(Block::default().borders(Borders::ALL).title("冲突列表 / Conflicts"));

        let mut list_state = self.list_state.clone();
        frame.render_stateful_widget(list, chunks[1], &mut list_state);

        // Footer
        let footer = Paragraph::new(Text::from(vec![Line::from(vec![
            Span::from("L: Local  "),
            Span::from("R: Remote  "),
            Span::from("N: Newer  "),
            Span::from("O: Older  "),
            Span::from("I: Interactive  "),
            Span::from("Enter: Confirm  "),
            Span::from("Esc: Cancel"),
        ])]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, chunks[2]);
    }
}
