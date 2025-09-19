// FHIR XML -> JSON converter that follows FHIR specification rules
// Usage:
//   cargo run --package fhirpath-dev-tools --bin convert-fhir-xml -- <input.xml> <output.json>
//   cargo run --package fhirpath-dev-tools --bin convert-fhir-xml -- <source_dir> <target_dir>

use roxmltree::Document;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn unescape_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

// FHIR elements that are always arrays even if single occurrence
const FHIR_ARRAY_ELEMENTS: &[&str] = &[
    "identifier", "name", "telecom", "address", "contact", "communication",
    "extension", "modifierExtension", "given", "prefix", "suffix", "line",
    "coding", "contained", "link", "photo", "generalPractitioner"
];

fn is_array_element(name: &str) -> bool {
    FHIR_ARRAY_ELEMENTS.contains(&name)
}

fn append_to_json_object(obj: &mut Map<String, Value>, key: &str, value: Value) {
    match obj.get_mut(key) {
        Some(existing) => {
            if existing.is_array() {
                existing.as_array_mut().unwrap().push(value);
            } else {
                let old = existing.clone();
                *existing = Value::Array(vec![old, value]);
            }
        }
        None => {
            if is_array_element(key) {
                obj.insert(key.to_string(), Value::Array(vec![value]));
            } else {
                obj.insert(key.to_string(), value);
            }
        }
    }
}

fn from_xml(input: &str) -> Result<Value, String> {
    let doc = Document::parse(input).map_err(|e| format!("XML parse error: {e}"))?;
    let root = doc.root_element();

    let mut root_obj = Map::new();
    root_obj.insert("resourceType".to_string(), Value::String(root.tag_name().name().to_string()));

    // Convert all children of the root element
    convert_element_children(&root, &mut root_obj)?;

    Ok(Value::Object(root_obj))
}

fn convert_element_children(element: &roxmltree::Node, obj: &mut Map<String, Value>) -> Result<(), String> {
    for child in element.children() {
        if child.is_element() {
            let child_name = child.tag_name().name();
            let child_value = convert_element(&child)?;
            append_to_json_object(obj, child_name, child_value);
        }
    }
    Ok(())
}

fn convert_element(element: &roxmltree::Node) -> Result<Value, String> {
    let element_name = element.tag_name().name();

    // Handle special cases
    if element_name == "extension" {
        return convert_extension(element);
    }

    if element_name == "div" {
        return convert_div_element(element);
    }

    // Get value attribute
    let value_attr = element.attribute("value");

    // Check if element has child elements
    let child_elements: Vec<_> = element.children().filter(|n| n.is_element()).collect();

    if child_elements.is_empty() {
        // Leaf element - return the value attribute or empty object
        if let Some(value) = value_attr {
            Ok(Value::String(value.to_string()))
        } else {
            Ok(Value::Object(Map::new()))
        }
    } else {
        // Complex element with children
        let mut obj = Map::new();

        // Add value attribute if present
        if let Some(value) = value_attr {
            obj.insert("value".to_string(), Value::String(value.to_string()));
        }

        // Add all child elements
        convert_element_children(element, &mut obj)?;

        Ok(Value::Object(obj))
    }
}

fn convert_extension(element: &roxmltree::Node) -> Result<Value, String> {
    let mut obj = Map::new();

    // Add url attribute
    if let Some(url) = element.attribute("url") {
        obj.insert("url".to_string(), Value::String(url.to_string()));
    }

    // Add all child elements (like valueDateTime, valueString, etc.)
    convert_element_children(element, &mut obj)?;

    Ok(Value::Object(obj))
}

fn convert_div_element(element: &roxmltree::Node) -> Result<Value, String> {
    // For div elements, we need to get the full HTML content
    // This is simplified - for full HTML reconstruction we'd need to rebuild the entire subtree
    let text_content = get_element_text_content(element);
    Ok(Value::String(text_content))
}

fn get_element_text_content(element: &roxmltree::Node) -> String {
    let mut result = String::new();

    for child in element.children() {
        if child.is_text() {
            result.push_str(child.text().unwrap_or(""));
        } else if child.is_element() {
            // For elements inside div, we should reconstruct the HTML
            // This is a simplified version
            result.push_str(&format!("<{}>", child.tag_name().name()));
            result.push_str(&get_element_text_content(&child));
            result.push_str(&format!("</{}>", child.tag_name().name()));
        }
    }

    result
}


fn convert_file(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let xml = fs::read_to_string(input_path)?;
    let json_value = from_xml(&xml).map_err(|e| format!("Conversion failed: {e}"))?;
    fs::write(output_path, serde_json::to_string_pretty(&json_value)?)?;
    Ok(())
}

fn convert_directory(source_dir: &Path, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create target directory if it doesn't exist
    fs::create_dir_all(target_dir)?;

    println!("🔄 Converting FHIR XML files from {} to {}", source_dir.display(), target_dir.display());

    let mut converted_count = 0;
    let mut failed_count = 0;

    // Read all entries in the source directory
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "xml") {
            let filename = path.file_stem().unwrap().to_str().unwrap();
            let output_path = target_dir.join(format!("{}.json", filename));

            match convert_file(&path, &output_path) {
                Ok(()) => {
                    println!("✅ Converted {} -> {}", path.display(), output_path.display());
                    converted_count += 1;
                }
                Err(e) => {
                    println!("❌ Failed to convert {}: {}", path.display(), e);
                    failed_count += 1;
                }
            }
        }
    }

    println!("🎉 Conversion completed! {} files converted, {} failed", converted_count, failed_count);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage:");
        eprintln!("  {} <input.xml> <output.json>      # Convert single file", args[0]);
        eprintln!("  {} <source_dir> <target_dir>      # Convert all XML files in directory", args[0]);
        std::process::exit(1);
    }

    let source_path = PathBuf::from(&args[1]);
    let target_path = PathBuf::from(&args[2]);

    if source_path.is_dir() {
        // Directory mode
        convert_directory(&source_path, &target_path)?;
    } else {
        // Single file mode
        convert_file(&source_path, &target_path)?;
        println!("✅ Converted {} -> {}", source_path.display(), target_path.display());
    }

    Ok(())
}
