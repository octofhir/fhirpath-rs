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

//! TUI Configuration System

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono;
use serde::{Deserialize, Serialize};

use super::events::KeyBindings;
use super::layout::LayoutConfig;
use super::themes::TuiTheme;

/// Serializable subset of TUI configuration (for persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableConfig {
    metadata: ConfigMetadata,
    features: FeatureFlags,
    performance: PerformanceConfig,
    ui_preferences: UiPreferences,
    theme_name: String,
    key_bindings: HashMap<String, String>,
}

/// Complete TUI configuration
#[derive(Debug, Clone)]
pub struct TuiConfig {
    pub metadata: ConfigMetadata,
    pub layout: LayoutConfig,
    pub theme: TuiTheme,
    pub key_bindings: HashMap<String, String>,
    pub features: FeatureFlags,
    pub performance: PerformanceConfig,
    pub ui_preferences: UiPreferences,
}

/// Configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub version: String,
    pub created_at: String,
    pub last_modified: String,
    pub user: String,
}

/// Feature flags for optional functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable real-time syntax highlighting
    pub syntax_highlighting: bool,

    /// Enable auto-completion
    pub auto_completion: bool,

    /// Enable real-time expression validation
    pub real_time_validation: bool,

    /// Enable mouse support
    pub mouse_support: bool,

    /// Enable advanced text editing features
    pub advanced_editing: bool,

    /// Enable performance monitoring
    pub performance_monitoring: bool,

    /// Enable diagnostic details
    pub diagnostic_details: bool,

    /// Enable history persistence
    pub persistent_history: bool,

    /// Enable configuration auto-save
    pub auto_save_config: bool,
}

/// Performance-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Target FPS for rendering
    pub target_fps: u16,

    /// Maximum number of history entries to keep
    pub max_history_entries: usize,

    /// Maximum number of completion suggestions
    pub max_completions: usize,

    /// Debounce delay for real-time validation (ms)
    pub validation_debounce_ms: u64,

    /// Syntax highlighting cache size
    pub syntax_cache_size: usize,

    /// Enable performance profiling
    pub enable_profiling: bool,

    /// Render optimization level
    pub render_optimization: RenderOptimization,
}

/// Render optimization levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RenderOptimization {
    /// No optimizations (best quality)
    None,
    /// Basic optimizations (balanced)
    Basic,
    /// Aggressive optimizations (best performance)
    Aggressive,
}

/// User interface preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub show_line_numbers: bool,
    pub show_cursor_position: bool,
    pub show_performance_info: bool,
    pub auto_focus_input: bool,
    pub confirm_exit: bool,
    pub save_window_state: bool,

    /// Default output format
    pub default_output_format: String,

    pub animation_duration_ms: u64,
    pub show_tooltips: bool,
    pub tooltip_delay_ms: u64,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            metadata: ConfigMetadata {
                version: "1.0.0".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                last_modified: chrono::Utc::now().to_rfc3339(),
                user: std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            },
            layout: LayoutConfig::default(),
            theme: TuiTheme::default(),
            key_bindings: HashMap::new(),
            features: FeatureFlags::default(),
            performance: PerformanceConfig::default(),
            ui_preferences: UiPreferences::default(),
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            syntax_highlighting: true,
            auto_completion: true,
            real_time_validation: true,
            mouse_support: true,
            advanced_editing: true,
            performance_monitoring: false, // Disabled by default for performance
            diagnostic_details: true,
            persistent_history: true,
            auto_save_config: true,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            max_history_entries: 1000,
            max_completions: 50,
            validation_debounce_ms: 250,
            syntax_cache_size: 100,
            enable_profiling: false,
            render_optimization: RenderOptimization::Basic,
        }
    }
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            show_line_numbers: false,
            show_cursor_position: true,
            show_performance_info: false,
            auto_focus_input: true,
            confirm_exit: false,
            save_window_state: true,
            default_output_format: "pretty".to_string(),
            animation_duration_ms: 150,
            show_tooltips: true,
            tooltip_delay_ms: 500,
        }
    }
}

impl TuiConfig {
    /// Load configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content =
            std::fs::read_to_string(path.as_ref()).context("Failed to read configuration file")?;

        // Deserialize the serializable subset
        let serializable: SerializableConfig =
            toml::from_str(&content).context("Failed to parse TOML configuration")?;

        // Create full config from serializable subset
        let mut config = Self::default();

        // Apply loaded settings
        config.metadata = serializable.metadata;
        config.features = serializable.features;
        config.performance = serializable.performance;
        config.ui_preferences = serializable.ui_preferences;
        config.key_bindings = serializable.key_bindings;

        // Load theme by name (keep default if theme not found)
        config.theme =
            TuiTheme::load_theme(&serializable.theme_name).unwrap_or_else(|| TuiTheme::default());

        // Update last modified time
        config.metadata.last_modified = chrono::Utc::now().to_rfc3339();

        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        // Update last modified time
        self.metadata.last_modified = chrono::Utc::now().to_rfc3339();

        // Create serializable subset
        let serializable = SerializableConfig {
            metadata: self.metadata.clone(),
            features: self.features.clone(),
            performance: self.performance.clone(),
            ui_preferences: self.ui_preferences.clone(),
            theme_name: self.theme.metadata.name.clone(),
            key_bindings: self.key_bindings.clone(),
        };

        // Serialize to TOML
        let content =
            toml::to_string_pretty(&serializable).context("Failed to serialize configuration")?;

        // Add header comment
        let content_with_header = format!(
            "# FHIRPath TUI Configuration\n\
             # Generated at {}\n\
             \n\
             {}",
            chrono::Utc::now().to_rfc3339(),
            content
        );

        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).context("Failed to create configuration directory")?;
        }

        // Write to file
        std::fs::write(path.as_ref(), content_with_header)
            .context("Failed to write configuration file")?;

        Ok(())
    }

    /// Get default configuration file path
    pub fn default_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Failed to get user config directory")?;

        Ok(config_dir.join("fhirpath-tui").join("config.toml"))
    }

    /// Load configuration with fallbacks
    pub fn load_with_fallbacks() -> Result<Self> {
        // Try to load from default location
        if let Ok(config_path) = Self::default_config_path() {
            if config_path.exists() {
                match Self::load_from_file(&config_path) {
                    Ok(config) => return Ok(config),
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to load config from {:?}: {}",
                            config_path, e
                        );
                    }
                }
            }
        }

        // Try environment-specific locations
        if let Ok(config_path) = std::env::var("FHIRPATH_TUI_CONFIG") {
            let path = PathBuf::from(config_path);
            if path.exists() {
                match Self::load_from_file(&path) {
                    Ok(config) => return Ok(config),
                    Err(e) => {
                        eprintln!("Warning: Failed to load config from {:?}: {}", path, e);
                    }
                }
            }
        }

        // Fall back to default configuration
        Ok(Self::default())
    }

    /// Auto-save configuration if enabled
    pub fn auto_save(&mut self) -> Result<()> {
        if !self.features.auto_save_config {
            return Ok(());
        }

        // Get config path
        let config_path = Self::default_config_path()?;

        // Save configuration
        self.save_to_file(config_path)?;

        Ok(())
    }

    /// Update feature flag
    pub fn set_feature(&mut self, feature: &str, enabled: bool) -> Result<()> {
        match feature {
            "syntax_highlighting" => self.features.syntax_highlighting = enabled,
            "auto_completion" => self.features.auto_completion = enabled,
            "real_time_validation" => self.features.real_time_validation = enabled,
            "mouse_support" => self.features.mouse_support = enabled,
            "advanced_editing" => self.features.advanced_editing = enabled,
            "performance_monitoring" => self.features.performance_monitoring = enabled,
            "diagnostic_details" => self.features.diagnostic_details = enabled,
            "persistent_history" => self.features.persistent_history = enabled,
            "auto_save_config" => self.features.auto_save_config = enabled,
            _ => anyhow::bail!("Unknown feature flag: {}", feature),
        }

        // Auto-save if enabled
        self.auto_save()?;

        Ok(())
    }

    /// Get feature flag value
    pub fn get_feature(&self, feature: &str) -> Option<bool> {
        match feature {
            "syntax_highlighting" => Some(self.features.syntax_highlighting),
            "auto_completion" => Some(self.features.auto_completion),
            "real_time_validation" => Some(self.features.real_time_validation),
            "mouse_support" => Some(self.features.mouse_support),
            "advanced_editing" => Some(self.features.advanced_editing),
            "performance_monitoring" => Some(self.features.performance_monitoring),
            "diagnostic_details" => Some(self.features.diagnostic_details),
            "persistent_history" => Some(self.features.persistent_history),
            "auto_save_config" => Some(self.features.auto_save_config),
            _ => None,
        }
    }

    /// Set theme by name
    pub fn set_theme(&mut self, theme_name: &str) -> Result<()> {
        if let Some(theme) = TuiTheme::load_theme(theme_name) {
            self.theme = theme;
            self.auto_save()?;
            Ok(())
        } else {
            anyhow::bail!("Unknown theme: {}", theme_name)
        }
    }

    /// Build key bindings from configuration
    pub fn build_key_bindings(&self) -> Result<KeyBindings> {
        if self.key_bindings.is_empty() {
            Ok(KeyBindings::default())
        } else {
            KeyBindings::from_config(&self.key_bindings)
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        // Validate performance settings
        if self.performance.target_fps < 10 || self.performance.target_fps > 120 {
            issues.push("Target FPS should be between 10 and 120".to_string());
        }

        if self.performance.max_history_entries < 10 {
            issues.push("Maximum history entries should be at least 10".to_string());
        }

        if self.performance.validation_debounce_ms > 2000 {
            issues.push("Validation debounce delay should not exceed 2000ms".to_string());
        }

        // Validate UI preferences
        if self.ui_preferences.tooltip_delay_ms > 5000 {
            issues.push("Tooltip delay should not exceed 5000ms".to_string());
        }

        // Validate theme compatibility
        issues.extend(super::themes::utils::validate_theme_compatibility(
            &self.theme,
        ));

        issues
    }

    /// Reset to default configuration
    pub fn reset_to_default(&mut self) -> Result<()> {
        let default_config = Self::default();

        // Preserve metadata user field
        let user = self.metadata.user.clone();
        *self = default_config;
        self.metadata.user = user;
        self.metadata.last_modified = chrono::Utc::now().to_rfc3339();

        self.auto_save()?;
        Ok(())
    }

    /// Export configuration to different format (TOML primary, JSON fallback)
    pub fn export_config(&self, format: ConfigFormat) -> Result<String> {
        // Create serializable subset
        let serializable = SerializableConfig {
            metadata: self.metadata.clone(),
            features: self.features.clone(),
            performance: self.performance.clone(),
            ui_preferences: self.ui_preferences.clone(),
            theme_name: self.theme.metadata.name.clone(),
            key_bindings: self.key_bindings.clone(),
        };

        match format {
            ConfigFormat::Toml => {
                let content =
                    toml::to_string_pretty(&serializable).context("Failed to serialize to TOML")?;
                Ok(format!(
                    "# FHIRPath TUI Configuration\n\
                     # Exported at {}\n\
                     \n\
                     {}",
                    chrono::Utc::now().to_rfc3339(),
                    content
                ))
            }
            ConfigFormat::Json => {
                let content = serde_json::to_string_pretty(&serializable)
                    .context("Failed to serialize to JSON")?;
                Ok(content)
            }
        }
    }

    /// Get configuration summary
    pub fn summary(&self) -> Vec<String> {
        let mut summary = Vec::new();

        summary.push(format!("Configuration Version: {}", self.metadata.version));
        summary.push(format!("Theme: {}", self.theme.metadata.name));
        summary.push(format!("Layout Mode: {:?}", self.layout.layout_mode));
        summary.push(format!(
            "Features Enabled: {}",
            self.enabled_features_count()
        ));
        summary.push(format!(
            "Performance Target: {} FPS",
            self.performance.target_fps
        ));
        summary.push(format!("Last Modified: {}", self.metadata.last_modified));

        summary
    }

    /// Count enabled features
    fn enabled_features_count(&self) -> usize {
        let mut count = 0;

        if self.features.syntax_highlighting {
            count += 1;
        }
        if self.features.auto_completion {
            count += 1;
        }
        if self.features.real_time_validation {
            count += 1;
        }
        if self.features.mouse_support {
            count += 1;
        }
        if self.features.advanced_editing {
            count += 1;
        }
        if self.features.performance_monitoring {
            count += 1;
        }
        if self.features.diagnostic_details {
            count += 1;
        }
        if self.features.persistent_history {
            count += 1;
        }
        if self.features.auto_save_config {
            count += 1;
        }

        count
    }
}

/// Configuration export formats
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    Toml,
    Json,
}

/// Configuration builder for programmatic setup
pub struct TuiConfigBuilder {
    config: TuiConfig,
}

impl TuiConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: TuiConfig::default(),
        }
    }

    /// Set theme
    pub fn with_theme(mut self, theme: TuiTheme) -> Self {
        self.config.theme = theme;
        self
    }

    /// Set theme by name
    pub fn with_theme_name(mut self, theme_name: &str) -> Result<Self> {
        if let Some(theme) = TuiTheme::load_theme(theme_name) {
            self.config.theme = theme;
            Ok(self)
        } else {
            anyhow::bail!("Unknown theme: {}", theme_name)
        }
    }

    /// Set layout configuration
    pub fn with_layout(mut self, layout: LayoutConfig) -> Self {
        self.config.layout = layout;
        self
    }

    /// Enable feature
    pub fn with_feature(mut self, feature: &str, enabled: bool) -> Result<Self> {
        self.config.set_feature(feature, enabled)?;
        Ok(self)
    }

    /// Set performance config
    pub fn with_performance(mut self, performance: PerformanceConfig) -> Self {
        self.config.performance = performance;
        self
    }

    /// Set UI preferences
    pub fn with_ui_preferences(mut self, preferences: UiPreferences) -> Self {
        self.config.ui_preferences = preferences;
        self
    }

    /// Build the configuration
    pub fn build(self) -> TuiConfig {
        self.config
    }
}

impl Default for TuiConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    #[ignore] // Disabled due to serialization limitations
    fn test_config_serialization() {
        // TODO: Implement when partial serialization is available
        let _config = TuiConfig::default();
        // Serialization disabled for complex theme types
    }

    #[test]
    fn test_config_file_operations() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.toml");

        let mut config = TuiConfig::default();
        config.save_to_file(&config_path).unwrap();

        let loaded_config = TuiConfig::load_from_file(&config_path).unwrap();
        assert_eq!(config.metadata.version, loaded_config.metadata.version);
    }

    #[test]
    fn test_feature_management() {
        let mut config = TuiConfig::default();

        assert!(config.get_feature("syntax_highlighting").unwrap());
        config.set_feature("syntax_highlighting", false).unwrap();
        assert!(!config.get_feature("syntax_highlighting").unwrap());

        assert!(config.get_feature("nonexistent").is_none());
    }

    #[test]
    fn test_config_builder() {
        let config = TuiConfigBuilder::new()
            .with_theme_name("dark")
            .unwrap()
            .with_feature("mouse_support", false)
            .unwrap()
            .build();

        assert_eq!(config.theme.metadata.name, "Dark");
        assert!(!config.features.mouse_support);
    }

    #[test]
    fn test_config_validation() {
        let mut config = TuiConfig::default();
        config.performance.target_fps = 5; // Invalid

        let issues = config.validate();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|issue| issue.contains("Target FPS")));
    }
}
