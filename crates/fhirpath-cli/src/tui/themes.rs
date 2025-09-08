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

//! Theme System for TUI
//! 
//! This module provides a comprehensive theming system with support for
//! multiple color schemes, syntax highlighting, and customizable visual styles.

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

use octofhir_fhirpath::diagnostics::ColorScheme as DiagnosticColorScheme;

/// Serializable wrapper for ratatui Color
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SerializableColor {
    Named(String),
    Rgb { r: u8, g: u8, b: u8 },
}

impl From<SerializableColor> for Color {
    fn from(color: SerializableColor) -> Self {
        match color {
            SerializableColor::Named(name) => match name.to_lowercase().as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "gray" => Color::Gray,
                "darkgray" => Color::DarkGray,
                "lightred" => Color::LightRed,
                "lightgreen" => Color::LightGreen,
                "lightyellow" => Color::LightYellow,
                "lightblue" => Color::LightBlue,
                "lightmagenta" => Color::LightMagenta,
                "lightcyan" => Color::LightCyan,
                "white" => Color::White,
                _ => Color::White, // Default fallback
            },
            SerializableColor::Rgb { r, g, b } => Color::Rgb(r, g, b),
        }
    }
}

impl From<Color> for SerializableColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Black => SerializableColor::Named("black".to_string()),
            Color::Red => SerializableColor::Named("red".to_string()),
            Color::Green => SerializableColor::Named("green".to_string()),
            Color::Yellow => SerializableColor::Named("yellow".to_string()),
            Color::Blue => SerializableColor::Named("blue".to_string()),
            Color::Magenta => SerializableColor::Named("magenta".to_string()),
            Color::Cyan => SerializableColor::Named("cyan".to_string()),
            Color::Gray => SerializableColor::Named("gray".to_string()),
            Color::DarkGray => SerializableColor::Named("darkgray".to_string()),
            Color::LightRed => SerializableColor::Named("lightred".to_string()),
            Color::LightGreen => SerializableColor::Named("lightgreen".to_string()),
            Color::LightYellow => SerializableColor::Named("lightyellow".to_string()),
            Color::LightBlue => SerializableColor::Named("lightblue".to_string()),
            Color::LightMagenta => SerializableColor::Named("lightmagenta".to_string()),
            Color::LightCyan => SerializableColor::Named("lightcyan".to_string()),
            Color::White => SerializableColor::Named("white".to_string()),
            Color::Rgb(r, g, b) => SerializableColor::Rgb { r, g, b },
            Color::Indexed(i) => SerializableColor::Named(format!("indexed_{}", i)),
            Color::Reset => SerializableColor::Named("reset".to_string()),
        }
    }
}

/// Complete TUI theme definition
#[derive(Debug, Clone)]
pub struct TuiTheme {
    /// Theme metadata
    pub metadata: ThemeMetadata,
    /// Color scheme for UI elements
    pub colors: ColorScheme,
    /// Syntax highlighting colors
    pub syntax: SyntaxTheme,
    /// Diagnostic message colors
    pub diagnostic_colors: DiagnosticColorScheme,
    /// UI element styles
    pub styles: StyleTheme,
}

/// Theme metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMetadata {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub dark_mode: bool,
}

/// Color scheme for UI elements
#[derive(Debug, Clone)]
pub struct ColorScheme {
    // Border colors
    pub focused_border: Color,
    pub unfocused_border: Color,
    
    // Background colors
    pub background: Color,
    pub selected_background: Color,
    pub active_background: Color,
    
    // Text colors
    pub normal_text: Color,
    pub selected_text: Color,
    pub disabled_text: Color,
    pub highlight_text: Color,
    
    // Status colors
    pub success_color: Color,
    pub warning_color: Color,
    pub error_color: Color,
    pub info_color: Color,
    
    // Special UI colors
    pub cursor_color: Color,
    pub selection_color: Color,
    pub match_color: Color,
    pub line_number_color: Color,
    
    // Panel-specific colors
    pub input_background: Option<Color>,
    pub output_background: Option<Color>,
    pub diagnostics_background: Option<Color>,
}

/// Syntax highlighting color scheme
#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    // FHIRPath language elements
    pub function_color: Color,
    pub property_color: Color,
    pub resource_type_color: Color,
    pub operator_color: Color,
    pub keyword_color: Color,
    
    // Literals
    pub string_color: Color,
    pub number_color: Color,
    pub boolean_color: Color,
    pub null_color: Color,
    
    // Punctuation and symbols
    pub punctuation_color: Color,
    pub bracket_color: Color,
    pub parenthesis_color: Color,
    
    // Variables and identifiers
    pub variable_color: Color,
    pub identifier_color: Color,
    
    // Comments (for help text)
    pub comment_color: Color,
    
    // Error highlighting
    pub syntax_error_color: Color,
    pub type_error_color: Color,
}

/// Style definitions for UI elements
#[derive(Debug, Clone)]
pub struct StyleTheme {
    // Text styles
    pub normal: Style,
    pub bold: Style,
    pub italic: Style,
    pub underline: Style,
    
    // Panel styles
    pub focused_panel: Style,
    pub unfocused_panel: Style,
    
    // Status line styles
    pub status_normal: Style,
    pub status_active: Style,
    pub status_error: Style,
    
    // Selection styles
    pub selected_item: Style,
    pub highlighted_item: Style,
    
    // Input styles
    pub input_normal: Style,
    pub input_error: Style,
    pub cursor_style: Style,
}

impl Default for TuiTheme {
    fn default() -> Self {
        Self::dark_theme()
    }
}

impl TuiTheme {
    /// Create a dark theme (default)
    pub fn dark_theme() -> Self {
        Self {
            metadata: ThemeMetadata {
                name: "Dark".to_string(),
                description: "Professional dark theme for FHIRPath TUI".to_string(),
                author: "OctoFHIR Team".to_string(),
                version: "1.0.0".to_string(),
                dark_mode: true,
            },
            colors: ColorScheme {
                focused_border: Color::Cyan,
                unfocused_border: Color::DarkGray,
                background: Color::Black,
                selected_background: Color::DarkGray,
                active_background: Color::Blue,
                normal_text: Color::White,
                selected_text: Color::Yellow,
                disabled_text: Color::DarkGray,
                highlight_text: Color::Cyan,
                success_color: Color::Green,
                warning_color: Color::Yellow,
                error_color: Color::Red,
                info_color: Color::Blue,
                cursor_color: Color::White,
                selection_color: Color::Blue,
                match_color: Color::Magenta,
                line_number_color: Color::DarkGray,
                input_background: None,
                output_background: None,
                diagnostics_background: None,
            },
            syntax: SyntaxTheme {
                function_color: Color::Blue,
                property_color: Color::Cyan,
                resource_type_color: Color::Yellow,
                operator_color: Color::Magenta,
                keyword_color: Color::Red,
                string_color: Color::Green,
                number_color: Color::Yellow,
                boolean_color: Color::Magenta,
                null_color: Color::DarkGray,
                punctuation_color: Color::White,
                bracket_color: Color::Yellow,
                parenthesis_color: Color::Yellow,
                variable_color: Color::Cyan,
                identifier_color: Color::White,
                comment_color: Color::DarkGray,
                syntax_error_color: Color::Red,
                type_error_color: Color::Red,
            },
            diagnostic_colors: DiagnosticColorScheme::default(),
            styles: StyleTheme::dark(),
        }
    }
    
    /// Create a light theme
    pub fn light_theme() -> Self {
        Self {
            metadata: ThemeMetadata {
                name: "Light".to_string(),
                description: "Professional light theme for FHIRPath TUI".to_string(),
                author: "OctoFHIR Team".to_string(),
                version: "1.0.0".to_string(),
                dark_mode: false,
            },
            colors: ColorScheme {
                focused_border: Color::Blue,
                unfocused_border: Color::Gray,
                background: Color::White,
                selected_background: Color::LightBlue,
                active_background: Color::Blue,
                normal_text: Color::Black,
                selected_text: Color::Blue,
                disabled_text: Color::Gray,
                highlight_text: Color::Blue,
                success_color: Color::Green,
                warning_color: Color::Red, // More visible in light theme
                error_color: Color::Red,
                info_color: Color::Blue,
                cursor_color: Color::Black,
                selection_color: Color::Blue,
                match_color: Color::Magenta,
                line_number_color: Color::Gray,
                input_background: None,
                output_background: None,
                diagnostics_background: None,
            },
            syntax: SyntaxTheme {
                function_color: Color::Blue,
                property_color: Color::DarkGray,
                resource_type_color: Color::Red,
                operator_color: Color::Magenta,
                keyword_color: Color::Blue,
                string_color: Color::Green,
                number_color: Color::Red,
                boolean_color: Color::Magenta,
                null_color: Color::Gray,
                punctuation_color: Color::Black,
                bracket_color: Color::Red,
                parenthesis_color: Color::Red,
                variable_color: Color::DarkGray,
                identifier_color: Color::Black,
                comment_color: Color::Gray,
                syntax_error_color: Color::Red,
                type_error_color: Color::Red,
            },
            diagnostic_colors: DiagnosticColorScheme::default(),
            styles: StyleTheme::light(),
        }
    }
    
    /// Create a high contrast theme for accessibility
    pub fn high_contrast_theme() -> Self {
        Self {
            metadata: ThemeMetadata {
                name: "High Contrast".to_string(),
                description: "High contrast theme for accessibility".to_string(),
                author: "OctoFHIR Team".to_string(),
                version: "1.0.0".to_string(),
                dark_mode: true,
            },
            colors: ColorScheme {
                focused_border: Color::White,
                unfocused_border: Color::Gray,
                background: Color::Black,
                selected_background: Color::White,
                active_background: Color::White,
                normal_text: Color::White,
                selected_text: Color::Black,
                disabled_text: Color::Gray,
                highlight_text: Color::Yellow,
                success_color: Color::Green,
                warning_color: Color::Yellow,
                error_color: Color::Red,
                info_color: Color::Cyan,
                cursor_color: Color::Yellow,
                selection_color: Color::White,
                match_color: Color::Yellow,
                line_number_color: Color::Gray,
                input_background: None,
                output_background: None,
                diagnostics_background: None,
            },
            syntax: SyntaxTheme {
                function_color: Color::Yellow,
                property_color: Color::White,
                resource_type_color: Color::Cyan,
                operator_color: Color::Yellow,
                keyword_color: Color::Red,
                string_color: Color::Green,
                number_color: Color::Cyan,
                boolean_color: Color::Yellow,
                null_color: Color::Gray,
                punctuation_color: Color::White,
                bracket_color: Color::Yellow,
                parenthesis_color: Color::Yellow,
                variable_color: Color::White,
                identifier_color: Color::White,
                comment_color: Color::Gray,
                syntax_error_color: Color::Red,
                type_error_color: Color::Red,
            },
            diagnostic_colors: DiagnosticColorScheme::default(),
            styles: StyleTheme::high_contrast(),
        }
    }
    
    /// Get available built-in themes
    pub fn available_themes() -> Vec<String> {
        vec![
            "dark".to_string(),
            "light".to_string(),
            "high_contrast".to_string(),
        ]
    }
    
    /// Load theme by name
    pub fn load_theme(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "dark" => Some(Self::dark_theme()),
            "light" => Some(Self::light_theme()),
            "high_contrast" | "high-contrast" => Some(Self::high_contrast_theme()),
            _ => None,
        }
    }
    
    /// Get style for FHIRPath token type
    pub fn get_syntax_style(&self, token_type: &str) -> Style {
        let color = match token_type {
            "function" => self.syntax.function_color,
            "property" => self.syntax.property_color,
            "resource_type" => self.syntax.resource_type_color,
            "operator" => self.syntax.operator_color,
            "keyword" => self.syntax.keyword_color,
            "string" => self.syntax.string_color,
            "number" => self.syntax.number_color,
            "boolean" => self.syntax.boolean_color,
            "null" => self.syntax.null_color,
            "punctuation" => self.syntax.punctuation_color,
            "bracket" => self.syntax.bracket_color,
            "parenthesis" => self.syntax.parenthesis_color,
            "variable" => self.syntax.variable_color,
            "identifier" => self.syntax.identifier_color,
            "comment" => self.syntax.comment_color,
            "syntax_error" => self.syntax.syntax_error_color,
            "type_error" => self.syntax.type_error_color,
            _ => self.colors.normal_text,
        };
        
        Style::default().fg(color)
    }
    
    /// Get style for diagnostic severity
    pub fn get_diagnostic_style(&self, severity: &str) -> Style {
        let color = match severity.to_lowercase().as_str() {
            "error" => self.colors.error_color,
            "warning" => self.colors.warning_color,
            "info" | "information" => self.colors.info_color,
            "hint" => self.colors.disabled_text,
            _ => self.colors.normal_text,
        };
        
        Style::default().fg(color)
    }
    
    /// Get style for result type
    pub fn get_result_style(&self, result_type: &str) -> Style {
        let color = match result_type {
            "success" => self.colors.success_color,
            "error" => self.colors.error_color,
            "warning" => self.colors.warning_color,
            "info" => self.colors.info_color,
            _ => self.colors.normal_text,
        };
        
        Style::default().fg(color)
    }
}

impl StyleTheme {
    pub fn dark() -> Self {
        Self {
            normal: Style::default().fg(Color::White),
            bold: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            italic: Style::default().fg(Color::White).add_modifier(Modifier::ITALIC),
            underline: Style::default().fg(Color::White).add_modifier(Modifier::UNDERLINED),
            focused_panel: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            unfocused_panel: Style::default().fg(Color::DarkGray),
            status_normal: Style::default().fg(Color::White).bg(Color::DarkGray),
            status_active: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            status_error: Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
            selected_item: Style::default()
                .fg(Color::Yellow)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            highlighted_item: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            input_normal: Style::default().fg(Color::White),
            input_error: Style::default().fg(Color::Red),
            cursor_style: Style::default()
                .bg(Color::White)
                .fg(Color::Black),
        }
    }
    
    pub fn light() -> Self {
        Self {
            normal: Style::default().fg(Color::Black),
            bold: Style::default().fg(Color::Black).add_modifier(Modifier::BOLD),
            italic: Style::default().fg(Color::Black).add_modifier(Modifier::ITALIC),
            underline: Style::default().fg(Color::Black).add_modifier(Modifier::UNDERLINED),
            focused_panel: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            unfocused_panel: Style::default().fg(Color::Gray),
            status_normal: Style::default().fg(Color::Black).bg(Color::LightBlue),
            status_active: Style::default()
                .fg(Color::White)
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            status_error: Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
            selected_item: Style::default()
                .fg(Color::Blue)
                .bg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
            highlighted_item: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            input_normal: Style::default().fg(Color::Black),
            input_error: Style::default().fg(Color::Red),
            cursor_style: Style::default()
                .bg(Color::Black)
                .fg(Color::White),
        }
    }
    
    pub fn high_contrast() -> Self {
        Self {
            normal: Style::default().fg(Color::White),
            bold: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            italic: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
            underline: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::UNDERLINED),
            focused_panel: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            unfocused_panel: Style::default().fg(Color::Gray),
            status_normal: Style::default()
                .fg(Color::Black)
                .bg(Color::White),
            status_active: Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            status_error: Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
            selected_item: Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
            highlighted_item: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            input_normal: Style::default().fg(Color::White),
            input_error: Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
            cursor_style: Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black),
        }
    }
}

/// Theme utilities and helpers
pub mod utils {
    use super::*;
    
    /// Blend two colors (simple average)
    pub fn blend_colors(color1: Color, color2: Color, ratio: f32) -> Color {
        // This is a simplified color blending - in a real implementation
        // you'd want to convert to RGB, blend, and convert back
        if ratio <= 0.0 {
            color1
        } else if ratio >= 1.0 {
            color2
        } else {
            color2 // Simplified - just return the second color
        }
    }
    
    /// Get contrasting color for text readability
    pub fn contrasting_color(background: Color) -> Color {
        match background {
            Color::Black | Color::DarkGray | Color::Red | Color::Blue => Color::White,
            Color::White | Color::LightBlue | Color::Yellow | Color::Cyan => Color::Black,
            _ => Color::White, // Default to white for most colors
        }
    }
    
    /// Darken a color
    pub fn darken_color(color: Color) -> Color {
        match color {
            Color::White => Color::Gray,
            Color::Gray => Color::DarkGray,
            Color::Yellow => Color::Red,
            Color::Green => Color::DarkGray,
            Color::Blue => Color::DarkGray,
            Color::Cyan => Color::Blue,
            Color::Magenta => Color::Red,
            _ => color,
        }
    }
    
    /// Lighten a color
    pub fn lighten_color(color: Color) -> Color {
        match color {
            Color::Black => Color::DarkGray,
            Color::DarkGray => Color::Gray,
            Color::Gray => Color::White,
            Color::Red => Color::Yellow,
            Color::Blue => Color::Cyan,
            _ => color,
        }
    }
    
    /// Check if theme is suitable for current terminal
    pub fn validate_theme_compatibility(theme: &TuiTheme) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Check for 16-color compatibility
        if uses_extended_colors(theme) {
            issues.push("Theme uses extended colors that may not be supported in all terminals".to_string());
        }
        
        // Check for adequate contrast
        if !has_adequate_contrast(theme) {
            issues.push("Theme may have insufficient contrast for accessibility".to_string());
        }
        
        issues
    }
    
    fn uses_extended_colors(theme: &TuiTheme) -> bool {
        // Check if theme uses colors beyond the basic 16-color palette
        // This is a simplified check
        matches!(theme.colors.focused_border, Color::Rgb(_, _, _))
    }
    
    fn has_adequate_contrast(theme: &TuiTheme) -> bool {
        // Simplified contrast check
        // In a real implementation, you'd calculate luminance ratios
        theme.colors.normal_text != theme.colors.background
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_theme_creation() {
        let dark_theme = TuiTheme::dark_theme();
        assert!(dark_theme.metadata.dark_mode);
        assert_eq!(dark_theme.metadata.name, "Dark");
        
        let light_theme = TuiTheme::light_theme();
        assert!(!light_theme.metadata.dark_mode);
        assert_eq!(light_theme.metadata.name, "Light");
    }
    
    #[test]
    fn test_theme_loading() {
        assert!(TuiTheme::load_theme("dark").is_some());
        assert!(TuiTheme::load_theme("light").is_some());
        assert!(TuiTheme::load_theme("high_contrast").is_some());
        assert!(TuiTheme::load_theme("nonexistent").is_none());
    }
    
    #[test]
    fn test_syntax_style() {
        let theme = TuiTheme::dark_theme();
        let function_style = theme.get_syntax_style("function");
        assert_eq!(function_style.fg, Some(Color::Blue));
        
        let unknown_style = theme.get_syntax_style("unknown");
        assert_eq!(unknown_style.fg, Some(theme.colors.normal_text));
    }
    
    #[test]
    fn test_diagnostic_style() {
        let theme = TuiTheme::dark_theme();
        let error_style = theme.get_diagnostic_style("error");
        assert_eq!(error_style.fg, Some(Color::Red));
        
        let warning_style = theme.get_diagnostic_style("warning");
        assert_eq!(warning_style.fg, Some(Color::Yellow));
    }
    
    #[test]
    fn test_available_themes() {
        let themes = TuiTheme::available_themes();
        assert!(themes.contains(&"dark".to_string()));
        assert!(themes.contains(&"light".to_string()));
        assert!(themes.contains(&"high_contrast".to_string()));
    }
}