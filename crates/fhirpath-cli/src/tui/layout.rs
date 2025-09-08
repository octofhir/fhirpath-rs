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

//! Layout Management System
//! 
//! This module provides flexible layout management for the TUI panels,
//! supporting responsive design, configurable panel sizes, and dynamic
//! visibility toggling.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use serde::{Deserialize, Serialize};

/// Panel types supported by the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, )]
pub enum PanelType {
    /// Expression input panel
    Input,
    /// Evaluation results panel
    Output,
    /// Diagnostics and error messages panel
    Diagnostics,
    /// Variables display panel
    Variables,
    /// Command history panel  
    History,
    /// Help and documentation panel
    Help,
}

impl std::fmt::Display for PanelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PanelType::Input => write!(f, "Input"),
            PanelType::Output => write!(f, "Output"),
            PanelType::Diagnostics => write!(f, "Diagnostics"),
            PanelType::Variables => write!(f, "Variables"),
            PanelType::History => write!(f, "History"),
            PanelType::Help => write!(f, "Help"),
        }
    }
}

/// Complete panel layout with areas for all panels
#[derive(Debug, Clone)]
pub struct PanelLayout {
    /// Input panel area
    pub input: Rect,
    /// Output panel area
    pub output: Rect,
    /// Diagnostics panel area
    pub diagnostics: Rect,
    /// Variables panel area
    pub variables: Rect,
    /// History panel area
    pub history: Rect,
    /// Help panel area
    pub help: Rect,
    /// Status line area
    pub status_line: Rect,
}

/// Layout manager handles panel positioning and sizing
pub struct LayoutManager {
    config: LayoutConfig,
    focused_panel: PanelType,
    panel_visibility: PanelVisibility,
    terminal_size: (u16, u16),
}

/// Configuration for layout proportions and constraints
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Panel size proportions (0.0 to 1.0)
    pub proportions: PanelProportions,
    /// Minimum panel sizes in characters
    pub minimum_sizes: PanelMinSizes,
    /// Layout mode (determines panel arrangement)
    pub layout_mode: LayoutMode,
    /// Side panel width as percentage of terminal width
    pub side_panel_width_percent: u16,
    /// Status line height
    pub status_line_height: u16,
}

/// Panel size proportions
#[derive(Debug, Clone)]
pub struct PanelProportions {
    /// Input panel height as fraction of main area
    pub input_height: f32,
    /// Output panel height as fraction of main area
    pub output_height: f32,
    /// Diagnostics panel height as fraction of main area
    pub diagnostics_height: f32,
}

/// Minimum panel sizes in characters
#[derive(Debug, Clone)]
pub struct PanelMinSizes {
    /// Minimum input panel height
    pub input_min_height: u16,
    /// Minimum output panel height
    pub output_min_height: u16,
    /// Minimum diagnostics panel height
    pub diagnostics_min_height: u16,
    /// Minimum side panel width
    pub side_panel_min_width: u16,
}

/// Layout arrangement modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, )]
pub enum LayoutMode {
    /// Classic three-panel horizontal layout
    ThreePanel,
    /// Split layout with side panels
    SplitWithSidebar,
    /// Maximized single panel
    Maximized(PanelType),
    /// Custom layout with specific proportions
    Custom,
}

/// Panel visibility state
#[derive(Debug, Clone)]
pub struct PanelVisibility {
    pub input: bool,
    pub output: bool,
    pub diagnostics: bool,
    pub variables: bool,
    pub history: bool,
    pub help: bool,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            proportions: PanelProportions {
                input_height: 0.25,      // 25% for input
                output_height: 0.50,     // 50% for output  
                diagnostics_height: 0.25, // 25% for diagnostics
            },
            minimum_sizes: PanelMinSizes {
                input_min_height: 3,
                output_min_height: 5,
                diagnostics_min_height: 3,
                side_panel_min_width: 20,
            },
            layout_mode: LayoutMode::SplitWithSidebar,
            side_panel_width_percent: 25, // 25% of terminal width
            status_line_height: 1,
        }
    }
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self {
            input: true,
            output: true,
            diagnostics: true,
            variables: true,
            history: false, // Hidden by default
            help: false,    // Hidden by default
        }
    }
}

impl LayoutManager {
    /// Create a new layout manager
    pub fn new(config: LayoutConfig) -> Self {
        Self {
            config,
            focused_panel: PanelType::Input,
            panel_visibility: PanelVisibility::default(),
            terminal_size: (80, 24), // Default terminal size
        }
    }
    
    /// Calculate panel layout for the given terminal size
    pub fn calculate_layout(&mut self, terminal_area: Rect) -> PanelLayout {
        self.terminal_size = (terminal_area.width, terminal_area.height);
        
        match self.config.layout_mode {
            LayoutMode::ThreePanel => self.calculate_three_panel_layout(terminal_area),
            LayoutMode::SplitWithSidebar => self.calculate_split_sidebar_layout(terminal_area),
            LayoutMode::Maximized(panel) => self.calculate_maximized_layout(terminal_area, panel),
            LayoutMode::Custom => self.calculate_custom_layout(terminal_area),
        }
    }
    
    /// Calculate three-panel horizontal layout (input | output | diagnostics)
    fn calculate_three_panel_layout(&self, area: Rect) -> PanelLayout {
        // Reserve space for status line
        let main_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height.saturating_sub(self.config.status_line_height),
        };
        
        let status_area = Rect {
            x: area.x,
            y: area.y + main_area.height,
            width: area.width,
            height: self.config.status_line_height,
        };
        
        // Split main area vertically into three panels
        let constraints = vec![
            Constraint::Ratio(
                (self.config.proportions.input_height * 100.0) as u32,
                100
            ),
            Constraint::Ratio(
                (self.config.proportions.output_height * 100.0) as u32,
                100
            ),
            Constraint::Ratio(
                (self.config.proportions.diagnostics_height * 100.0) as u32,
                100
            ),
        ];
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(main_area);
        
        PanelLayout {
            input: chunks[0],
            output: chunks[1],
            diagnostics: chunks[2],
            variables: Rect::default(), // Not visible in this layout
            history: Rect::default(),   // Not visible in this layout
            help: Rect::default(),      // Not visible in this layout
            status_line: status_area,
        }
    }
    
    /// Calculate split layout with sidebar (main area | sidebar)
    fn calculate_split_sidebar_layout(&self, area: Rect) -> PanelLayout {
        // Reserve space for status line
        let main_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height.saturating_sub(self.config.status_line_height),
        };
        
        let status_area = Rect {
            x: area.x,
            y: area.y + main_area.height,
            width: area.width,
            height: self.config.status_line_height,
        };
        
        // Split horizontally: main panels | side panels
        let horizontal_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(100 - self.config.side_panel_width_percent),
                Constraint::Percentage(self.config.side_panel_width_percent),
            ])
            .split(main_area);
        
        let main_panels_area = horizontal_split[0];
        let side_panels_area = horizontal_split[1];
        
        // Split main panels area vertically
        let main_constraints = vec![
            Constraint::Ratio(
                (self.config.proportions.input_height * 100.0) as u32,
                100
            ),
            Constraint::Ratio(
                (self.config.proportions.output_height * 100.0) as u32,
                100
            ),
            Constraint::Ratio(
                (self.config.proportions.diagnostics_height * 100.0) as u32,
                100
            ),
        ];
        
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(main_constraints)
            .split(main_panels_area);
        
        // Split side panels area vertically for variables and history
        let side_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // Variables get 60%
                Constraint::Percentage(40), // History gets 40%
            ])
            .split(side_panels_area);
        
        PanelLayout {
            input: main_chunks[0],
            output: main_chunks[1],
            diagnostics: main_chunks[2],
            variables: if self.panel_visibility.variables { side_chunks[0] } else { Rect::default() },
            history: if self.panel_visibility.history { side_chunks[1] } else { Rect::default() },
            help: if self.panel_visibility.help { main_area } else { Rect::default() },
            status_line: status_area,
        }
    }
    
    /// Calculate maximized layout for a single panel
    fn calculate_maximized_layout(&self, area: Rect, panel: PanelType) -> PanelLayout {
        let main_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height.saturating_sub(self.config.status_line_height),
        };
        
        let status_area = Rect {
            x: area.x,
            y: area.y + main_area.height,
            width: area.width,
            height: self.config.status_line_height,
        };
        
        let empty_rect = Rect::default();
        
        PanelLayout {
            input: if panel == PanelType::Input { main_area } else { empty_rect },
            output: if panel == PanelType::Output { main_area } else { empty_rect },
            diagnostics: if panel == PanelType::Diagnostics { main_area } else { empty_rect },
            variables: if panel == PanelType::Variables { main_area } else { empty_rect },
            history: if panel == PanelType::History { main_area } else { empty_rect },
            help: if panel == PanelType::Help { main_area } else { empty_rect },
            status_line: status_area,
        }
    }
    
    /// Calculate custom layout based on configuration
    fn calculate_custom_layout(&self, area: Rect) -> PanelLayout {
        // For now, fallback to split sidebar layout
        // This can be extended with more sophisticated custom layout logic
        self.calculate_split_sidebar_layout(area)
    }
    
    /// Set the focused panel
    pub fn set_focused_panel(&mut self, panel: PanelType) {
        self.focused_panel = panel;
    }
    
    /// Get the currently focused panel
    pub fn focused_panel(&self) -> PanelType {
        self.focused_panel
    }
    
    /// Toggle panel visibility
    pub fn toggle_panel_visibility(&mut self, panel: PanelType) {
        match panel {
            PanelType::Input => self.panel_visibility.input = !self.panel_visibility.input,
            PanelType::Output => self.panel_visibility.output = !self.panel_visibility.output,
            PanelType::Diagnostics => self.panel_visibility.diagnostics = !self.panel_visibility.diagnostics,
            PanelType::Variables => self.panel_visibility.variables = !self.panel_visibility.variables,
            PanelType::History => self.panel_visibility.history = !self.panel_visibility.history,
            PanelType::Help => self.panel_visibility.help = !self.panel_visibility.help,
        }
    }
    
    /// Set panel visibility
    pub fn set_panel_visibility(&mut self, panel: PanelType, visible: bool) {
        match panel {
            PanelType::Input => self.panel_visibility.input = visible,
            PanelType::Output => self.panel_visibility.output = visible,
            PanelType::Diagnostics => self.panel_visibility.diagnostics = visible,
            PanelType::Variables => self.panel_visibility.variables = visible,
            PanelType::History => self.panel_visibility.history = visible,
            PanelType::Help => self.panel_visibility.help = visible,
        }
    }
    
    /// Check if panel is visible
    pub fn is_panel_visible(&self, panel: PanelType) -> bool {
        match panel {
            PanelType::Input => self.panel_visibility.input,
            PanelType::Output => self.panel_visibility.output,
            PanelType::Diagnostics => self.panel_visibility.diagnostics,
            PanelType::Variables => self.panel_visibility.variables,
            PanelType::History => self.panel_visibility.history,
            PanelType::Help => self.panel_visibility.help,
        }
    }
    
    /// Handle terminal resize
    pub fn handle_resize(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
        
        // Adjust layout if panels are too small
        self.adjust_for_terminal_size();
    }
    
    /// Adjust layout for current terminal size
    fn adjust_for_terminal_size(&mut self) {
        let (width, height) = self.terminal_size;
        
        // If terminal is too narrow for sidebar layout, switch to three-panel
        if width < 60 && self.config.layout_mode == LayoutMode::SplitWithSidebar {
            self.config.layout_mode = LayoutMode::ThreePanel;
        }
        
        // If terminal is too short, hide some panels
        if height < 15 {
            self.panel_visibility.history = false;
            if height < 10 {
                self.panel_visibility.diagnostics = false;
            }
        }
    }
    
    /// Get next panel in focus order
    pub fn next_panel(&self) -> PanelType {
        let visible_panels = self.get_visible_panels();
        if visible_panels.is_empty() {
            return self.focused_panel;
        }
        
        if let Some(current_index) = visible_panels.iter().position(|&p| p == self.focused_panel) {
            visible_panels[(current_index + 1) % visible_panels.len()]
        } else {
            visible_panels[0]
        }
    }
    
    /// Get previous panel in focus order
    pub fn previous_panel(&self) -> PanelType {
        let visible_panels = self.get_visible_panels();
        if visible_panels.is_empty() {
            return self.focused_panel;
        }
        
        if let Some(current_index) = visible_panels.iter().position(|&p| p == self.focused_panel) {
            let prev_index = if current_index == 0 { 
                visible_panels.len() - 1 
            } else { 
                current_index - 1 
            };
            visible_panels[prev_index]
        } else {
            visible_panels[0]
        }
    }
    
    /// Get list of currently visible panels
    fn get_visible_panels(&self) -> Vec<PanelType> {
        let mut panels = Vec::new();
        
        if self.panel_visibility.input {
            panels.push(PanelType::Input);
        }
        if self.panel_visibility.output {
            panels.push(PanelType::Output);
        }
        if self.panel_visibility.diagnostics {
            panels.push(PanelType::Diagnostics);
        }
        if self.panel_visibility.variables {
            panels.push(PanelType::Variables);
        }
        if self.panel_visibility.history {
            panels.push(PanelType::History);
        }
        if self.panel_visibility.help {
            panels.push(PanelType::Help);
        }
        
        panels
    }
    
    /// Update layout configuration
    pub fn update_config(&mut self, config: LayoutConfig) {
        self.config = config;
        self.adjust_for_terminal_size();
    }
    
    /// Get current layout configuration
    pub fn config(&self) -> &LayoutConfig {
        &self.config
    }
    
    /// Get panel type at given screen coordinates
    pub fn get_panel_at_position(&self, x: u16, y: u16) -> Option<PanelType> {
        // This is a simplified implementation - in a real app you'd use the current layout
        // For now, just return the focused panel or determine based on rough screen areas
        let (width, height) = self.terminal_size;
        
        // Rough panel detection based on layout mode
        match self.config.layout_mode {
            LayoutMode::SplitWithSidebar => {
                let sidebar_width = (width * self.config.side_panel_width_percent) / 100;
                let main_width = width - sidebar_width;
                
                if x >= main_width {
                    // Right sidebar area
                    if y < height / 2 {
                        Some(PanelType::Variables)
                    } else {
                        Some(PanelType::History)
                    }
                } else {
                    // Main area
                    let third = height / 3;
                    if y < third {
                        Some(PanelType::Input)
                    } else if y < third * 2 {
                        Some(PanelType::Output)
                    } else {
                        Some(PanelType::Diagnostics)
                    }
                }
            },
            LayoutMode::ThreePanel => {
                let third = height / 3;
                if y < third {
                    Some(PanelType::Input)
                } else if y < third * 2 {
                    Some(PanelType::Output)
                } else {
                    Some(PanelType::Diagnostics)
                }
            },
            LayoutMode::Maximized(panel) => Some(panel),
            LayoutMode::Custom => {
                // For custom layout, fall back to input panel
                Some(PanelType::Input)
            },
        }
    }
}

/// Responsive layout utilities
pub mod responsive {
    use super::*;
    
    /// Determine optimal layout mode based on terminal size
    pub fn optimal_layout_mode(width: u16, height: u16) -> LayoutMode {
        match (width, height) {
            // Very small terminals: maximize single panel
            (w, h) if w < 40 || h < 10 => LayoutMode::Maximized(PanelType::Input),
            // Small terminals: three panel layout
            (w, _) if w < 80 => LayoutMode::ThreePanel,
            // Medium and large terminals: split with sidebar
            _ => LayoutMode::SplitWithSidebar,
        }
    }
    
    /// Calculate responsive proportions based on content
    pub fn responsive_proportions(
        terminal_height: u16,
        has_diagnostics: bool,
        has_long_output: bool,
    ) -> PanelProportions {
        match (terminal_height, has_diagnostics, has_long_output) {
            // Tall terminal with diagnostics and long output
            (h, true, true) if h > 30 => PanelProportions {
                input_height: 0.2,
                output_height: 0.6,
                diagnostics_height: 0.2,
            },
            // Medium terminal with diagnostics
            (h, true, _) if h > 20 => PanelProportions {
                input_height: 0.25,
                output_height: 0.5,
                diagnostics_height: 0.25,
            },
            // Small terminal or no diagnostics
            _ => PanelProportions {
                input_height: 0.3,
                output_height: 0.7,
                diagnostics_height: 0.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_layout_manager_creation() {
        let config = LayoutConfig::default();
        let manager = LayoutManager::new(config);
        assert_eq!(manager.focused_panel(), PanelType::Input);
    }
    
    #[test]
    fn test_panel_visibility_toggle() {
        let mut manager = LayoutManager::new(LayoutConfig::default());
        
        assert!(manager.is_panel_visible(PanelType::Variables));
        manager.toggle_panel_visibility(PanelType::Variables);
        assert!(!manager.is_panel_visible(PanelType::Variables));
    }
    
    #[test]
    fn test_panel_navigation() {
        let manager = LayoutManager::new(LayoutConfig::default());
        
        // Test that navigation works with visible panels
        let next = manager.next_panel();
        assert_ne!(next, manager.focused_panel());
    }
    
    #[test]
    fn test_responsive_layout_mode() {
        assert_eq!(
            responsive::optimal_layout_mode(30, 10),
            LayoutMode::Maximized(PanelType::Input)
        );
        
        assert_eq!(
            responsive::optimal_layout_mode(60, 20),
            LayoutMode::ThreePanel
        );
        
        assert_eq!(
            responsive::optimal_layout_mode(120, 30),
            LayoutMode::SplitWithSidebar
        );
    }
}