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

//! Input Panel Component

use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;
use tui_textarea::TextArea;

use super::{ComponentResult, SizeConstraints, TuiComponent, utils};
use crate::tui::app::AppState;
use crate::tui::config::TuiConfig;
use crate::tui::layout::PanelType;
use crate::tui::themes::TuiTheme;

// use octofhir_fhirpath::analyzer::StaticAnalyzer; // Removed

/// Input panel for FHIRPath expression editing
pub struct InputPanel {
    text_area: TextArea<'static>,
    // _analyzer: Option<std::sync::Arc<StaticAnalyzer>>, // Removed
}

impl InputPanel {
    /// Create a new input panel
    pub async fn new(_config: &TuiConfig /*, _analyzer: &StaticAnalyzer*/) -> Result<Self> {
        let mut text_area = TextArea::default();
        text_area.set_placeholder_text("Enter FHIRPath expression...");

        Ok(Self {
            text_area,
            // _analyzer: None,
        })
    }

    /// Get current expression text
    pub fn get_expression(&self) -> String {
        self.text_area.lines().join("")
    }

    /// Clear all text in the input area
    pub fn clear_text(&mut self) {
        // Reinitialize the text area to ensure a clean state
        let mut new_area = TextArea::default();
        new_area.set_placeholder_text("Enter FHIRPath expression...");
        self.text_area = new_area;
    }
}

impl TuiComponent for InputPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState, theme: &TuiTheme) {
        use ratatui::style::{Modifier, Style};
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, Clear, List, ListItem};

        let is_focused = state.focused_panel == PanelType::Input;
        let block = utils::create_panel_block("Input", PanelType::Input, is_focused, theme);

        self.text_area.set_block(block);

        if is_focused {
            self.text_area
                .set_cursor_line_style(theme.styles.cursor_style);
        }

        frame.render_widget(&self.text_area, area);

        // Render completion popup if focused and completions are available
        if is_focused && !state.completions.is_empty() {
            let visible = state.completions.len().min(8);
            let popup_height = visible as u16 + 2; // include borders
            let popup_width = area.width.saturating_sub(4).min(60);
            let popup_x = area.x + 2;
            let popup_y = area
                .y
                .saturating_add(area.height.saturating_sub(popup_height + 1));
            let popup_area = Rect {
                x: popup_x,
                y: popup_y,
                width: popup_width,
                height: popup_height,
            };

            // Clear background to avoid overlapping artifacts
            frame.render_widget(Clear, popup_area);

            let highlight_style = Style::default()
                .bg(theme.colors.selected_background)
                .fg(theme.colors.selected_text)
                .add_modifier(Modifier::BOLD);

            let items: Vec<ListItem> = state
                .completions
                .iter()
                .take(visible)
                .enumerate()
                .map(|(index, completion)| {
                    let is_selected = index == state.selected_completion;
                    let title_span = Span::styled(
                        completion.display.clone(),
                        if is_selected {
                            highlight_style
                        } else {
                            Style::default().fg(theme.colors.normal_text)
                        },
                    );

                    let mut lines = vec![Line::from(vec![title_span])];

                    if is_selected {
                        if let Some(doc) = completion.documentation.as_deref() {
                            let doc_line = Line::from(vec![Span::styled(
                                doc,
                                Style::default()
                                    .fg(theme.colors.disabled_text)
                                    .add_modifier(Modifier::ITALIC),
                            )]);
                            lines.push(doc_line);
                        }
                    }

                    ListItem::new(lines)
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Completions ")
                    .border_style(Style::default().fg(theme.colors.focused_border)),
            );

            frame.render_widget(list, popup_area);
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> ComponentResult {
        use crossterm::event::{KeyCode, KeyModifiers};

        // If completion list is visible, handle navigation/selection first
        if !state.completions.is_empty() {
            match (key.code, key.modifiers) {
                (KeyCode::Up, KeyModifiers::NONE) => {
                    if state.selected_completion > 0 {
                        state.selected_completion -= 1;
                    } else if !state.completions.is_empty() {
                        state.selected_completion = state.completions.len().saturating_sub(1);
                    }
                    return ComponentResult::Handled;
                }
                (KeyCode::Down, KeyModifiers::NONE) => {
                    if state.selected_completion + 1 < state.completions.len() {
                        state.selected_completion += 1;
                    } else {
                        state.selected_completion = 0;
                    }
                    return ComponentResult::Handled;
                }
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    if let Some(item) = state.completions.get(state.selected_completion) {
                        // Insert completion text at the end by simulating typing
                        for ch in item.text.chars() {
                            let input = tui_textarea::Input {
                                key: tui_textarea::Key::Char(ch),
                                ctrl: false,
                                alt: false,
                                shift: false,
                            };
                            self.text_area.input(input);
                        }
                        state.current_expression = self.get_expression();
                    }
                    // Clear completions after insertion
                    state.completions.clear();
                    return ComponentResult::Handled;
                }
                (KeyCode::Tab, KeyModifiers::NONE) => {
                    if let Some(item) = state.completions.get(state.selected_completion) {
                        for ch in item.text.chars() {
                            let input = tui_textarea::Input {
                                key: tui_textarea::Key::Char(ch),
                                ctrl: false,
                                alt: false,
                                shift: false,
                            };
                            self.text_area.input(input);
                        }
                        state.current_expression = self.get_expression();
                    }
                    state.completions.clear();
                    return ComponentResult::Handled;
                }
                (KeyCode::BackTab, KeyModifiers::SHIFT) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
                    if state.selected_completion > 0 {
                        state.selected_completion -= 1;
                    } else if !state.completions.is_empty() {
                        state.selected_completion = state.completions.len() - 1;
                    }
                    return ComponentResult::Handled;
                }
                (KeyCode::PageUp, KeyModifiers::NONE) => {
                    state.selected_completion = 0;
                    return ComponentResult::Handled;
                }
                (KeyCode::PageDown, KeyModifiers::NONE) => {
                    if !state.completions.is_empty() {
                        state.selected_completion = state.completions.len() - 1;
                    }
                    return ComponentResult::Handled;
                }
                (KeyCode::Esc, KeyModifiers::NONE) => {
                    state.completions.clear();
                    return ComponentResult::Handled;
                }
                _ => {}
            }
        }

        match (key.code, key.modifiers) {
            (KeyCode::Enter, KeyModifiers::NONE) => {
                state.current_expression = self.get_expression();
                ComponentResult::ExecuteExpression
            }
            (KeyCode::Char(' '), KeyModifiers::CONTROL) => {
                state.current_expression = self.get_expression();
                ComponentResult::ShowCompletions
            }
            _ => {
                // Forward all other keys to the textarea for text input
                // Create input event for tui_textarea, using raw event conversion
                let input = match key.code {
                    KeyCode::Char(c) => tui_textarea::Input {
                        key: tui_textarea::Key::Char(c),
                        ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
                        alt: key.modifiers.contains(KeyModifiers::ALT),
                        shift: key.modifiers.contains(KeyModifiers::SHIFT),
                    },
                    KeyCode::Backspace => tui_textarea::Input {
                        key: tui_textarea::Key::Backspace,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    },
                    KeyCode::Delete => tui_textarea::Input {
                        key: tui_textarea::Key::Delete,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    },
                    KeyCode::Left => tui_textarea::Input {
                        key: tui_textarea::Key::Left,
                        ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
                        alt: key.modifiers.contains(KeyModifiers::ALT),
                        shift: key.modifiers.contains(KeyModifiers::SHIFT),
                    },
                    KeyCode::Right => tui_textarea::Input {
                        key: tui_textarea::Key::Right,
                        ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
                        alt: key.modifiers.contains(KeyModifiers::ALT),
                        shift: key.modifiers.contains(KeyModifiers::SHIFT),
                    },
                    KeyCode::Up => tui_textarea::Input {
                        key: tui_textarea::Key::Up,
                        ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
                        alt: key.modifiers.contains(KeyModifiers::ALT),
                        shift: key.modifiers.contains(KeyModifiers::SHIFT),
                    },
                    KeyCode::Down => tui_textarea::Input {
                        key: tui_textarea::Key::Down,
                        ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
                        alt: key.modifiers.contains(KeyModifiers::ALT),
                        shift: key.modifiers.contains(KeyModifiers::SHIFT),
                    },
                    KeyCode::Home => tui_textarea::Input {
                        key: tui_textarea::Key::Home,
                        ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
                        alt: key.modifiers.contains(KeyModifiers::ALT),
                        shift: key.modifiers.contains(KeyModifiers::SHIFT),
                    },
                    KeyCode::End => tui_textarea::Input {
                        key: tui_textarea::Key::End,
                        ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
                        alt: key.modifiers.contains(KeyModifiers::ALT),
                        shift: key.modifiers.contains(KeyModifiers::SHIFT),
                    },
                    KeyCode::Tab => tui_textarea::Input {
                        key: tui_textarea::Key::Tab,
                        ctrl: false,
                        alt: false,
                        shift: false,
                    },
                    _ => return ComponentResult::Handled,
                };

                // Apply the input to the text area
                self.text_area.input(input);

                // Update the current expression in state and propagate the change
                let expression = self.get_expression();
                state.current_expression = expression.clone();

                ComponentResult::UpdateExpression(expression)
            }
        }
    }

    fn update(&mut self, _state: &mut AppState) -> ComponentResult {
        ComponentResult::Handled
    }

    fn size_constraints(&self) -> SizeConstraints {
        SizeConstraints {
            min_height: Some(3),
            preferred_height: Some(5),
            ..Default::default()
        }
    }
}
