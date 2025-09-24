// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! History Panel Component

use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::{Frame, text::Text};

use super::{ComponentResult, ScrollState, SizeConstraints, TuiComponent, utils};
use crate::tui::app::AppState;
use crate::tui::config::TuiConfig;
use crate::tui::layout::PanelType;
use crate::tui::themes::TuiTheme;

/// History panel for displaying command history
pub struct HistoryPanel {
    list_state: ListState,
    scroll_state: ScrollState,
}

impl HistoryPanel {
    /// Create a new history panel
    pub async fn new(_config: &TuiConfig) -> Result<Self> {
        Ok(Self {
            list_state: ListState::default(),
            scroll_state: ScrollState::new(),
        })
    }
}

impl TuiComponent for HistoryPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState, theme: &TuiTheme) {
        let is_focused = state.focused_panel == PanelType::History;
        let block = utils::create_panel_block("History", PanelType::History, is_focused, theme);

        let items: Vec<ListItem> = if state.evaluation_history.is_empty() {
            vec![ListItem::new("No history yet")]
        } else {
            state
                .evaluation_history
                .iter()
                .rev() // Show most recent first
                .take(20) // Limit display
                .map(|entry| {
                    let status = if entry.result.is_ok() { "✓" } else { "✗" };
                    let time = format!("{:.1}ms", entry.execution_time.as_secs_f64() * 1000.0);
                    let item_text = format!("{} {} ({})", status, entry.expression, time);
                    ListItem::new(Text::from(item_text))
                })
                .collect()
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(theme.styles.selected_item);

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> ComponentResult {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Up => {
                self.scroll_state
                    .select_previous(state.evaluation_history.len().min(20));
                self.list_state.select(self.scroll_state.selected_index);
                ComponentResult::Handled
            }
            KeyCode::Down => {
                self.scroll_state
                    .select_next(state.evaluation_history.len().min(20));
                self.list_state.select(self.scroll_state.selected_index);
                ComponentResult::Handled
            }
            KeyCode::Enter => {
                if let Some(selected) = self.scroll_state.selected_index {
                    if state
                        .evaluation_history
                        .iter()
                        .rev()
                        .nth(selected)
                        .is_some()
                    {
                        return ComponentResult::LoadFromHistory(selected);
                    }
                }
                ComponentResult::Handled
            }
            KeyCode::Delete => ComponentResult::ClearHistory,
            _ => ComponentResult::NotHandled,
        }
    }

    fn update(&mut self, _state: &mut AppState) -> ComponentResult {
        ComponentResult::Handled
    }

    fn size_constraints(&self) -> SizeConstraints {
        SizeConstraints {
            min_width: Some(25),
            preferred_width: Some(35),
            max_width: None,
            min_height: Some(5),
            preferred_height: Some(10),
            max_height: None,
        }
    }
}
