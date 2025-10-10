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

//! CLI configuration file support

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::output::OutputFormat;

/// CLI configuration loaded from ~/.fhirpathrc or .fhirpathrc
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliConfig {
    /// Default FHIR version to use
    #[serde(default)]
    pub fhir_version: Option<String>,

    /// Default output format
    #[serde(default)]
    pub output_format: Option<OutputFormat>,

    /// Disable colored output by default
    #[serde(default)]
    pub no_color: bool,

    /// Enable quiet mode by default
    #[serde(default)]
    pub quiet: bool,

    /// Enable verbose mode by default
    #[serde(default)]
    pub verbose: bool,

    /// Default packages to load
    #[serde(default)]
    pub packages: Vec<String>,

    /// Default variables in format "name=value"
    #[serde(default)]
    pub variables: Vec<String>,

    /// Favorite expressions with aliases
    #[serde(default)]
    pub favorites: Vec<FavoriteExpression>,

    /// Expression history settings
    #[serde(default)]
    pub history: HistoryConfig,

    /// Custom output templates
    #[serde(default)]
    pub templates: Vec<OutputTemplate>,
}

/// A favorite FHIRPath expression with an alias
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteExpression {
    /// Alias name to invoke this expression
    pub alias: String,

    /// The FHIRPath expression
    pub expression: String,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

/// History configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// Enable expression history
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum number of history entries
    #[serde(default = "default_history_size")]
    pub max_size: usize,

    /// History file path (relative to home directory if not absolute)
    #[serde(default = "default_history_file")]
    pub file: String,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size: 1000,
            file: ".fhirpath_history".to_string(),
        }
    }
}

/// Custom output template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputTemplate {
    /// Template name
    pub name: String,

    /// Template format string (supports placeholders like {expression}, {result}, {type})
    pub format: String,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_history_size() -> usize {
    1000
}

fn default_history_file() -> String {
    ".fhirpath_history".to_string()
}

impl CliConfig {
    /// Load configuration from standard locations
    ///
    /// Search order:
    /// 1. ./.fhirpathrc (current directory)
    /// 2. ~/.fhirpathrc (home directory)
    /// 3. ~/.config/fhirpath/config.toml
    pub fn load() -> anyhow::Result<Self> {
        // Try current directory first
        if let Ok(config) = Self::load_from_file(".fhirpathrc") {
            return Ok(config);
        }

        // Try home directory
        if let Some(home) = dirs::home_dir() {
            let home_rc = home.join(".fhirpathrc");
            if home_rc.exists()
                && let Ok(config) = Self::load_from_file(&home_rc)
            {
                return Ok(config);
            }

            // Try XDG config directory
            let config_dir = home.join(".config").join("fhirpath").join("config.toml");
            if config_dir.exists()
                && let Ok(config) = Self::load_from_file(&config_dir)
            {
                return Ok(config);
            }
        }

        // No config found, return default
        Ok(Self::default())
    }

    /// Load configuration from a specific file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get default config file path (user's home directory)
    pub fn default_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".fhirpathrc"))
    }

    /// Create a sample configuration file with helpful comments
    pub fn sample_config() -> String {
        r#"# FHIRPath CLI Configuration
# This file can be placed at:
#   - ./.fhirpathrc (current directory)
#   - ~/.fhirpathrc (home directory)
#   - ~/.config/fhirpath/config.toml

# Default FHIR version (r4, r4b, r5)
fhir_version = "r4"

# Default output format (pretty, json, raw)
output_format = "pretty"

# Disable colored output
no_color = false

# Enable quiet mode (suppress informational messages)
quiet = false

# Enable verbose mode
verbose = false

# Default packages to load (format: package@version)
packages = [
    # "hl7.fhir.us.core@5.0.1",
]

# Default variables (format: name=value)
variables = [
    # "env=production",
]

# Favorite expressions with aliases
[[favorites]]
alias = "patient-name"
expression = "Patient.name.given.first() + ' ' + Patient.name.family"
description = "Get patient's full name"

[[favorites]]
alias = "obs-value"
expression = "Observation.value.ofType(Quantity).value"
description = "Extract observation quantity value"

# Expression history configuration
[history]
enabled = true
max_size = 1000
file = ".fhirpath_history"

# Custom output templates
[[templates]]
name = "csv"
format = "{result}"
description = "CSV output format"

[[templates]]
name = "markdown"
format = "**Expression**: `{expression}`\n\n**Result**: {result}"
description = "Markdown formatted output"
"#
        .to_string()
    }

    /// Get favorite expression by alias
    pub fn get_favorite(&self, alias: &str) -> Option<&FavoriteExpression> {
        self.favorites.iter().find(|f| f.alias == alias)
    }

    /// Get output template by name
    pub fn get_template(&self, name: &str) -> Option<&OutputTemplate> {
        self.templates.iter().find(|t| t.name == name)
    }

    /// Merge with CLI arguments (CLI args take precedence)
    pub fn merge_with_cli(&self, cli: &super::Cli) -> super::Cli {
        let mut merged = cli.clone();

        // Apply config defaults only if CLI didn't specify them
        if merged.fhir_version == "r4" && self.fhir_version.is_some() {
            merged.fhir_version = self.fhir_version.clone().unwrap();
        }

        if let Some(format) = &self.output_format {
            // Only override if using default
            if merged.output_format == OutputFormat::Pretty {
                merged.output_format = format.clone();
            }
        }

        if self.no_color && !merged.no_color {
            merged.no_color = true;
        }

        if self.quiet && !merged.quiet {
            merged.quiet = true;
        }

        if self.verbose && !merged.verbose {
            merged.verbose = true;
        }

        // Merge packages (config packages + CLI packages)
        let mut all_packages = self.packages.clone();
        all_packages.extend(merged.packages.clone());
        merged.packages = all_packages;

        merged
    }
}
