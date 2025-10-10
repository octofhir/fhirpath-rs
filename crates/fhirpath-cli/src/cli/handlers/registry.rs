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

//! Handler for registry commands

use crate::cli::context::CliContext;
use crate::cli::{RegistryCommands, RegistryShowTarget, RegistryTarget};

/// Handle registry command
pub async fn handle_registry(command: &RegistryCommands, context: &CliContext) {
    match command {
        RegistryCommands::List {
            target,
            category,
            search,
            output_format,
            no_color,
            quiet,
            verbose,
        } => {
            let ctx =
                context.with_subcommand_options(output_format.clone(), *no_color, *quiet, *verbose);

            match target {
                RegistryTarget::Functions => {
                    handle_registry_list_functions(category, search, &ctx).await;
                }
                RegistryTarget::Operators => {
                    handle_registry_list_operators(category, search, &ctx).await;
                }
            }
        }
        RegistryCommands::Show {
            name,
            target,
            output_format,
            no_color,
            quiet,
            verbose,
        } => {
            let ctx =
                context.with_subcommand_options(output_format.clone(), *no_color, *quiet, *verbose);

            handle_registry_show(name, target, &ctx).await;
        }
    }
}

/// List functions from the registry
pub async fn handle_registry_list_functions(
    category: &Option<String>,
    search: &Option<String>,
    context: &CliContext,
) {
    use octofhir_fhirpath::evaluator::function_registry::{
        FunctionCategory, create_function_registry,
    };

    let registry = create_function_registry();
    let mut functions: Vec<_> = registry.all_metadata().iter().collect();

    // Filter by category if provided
    if let Some(cat_filter) = category {
        let category_enum = match cat_filter.to_lowercase().as_str() {
            "existence" => Some(FunctionCategory::Existence),
            "filtering" | "projection" => Some(FunctionCategory::FilteringProjection),
            "subsetting" => Some(FunctionCategory::Subsetting),
            "combining" => Some(FunctionCategory::Combining),
            "conversion" => Some(FunctionCategory::Conversion),
            "logic" => Some(FunctionCategory::Logic),
            "string" => Some(FunctionCategory::StringManipulation),
            "math" => Some(FunctionCategory::Math),
            "tree" | "navigation" => Some(FunctionCategory::TreeNavigation),
            "utility" => Some(FunctionCategory::Utility),
            "terminology" => Some(FunctionCategory::Terminology),
            "types" => Some(FunctionCategory::Types),
            "aggregate" => Some(FunctionCategory::Aggregate),
            "cda" => Some(FunctionCategory::CDA),
            _ => {
                if !context.quiet {
                    eprintln!("Unknown category '{cat_filter}'.");
                }
                return;
            }
        };

        if let Some(cat) = category_enum {
            functions.retain(|(_, metadata)| metadata.category == cat);
        }
    }

    // Filter by search pattern
    if let Some(search_pattern) = search {
        let pattern = search_pattern.to_lowercase();
        functions.retain(|(name, metadata)| {
            name.to_lowercase().contains(&pattern)
                || metadata.description.to_lowercase().contains(&pattern)
                || metadata.name.to_lowercase().contains(&pattern)
        });
    }

    // Sort by name
    functions.sort_by(|(a, _), (b, _)| a.cmp(b));

    if functions.is_empty() {
        if !context.quiet {
            println!("No functions found matching the criteria");
        }
        return;
    }

    // Output functions
    if !context.quiet {
        println!("📋 FHIRPath Functions ({} found)", functions.len());
        println!("{}", "=".repeat(50));
    }

    for (name, metadata) in functions {
        let category_str = format!("{:?}", metadata.category);
        let param_count = metadata.signature.parameters.len();
        let param_info = if param_count == 0 {
            "no params".to_string()
        } else if metadata.signature.max_params.is_none() {
            format!("{}+ params", metadata.signature.min_params)
        } else {
            format!(
                "{}-{} params",
                metadata.signature.min_params,
                metadata.signature.max_params.unwrap_or(0)
            )
        };

        println!(
            "🔧 {:<20} | {:<15} | {:<15} | {}",
            name,
            category_str,
            param_info,
            metadata.description.chars().take(40).collect::<String>()
        );
    }

    if !context.quiet {
        println!("\n💡 Use 'registry show <function_name>' for detailed information");
    }
}

/// List operators from the registry
pub async fn handle_registry_list_operators(
    _category: &Option<String>,
    search: &Option<String>,
    context: &CliContext,
) {
    use octofhir_fhirpath::evaluator::operator_registry::create_standard_operator_registry;

    let registry = create_standard_operator_registry();
    let mut operators: Vec<_> = registry.all_metadata().iter().collect();

    // Filter by search pattern
    if let Some(search_pattern) = search {
        let pattern = search_pattern.to_lowercase();
        operators.retain(|(name, metadata)| {
            name.to_lowercase().contains(&pattern)
                || metadata.description.to_lowercase().contains(&pattern)
                || metadata.name.to_lowercase().contains(&pattern)
        });
    }

    // Sort by name
    operators.sort_by(|(a, _), (b, _)| a.cmp(b));

    if operators.is_empty() {
        if !context.quiet {
            println!("No operators found matching the criteria");
        }
        return;
    }

    // Output operators
    if !context.quiet {
        println!("🔧 FHIRPath Operators ({} found)", operators.len());
        println!("{}", "=".repeat(50));
    }

    for (name, metadata) in operators {
        let precedence = metadata.precedence;
        let assoc = format!("{:?}", metadata.associativity);

        println!(
            "⚙️  {:<15} | P:{:<2} | {:<5} | {}",
            name,
            precedence,
            assoc,
            metadata.description.chars().take(50).collect::<String>()
        );
    }

    if !context.quiet {
        println!("\n💡 Use 'registry show <operator_name>' for detailed information");
    }
}

/// Show detailed information about a function or operator
pub async fn handle_registry_show(name: &str, target: &RegistryShowTarget, context: &CliContext) {
    match target {
        RegistryShowTarget::Auto => {
            if !try_show_function(name, context).await {
                try_show_operator(name, context).await;
            }
        }
        RegistryShowTarget::Function => {
            try_show_function(name, context).await;
        }
        RegistryShowTarget::Operator => {
            try_show_operator(name, context).await;
        }
    }
}

async fn try_show_function(name: &str, _context: &CliContext) -> bool {
    use octofhir_fhirpath::evaluator::function_registry::create_function_registry;

    let registry = create_function_registry();

    if let Some(metadata) = registry.get_metadata(name) {
        println!("🔧 Function: {}", metadata.name);
        println!("{}", "=".repeat(60));
        println!("📝 Description: {}", metadata.description);
        println!("📂 Category: {:?}", metadata.category);
        println!("🔢 Input Type: {}", metadata.signature.input_type);
        println!("🎯 Return Type: {}", metadata.signature.return_type);
        println!("⚡ Deterministic: {}", metadata.deterministic);
        println!("📋 Empty Propagation: {:?}", metadata.empty_propagation);

        if !metadata.signature.parameters.is_empty() {
            println!("\n📥 Parameters:");
            for (i, param) in metadata.signature.parameters.iter().enumerate() {
                let optional = if param.optional { " (optional)" } else { "" };
                let expr = if param.is_expression {
                    " [expression]"
                } else {
                    ""
                };
                println!(
                    "  {}: {} - {}{}{}",
                    i + 1,
                    param.name,
                    param.parameter_type.join(" | "),
                    optional,
                    expr
                );
                if !param.description.is_empty() {
                    println!("     {}", param.description);
                }
            }
        }

        println!(
            "\n🎛️  Signature: {}({})",
            metadata.name,
            metadata
                .signature
                .parameters
                .iter()
                .map(|p| {
                    let name = &p.name;
                    let types = p.parameter_type.join("|");
                    if p.optional {
                        format!("[{name}: {types}]")
                    } else {
                        format!("{name}: {types}")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        );

        true
    } else {
        false
    }
}

async fn try_show_operator(name: &str, context: &CliContext) -> bool {
    use octofhir_fhirpath::evaluator::operator_registry::create_standard_operator_registry;

    let registry = create_standard_operator_registry();

    if let Some(metadata) = registry.get_metadata(name) {
        println!("⚙️  Operator: {}", metadata.name);
        println!("{}", "=".repeat(60));
        println!("📝 Description: {}", metadata.description);
        println!(
            "🎯 Precedence: {} (higher = evaluated first)",
            metadata.precedence
        );
        println!("↔️  Associativity: {:?}", metadata.associativity);
        println!("⚡ Deterministic: {}", metadata.deterministic);
        println!("📋 Empty Propagation: {:?}", metadata.empty_propagation);

        println!("\n🎛️  Primary Signature:");
        println!(
            "  Input Types: {}",
            metadata
                .signature
                .signature
                .parameters
                .iter()
                .map(|t| format!("{t:?}"))
                .collect::<Vec<_>>()
                .join(" × ")
        );
        println!(
            "  Return Type: {:?}",
            metadata.signature.signature.return_type
        );

        if !metadata.signature.overloads.is_empty() {
            println!("\n🔄 Overloaded Signatures:");
            for (i, overload) in metadata.signature.overloads.iter().enumerate() {
                println!(
                    "  {}: {} → {:?}",
                    i + 1,
                    overload
                        .parameters
                        .iter()
                        .map(|t| format!("{t:?}"))
                        .collect::<Vec<_>>()
                        .join(" × "),
                    overload.return_type
                );
            }
        }

        true
    } else {
        if !context.quiet {
            eprintln!("Operator '{name}' not found");
        }
        false
    }
}
