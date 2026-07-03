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
            mdx.push('\n');
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
            index.push('\n');
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
        index.push('\n');
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
