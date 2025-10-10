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

//! Configuration management handler

use crate::cli::ConfigCommands;
use crate::cli::config::{CliConfig, FavoriteExpression};
use comfy_table::{Table, presets::UTF8_FULL};
use std::path::PathBuf;

/// Handle config subcommands
pub fn handle_config(command: &ConfigCommands) -> anyhow::Result<()> {
    match command {
        ConfigCommands::Show => handle_show(),
        ConfigCommands::Path => handle_path(),
        ConfigCommands::Init { force, path } => handle_init(*force, path.as_deref()),
        ConfigCommands::Edit => handle_edit(),
        ConfigCommands::AddFavorite {
            alias,
            expression,
            description,
        } => handle_add_favorite(alias, expression, description.as_deref()),
        ConfigCommands::ListFavorites => handle_list_favorites(),
        ConfigCommands::RemoveFavorite { alias } => handle_remove_favorite(alias),
    }
}

fn handle_show() -> anyhow::Result<()> {
    let config = CliConfig::load()?;
    let toml_str = toml::to_string_pretty(&config)?;

    println!("📄 Current Configuration:\n");
    println!("{}", toml_str);

    if let Some(path) = find_config_file() {
        println!("\n💾 Loaded from: {}", path.display());
    } else {
        println!("\n💡 No configuration file found (using defaults)");
        println!("   Run `octofhir-fhirpath config init` to create one");
    }

    Ok(())
}

fn handle_path() -> anyhow::Result<()> {
    if let Some(path) = find_config_file() {
        println!("📍 Config file: {}", path.display());
    } else {
        println!("❌ No configuration file found");
        if let Some(default_path) = CliConfig::default_path() {
            println!("💡 Default location: {}", default_path.display());
            println!("   Run `octofhir-fhirpath config init` to create one");
        }
    }
    Ok(())
}

fn handle_init(force: bool, path: Option<&str>) -> anyhow::Result<()> {
    let target_path = if let Some(p) = path {
        PathBuf::from(p)
    } else {
        CliConfig::default_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
    };

    if target_path.exists() && !force {
        return Err(anyhow::anyhow!(
            "Configuration file already exists: {}\nUse --force to overwrite",
            target_path.display()
        ));
    }

    // Create parent directory if needed
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write sample config
    std::fs::write(&target_path, CliConfig::sample_config())?;

    println!("✅ Configuration file created: {}", target_path.display());
    println!("📝 Edit it to customize your FHIRPath CLI experience");

    Ok(())
}

fn handle_edit() -> anyhow::Result<()> {
    let config_path = find_config_file().ok_or_else(|| {
        anyhow::anyhow!("No configuration file found.\nRun `octofhir-fhirpath config init` first")
    })?;

    // Get editor from environment or use sensible defaults
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

    println!("📝 Opening {} in {}...", config_path.display(), editor);

    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status()?;

    if status.success() {
        println!("✅ Configuration saved");
    } else {
        return Err(anyhow::anyhow!("Editor exited with error"));
    }

    Ok(())
}

fn handle_add_favorite(
    alias: &str,
    expression: &str,
    description: Option<&str>,
) -> anyhow::Result<()> {
    let config_path = get_or_create_config()?;
    let mut config = CliConfig::load()?;

    // Check if alias already exists
    if config.get_favorite(alias).is_some() {
        return Err(anyhow::anyhow!(
            "Favorite with alias '{}' already exists.\nUse `config remove-favorite {}` first",
            alias,
            alias
        ));
    }

    // Add new favorite
    config.favorites.push(FavoriteExpression {
        alias: alias.to_string(),
        expression: expression.to_string(),
        description: description.map(|s| s.to_string()),
    });

    // Save config
    config.save_to_file(&config_path)?;

    println!("✅ Added favorite expression:");
    println!("   Alias: {}", alias);
    println!("   Expression: {}", expression);
    if let Some(desc) = description {
        println!("   Description: {}", desc);
    }

    Ok(())
}

fn handle_list_favorites() -> anyhow::Result<()> {
    let config = CliConfig::load()?;

    if config.favorites.is_empty() {
        println!("📋 No favorite expressions defined");
        println!("💡 Add one with: octofhir-fhirpath config add-favorite <alias> <expression>");
        return Ok(());
    }

    println!("⭐ Favorite Expressions:\n");

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Alias", "Expression", "Description"]);

    for fav in &config.favorites {
        table.add_row(vec![
            &fav.alias,
            &fav.expression,
            fav.description.as_deref().unwrap_or(""),
        ]);
    }

    println!("{}", table);
    println!("\n💡 Use favorites in expressions with: @{{alias}}");

    Ok(())
}

fn handle_remove_favorite(alias: &str) -> anyhow::Result<()> {
    let config_path = get_or_create_config()?;
    let mut config = CliConfig::load()?;

    let original_count = config.favorites.len();
    config.favorites.retain(|f| f.alias != alias);

    if config.favorites.len() == original_count {
        return Err(anyhow::anyhow!("Favorite '{}' not found", alias));
    }

    config.save_to_file(&config_path)?;

    println!("✅ Removed favorite: {}", alias);

    Ok(())
}

/// Find existing config file
fn find_config_file() -> Option<PathBuf> {
    // Check current directory
    let cwd_config = PathBuf::from(".fhirpathrc");
    if cwd_config.exists() {
        return Some(cwd_config);
    }

    // Check home directory
    if let Some(home) = dirs::home_dir() {
        let home_rc = home.join(".fhirpathrc");
        if home_rc.exists() {
            return Some(home_rc);
        }

        // Check XDG config
        let xdg_config = home.join(".config").join("fhirpath").join("config.toml");
        if xdg_config.exists() {
            return Some(xdg_config);
        }
    }

    None
}

/// Get config file path or create default one
fn get_or_create_config() -> anyhow::Result<PathBuf> {
    if let Some(path) = find_config_file() {
        Ok(path)
    } else {
        let default_path = CliConfig::default_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        // Create with default config
        let config = CliConfig::default();
        config.save_to_file(&default_path)?;

        Ok(default_path)
    }
}
