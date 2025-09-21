#![allow(clippy::uninlined_format_args)]
#![allow(clippy::single_char_add_str)]

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use octofhir_fhirpath::core::error_code::{ErrorCategory, ErrorCode};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

/// Generate MDX documentation for FHIRPath functions and error codes
#[derive(Parser, Debug)]
#[command(name = "generate-docs")]
#[command(about = "Generate MDX docs for functions and error codes", long_about = None)]
struct Args {
    /// Output documentation root directory (Starlight docs content dir)
    #[arg(short = 'o', long = "out", default_value = "docs/src/content/docs")]
    out_dir: PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    // Ensure base structure exists
    let functions_dir = args.out_dir.join("functions");
    let errors_dir = args.out_dir.join("errors");
    fs::create_dir_all(&functions_dir).context("create functions dir")?;
    fs::create_dir_all(&errors_dir).context("create errors dir")?;

    generate_errors_docs(&errors_dir)?;
    generate_section_indexes(&args.out_dir)?;
    println!("📚 Error docs generation complete.");

    println!("✅ Documentation generated at {}", args.out_dir.display());
    Ok(())
}

/*
fn generate_functions_docs(dir: &Path) -> Result<()> {
    let registry = FunctionRegistry::default();
    let mut items = registry.list_functions();
    // Sort by name
    items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Group by category for index later (use HashMap to avoid Ord bound)
    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();

    for meta in items {
        let file_name = format!("{}.mdx", meta.name);
        let path = dir.join(&file_name);
        let category = format_function_category(&meta.category);

        let mut mdx = String::new();
        mdx.push_str(&format!(
            "---\ntitle: {}\nsidebar:\n  label: {}\n---\n\n",
            meta.name, meta.name
        ));
        mdx.push_str(&format!("# {}()\n\n", meta.name));
        mdx.push_str(&format!("**Category**: {}\n\n", category));
        if !meta.description.is_empty() {
            mdx.push_str(&format!("{}\n\n", meta.description));
        }

        // Signature
        mdx.push_str("## Signature\n\n");
        let return_type_name = meta.signature.returns.display_name();
        if !return_type_name.is_empty() {
            mdx.push_str(&format!("- Return type: `{}`\n", return_type_name));
        }
        if !meta.signature.parameters.is_empty() {
            mdx.push_str("- Parameters:\n");
            for p in &meta.signature.parameters {
                let t = format!(" `{:?}`", p.ty);
                let opt = if p.variadic { " (variadic)" } else { "" };
                mdx.push_str(&format!("  - `{}`{}{}\n", p.name, t, opt));
            }
        }
        mdx.push_str("\n");

        // Examples
        if !meta.examples.is_empty() {
            mdx.push_str("## Examples\n\n");
            for ex in &meta.examples {
                mdx.push_str("```fhirpath\n");
                mdx.push_str(ex);
                mdx.push_str("\n```\n\n");
            }
        }

        fs::write(&path, mdx).with_context(|| format!("write {}", path.display()))?;

        grouped
            .entry(category.to_string())
            .or_default()
            .push(file_name);
    }

    // Build a simple index with links
    let mut index = String::new();
    index.push_str("---\ntitle: Function Library\n---\n\n");
    index.push_str("# Function Library\n\n");
    let desired_order = [
        "Collection",
        "String",
        "Math",
        "Type",
        "Conversion",
        "Date & Time",
        "FHIR",
        "Terminology",
        "Logic",
        "Utility",
    ];
    let mut printed: BTreeSet<String> = BTreeSet::new();
    for cat in desired_order {
        if let Some(files) = grouped.get(cat) {
            index.push_str(&format!("## {}\n\n", cat));
            let mut files = files.clone();
            files.sort();
            for f in files {
                let name = f.trim_end_matches(".mdx");
                index.push_str(&format!("- [{}](/functions/{})\n", name, name));
            }
            index.push_str("\n");
            printed.insert(cat.to_string());
        }
    }
    // Print any remaining categories alphabetically
    let mut remaining: Vec<_> = grouped
        .iter()
        .filter(|(k, _)| !printed.contains(*k))
        .collect();
    remaining.sort_by(|a, b| a.0.cmp(b.0));
    for (cat, files) in remaining {
        index.push_str(&format!("## {}\n\n", cat));
        let mut files = files.clone();
        files.sort();
        for f in files {
            let name = f.trim_end_matches(".mdx");
            index.push_str(&format!("- [{}](/functions/{})\n", name, name));
        }
        index.push_str("\n");
    }
    fs::write(dir.join("index.mdx"), index).context("write functions index")?;

    Ok(())
}

*/

fn generate_errors_docs(dir: &Path) -> Result<()> {
    // Iterate a reasonable range and filter defined codes
    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();

    for code in 1u16..=250u16 {
        let ec = ErrorCode::new(code);
        let info = ec.info();
        if info.code != code {
            // unknown
            continue;
        }
        let file = format!("FP{:04}.mdx", code);
        let path = dir.join(&file);

        let mut mdx = String::new();
        mdx.push_str(&format!(
            "---\ntitle: {}\nsidebar:\n  label: FP{:04}\n---\n\n",
            ec.code_str(),
            code
        ));
        mdx.push_str(&format!("# {} - {}\n\n", ec.code_str(), info.title));
        mdx.push_str(&format!(
            "**Category**: {}\n\n",
            format_error_category(&ec.category())
        ));
        if !info.description.is_empty() {
            mdx.push_str("## Description\n\n");
            mdx.push_str(info.description);
            mdx.push_str("\n\n");
        }
        if !info.help.is_empty() {
            mdx.push_str("## Help\n\n");
            mdx.push_str(info.help);
            mdx.push_str("\n");
        }

        fs::write(&path, mdx).with_context(|| format!("write {}", path.display()))?;

        let cat = format_error_category(&ec.category()).to_string();
        grouped.entry(cat).or_default().push(file);
    }

    // Build index page
    let mut index = String::new();
    index.push_str("---\ntitle: Error Codes\n---\n\n");
    index.push_str("# Error Codes\n\n");
    let error_order = ["Parser", "Evaluation", "Model Provider", "Analysis"];
    let mut printed: BTreeSet<String> = BTreeSet::new();
    for cat in error_order {
        if let Some(files) = grouped.get(cat) {
            index.push_str(&format!("## {}\n\n", cat));
            let mut files = files.clone();
            files.sort();
            for f in files {
                let name = f.trim_end_matches(".mdx");
                index.push_str(&format!("- [{}](/errors/{})\n", name, name));
            }
            index.push_str("\n");
            printed.insert(cat.to_string());
        }
    }
    // Remaining categories (if any)
    let mut remaining: Vec<_> = grouped
        .iter()
        .filter(|(k, _)| !printed.contains(*k))
        .collect();
    remaining.sort_by(|a, b| a.0.cmp(b.0));
    for (cat, files) in remaining {
        index.push_str(&format!("## {}\n\n", cat));
        let mut files = files.clone();
        files.sort();
        for f in files {
            let name = f.trim_end_matches(".mdx");
            index.push_str(&format!("- [{}](/errors/{})\n", name, name));
        }
        index.push_str("\n");
    }
    fs::write(dir.join("index.mdx"), index).context("write errors index")?;

    Ok(())
}

fn generate_section_indexes(root: &Path) -> Result<()> {
    // Root index page
    let mut mdx = String::new();
    mdx.push_str("---\ntitle: Introduction\n---\n\n");
    mdx.push_str("# FHIRPath-rs Documentation\n\n");
    mdx.push_str("- [Error Codes](/errors/)\n");
    fs::create_dir_all(root).context("create docs root")?;
    fs::write(root.join("index.mdx"), mdx).context("write root index")?;
    Ok(())
}

#[allow(dead_code)]
fn format_error_category(cat: &ErrorCategory) -> &'static str {
    match cat {
        ErrorCategory::Parser => "Parser",
        ErrorCategory::Evaluation => "Evaluation",
        ErrorCategory::ModelProvider => "Model Provider",
        ErrorCategory::Analysis => "Analysis",
    }
}
