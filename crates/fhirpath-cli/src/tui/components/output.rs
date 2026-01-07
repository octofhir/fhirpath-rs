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

use std::cmp::min;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::KeyEvent;
use octofhir_fhirpath::FhirPathValue;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Wrap,
};

use super::{ComponentResult, ScrollState, SizeConstraints, TuiComponent, utils};
use crate::tui::app::AppState;
use crate::tui::config::TuiConfig;
use crate::tui::layout::PanelType;
use crate::tui::themes::TuiTheme;

/// Output panel for displaying FHIRPath evaluation results
pub struct OutputPanel {
    scroll_state: ScrollState,
    total_items: usize,
    visible_items: usize,
}

impl OutputPanel {
    /// Create a new output panel
    pub async fn new(_config: &TuiConfig) -> Result<Self> {
        Ok(Self {
            scroll_state: ScrollState::new(),
            total_items: 0,
            visible_items: 0,
        })
    }

    fn render_collection(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        collection: &octofhir_fhirpath::Collection,
        state: &AppState,
        theme: &TuiTheme,
    ) {
        self.total_items = collection.len();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        let visible_range = self.render_collection_items(frame, layout[1], collection, theme);

        let summary_line = self.build_summary_line(collection, state, theme, visible_range);
        let summary = Paragraph::new(summary_line).wrap(Wrap { trim: true });
        frame.render_widget(summary, layout[0]);

        if self.total_items > self.visible_items && self.visible_items > 0 {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .track_symbol(Some(" "))
                .thumb_symbol("|");
            let mut state = ScrollbarState::new(self.total_items)
                .position(self.scroll_state.offset)
                .viewport_content_length(self.visible_items.max(1));
            let mut area_for_scroll = layout[1];
            if area_for_scroll.width > 0 {
                area_for_scroll.width = area_for_scroll.width.saturating_add(1);
            }
            frame.render_stateful_widget(scrollbar, area_for_scroll, &mut state);
        }
    }

    fn render_collection_items(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        collection: &octofhir_fhirpath::Collection,
        theme: &TuiTheme,
    ) -> Option<(usize, usize)> {
        let mut available_rows = area.height as usize;
        if available_rows == 0 {
            self.visible_items = 0;
            return None;
        }

        // Account for top border when rendering list block
        let list_block = Block::default().borders(Borders::TOP);
        let inner_area = list_block.inner(area);
        available_rows = inner_area.height.max(1) as usize;

        self.visible_items = available_rows.min(self.total_items);
        let max_offset = self.total_items.saturating_sub(self.visible_items);
        if self.scroll_state.offset > max_offset {
            self.scroll_state.offset = max_offset;
        }

        let start = self.scroll_state.offset;
        let max_preview_width = inner_area.width.saturating_sub(10) as usize;

        let items: Vec<ListItem> = if self.total_items == 0 {
            vec![ListItem::new(Line::from(vec![Span::styled(
                "Collection is empty",
                Style::default().fg(theme.colors.disabled_text),
            )]))]
        } else {
            collection
                .values()
                .iter()
                .enumerate()
                .skip(start)
                .take(self.visible_items)
                .map(|(idx, value)| self.build_value_item(idx, value, max_preview_width, theme))
                .collect()
        };

        let list = List::new(items).block(list_block);
        frame.render_widget(list, area);

        if self.total_items == 0 {
            None
        } else {
            let end = min(self.total_items, start.saturating_add(self.visible_items));
            Some((start.saturating_add(1), end))
        }
    }

    fn render_empty_state(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        theme: &TuiTheme,
        message: &str,
    ) {
        self.total_items = 0;
        self.visible_items = 0;
        let paragraph = Paragraph::new(message)
            .style(Style::default().fg(theme.colors.disabled_text))
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    fn render_error_state(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        theme: &TuiTheme,
        error: &str,
        duration: Duration,
    ) {
        self.total_items = 0;
        self.visible_items = 0;
        let content = vec![
            Line::from(vec![Span::styled(
                "Evaluation failed",
                Style::default()
                    .fg(theme.colors.error_color)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::raw(format!(
                "Duration: {}",
                Self::format_duration(duration)
            ))]),
            Line::from(vec![
                Span::raw(""),
                Span::styled(error, Style::default().fg(theme.colors.error_color)),
            ]),
        ];
        let paragraph = Paragraph::new(content).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    fn build_summary_line(
        &self,
        collection: &octofhir_fhirpath::Collection,
        state: &AppState,
        theme: &TuiTheme,
        visible_range: Option<(usize, usize)>,
    ) -> Line<'_> {
        let mut spans = Vec::new();

        spans.push(Span::styled(
            format!(" {} items ", collection.len()),
            Style::default()
                .bg(theme.colors.focused_border)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ));

        if let Some(first) = collection.first() {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                format!("type {}", first.display_type_name()),
                Style::default().fg(theme.colors.highlight_text),
            ));
        }

        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            if collection.is_ordered() {
                "ordered"
            } else {
                "unordered"
            },
            Style::default().fg(theme.colors.info_color),
        ));

        if let Some((start, end)) = visible_range {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                format!("showing {}-{}", start, end),
                Style::default().fg(theme.colors.selected_text),
            ));
        }

        if let Some(duration) = state.performance.last_evaluation_time {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                format!("last {}", Self::format_duration(duration)),
                Style::default().fg(theme.colors.success_color),
            ));
        }

        if state.performance.total_evaluations > 1 {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                format!(
                    "avg {}",
                    Self::format_duration(state.performance.average_evaluation_time)
                ),
                Style::default().fg(theme.colors.normal_text),
            ));
        }

        Line::from(spans)
    }

    fn build_value_item(
        &self,
        index: usize,
        value: &FhirPathValue,
        max_preview_width: usize,
        theme: &TuiTheme,
    ) -> ListItem<'_> {
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled(
            format!("{:>4}", index + 1),
            Style::default().fg(theme.colors.line_number_color),
        ));
        spans.push(Span::raw("  "));

        let (type_label, type_style) = Self::type_label_and_style(value, theme);
        spans.push(Span::styled(type_label, type_style));
        spans.push(Span::raw("  "));

        let preview = Self::value_preview(value, max_preview_width);
        spans.push(Span::styled(
            preview,
            Style::default().fg(theme.colors.normal_text),
        ));

        if let Some(meta) = Self::value_metadata(value) {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                meta,
                Style::default()
                    .fg(theme.colors.disabled_text)
                    .add_modifier(Modifier::ITALIC),
            ));
        }

        ListItem::new(Line::from(spans))
    }

    fn type_label_and_style(value: &FhirPathValue, theme: &TuiTheme) -> (String, Style) {
        match value {
            FhirPathValue::Boolean(v, _, _) => (
                format!("bool:{}", v),
                Style::default().fg(if *v {
                    theme.colors.success_color
                } else {
                    theme.colors.warning_color
                }),
            ),
            FhirPathValue::Integer(_, _, _) => (
                "integer".to_string(),
                Style::default()
                    .fg(theme.colors.highlight_text)
                    .add_modifier(Modifier::BOLD),
            ),
            FhirPathValue::Decimal(_, _, _) => (
                "decimal".to_string(),
                Style::default()
                    .fg(theme.colors.highlight_text)
                    .add_modifier(Modifier::BOLD),
            ),
            FhirPathValue::String(_, _, _) => (
                "string".to_string(),
                Style::default().fg(theme.colors.info_color),
            ),
            FhirPathValue::Date(_, _, _) => (
                "date".to_string(),
                Style::default().fg(theme.colors.info_color),
            ),
            FhirPathValue::DateTime(_, _, _) => (
                "datetime".to_string(),
                Style::default().fg(theme.colors.info_color),
            ),
            FhirPathValue::Time(_, _, _) => (
                "time".to_string(),
                Style::default().fg(theme.colors.info_color),
            ),
            FhirPathValue::Quantity { .. } => (
                "quantity".to_string(),
                Style::default().fg(theme.colors.highlight_text),
            ),
            FhirPathValue::Resource(_, type_info, _) => (
                type_info.type_name.clone(),
                Style::default()
                    .fg(theme.colors.selected_text)
                    .add_modifier(Modifier::BOLD),
            ),
            FhirPathValue::Collection(collection) => (
                format!("collection({})", collection.len()),
                Style::default().fg(theme.colors.highlight_text),
            ),
            FhirPathValue::Empty => (
                "empty".to_string(),
                Style::default().fg(theme.colors.disabled_text),
            ),
        }
    }

    fn value_preview(value: &FhirPathValue, max_width: usize) -> String {
        let raw = match value {
            FhirPathValue::Boolean(v, _, _) => v.to_string(),
            FhirPathValue::Integer(i, _, _) => i.to_string(),
            FhirPathValue::Decimal(d, _, _) => d.normalize().to_string(),
            FhirPathValue::String(s, _, _) => format!("\"{}\"", s),
            FhirPathValue::Date(d, _, _) => d.to_string(),
            FhirPathValue::DateTime(dt, _, _) => dt.to_string(),
            FhirPathValue::Time(t, _, _) => t.to_string(),
            FhirPathValue::Quantity { value, unit, .. } => unit
                .as_ref()
                .map(|u| format!("{} {}", value, u))
                .unwrap_or_else(|| value.to_string()),
            FhirPathValue::Resource(json, type_info, _) => json
                .as_object()
                .and_then(|obj| obj.get("id"))
                .and_then(|id| id.as_str())
                .map(|id| format!("{}#{}", type_info.type_name, id))
                .unwrap_or_else(|| type_info.type_name.clone()),
            FhirPathValue::Collection(collection) => {
                format!("[{} items]", collection.len())
            }
            FhirPathValue::Empty => "empty".to_string(),
        };

        Self::truncate(&raw, max_width)
    }

    fn value_metadata(value: &FhirPathValue) -> Option<String> {
        match value {
            FhirPathValue::String(s, _, _) => Some(format!("len {}", s.chars().count())),
            FhirPathValue::Collection(collection) => Some(format!("items {}", collection.len())),
            FhirPathValue::Resource(json, _, _) => json
                .as_object()
                .and_then(|obj| obj.get("resourceType"))
                .and_then(|t| t.as_str())
                .map(|t| format!("resourceType {}", t)),
            _ => None,
        }
    }

    fn truncate(text: &str, max_width: usize) -> String {
        if max_width == 0 || text.chars().count() <= max_width {
            return text.to_string();
        }
        let mut truncated = String::new();
        for ch in text.chars() {
            if truncated.chars().count() + 1 >= max_width {
                break;
            }
            truncated.push(ch);
        }
        truncated.push_str("...");
        truncated
    }

    fn format_duration(duration: Duration) -> String {
        if duration.as_millis() >= 1000 {
            format!("{:.1}s", duration.as_secs_f32())
        } else {
            format!("{}ms", duration.as_millis())
        }
    }
}

impl TuiComponent for OutputPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState, theme: &TuiTheme) {
        let is_focused = state.focused_panel == PanelType::Output;
        let block = utils::create_panel_block("Output", PanelType::Output, is_focused, theme);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if let Some(collection) = &state.last_result {
            self.render_collection(frame, inner, collection, state, theme);
        } else if let Some(history) = state.evaluation_history.last() {
            if let Err(error) = &history.result {
                self.render_error_state(frame, inner, theme, error, history.execution_time);
            } else {
                self.render_empty_state(
                    frame,
                    inner,
                    theme,
                    "Previous evaluation produced no output.",
                );
            }
        } else {
            self.render_empty_state(
                frame,
                inner,
                theme,
                "No results yet. Execute an expression to see output here.",
            );
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent, _state: &mut AppState) -> ComponentResult {
        use crossterm::event::KeyCode;

        if self.visible_items == 0 {
            return ComponentResult::Handled;
        }

        match key.code {
            KeyCode::Up => {
                self.scroll_state.scroll_up();
                ComponentResult::Handled
            }
            KeyCode::Down => {
                self.scroll_state
                    .scroll_down(self.total_items, self.visible_items.max(1));
                ComponentResult::Handled
            }
            KeyCode::PageUp => {
                let step = self.visible_items.max(1);
                if self.scroll_state.offset >= step {
                    self.scroll_state.offset -= step;
                } else {
                    self.scroll_state.offset = 0;
                }
                ComponentResult::Handled
            }
            KeyCode::PageDown => {
                let step = self.visible_items.max(1);
                let max_offset = self.total_items.saturating_sub(step);
                self.scroll_state.offset = (self.scroll_state.offset + step).min(max_offset);
                ComponentResult::Handled
            }
            KeyCode::Home => {
                self.scroll_state.offset = 0;
                ComponentResult::Handled
            }
            KeyCode::End => {
                let visible = self.visible_items.max(1);
                self.scroll_state.offset = self.total_items.saturating_sub(visible);
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
