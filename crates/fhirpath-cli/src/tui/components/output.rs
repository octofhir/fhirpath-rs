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

//! Output Panel Component

use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use ratatui::{Frame, text::Text};

use super::{ComponentResult, ScrollState, SizeConstraints, TuiComponent, utils};
use crate::tui::app::AppState;
use crate::tui::config::TuiConfig;
use crate::tui::layout::PanelType;
use crate::tui::themes::TuiTheme;

/// Output panel for displaying FHIRPath evaluation results
pub struct OutputPanel {
    scroll_state: ScrollState,
}

impl OutputPanel {
    /// Create a new output panel
    pub async fn new(_config: &TuiConfig) -> Result<Self> {
        Ok(Self {
            scroll_state: ScrollState::new(),
        })
    }
}

impl TuiComponent for OutputPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState, theme: &TuiTheme) {
        let is_focused = state.focused_panel == PanelType::Output;
        let block = utils::create_panel_block("Output", PanelType::Output, is_focused, theme);

        let content = if let Some(result) = &state.last_result {
            format!("{:?}", result) // Simplified for now
        } else {
            "No results yet. Execute an expression to see output here.".to_string()
        };

        let paragraph = Paragraph::new(Text::from(content))
            .block(block)
            .scroll((self.scroll_state.offset as u16, 0));

        frame.render_widget(paragraph, area);

        // Render scrollbar if needed
        if is_focused {
            let scrollbar = Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = ScrollbarState::default();
            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent, _state: &mut AppState) -> ComponentResult {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Up => {
                self.scroll_state.scroll_up();
                ComponentResult::Handled
            }
            KeyCode::Down => {
                self.scroll_state.scroll_down(100, 10); // TODO: Calculate actual content size
                ComponentResult::Handled
            }
            KeyCode::PageUp => {
                for _ in 0..5 {
                    self.scroll_state.scroll_up();
                }
                ComponentResult::Handled
            }
            KeyCode::PageDown => {
                for _ in 0..5 {
                    self.scroll_state.scroll_down(100, 10);
                }
                ComponentResult::Handled
            }
            _ => ComponentResult::NotHandled,
        }
    }

    fn update(&mut self, _state: &mut AppState) -> ComponentResult {
        ComponentResult::Handled
    }

    fn size_constraints(&self) -> SizeConstraints {
        SizeConstraints {
            min_height: Some(5),
            preferred_height: Some(15),
            ..Default::default()
        }
    }
}
