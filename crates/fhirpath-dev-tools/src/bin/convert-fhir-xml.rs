// Simple FHIR XML -> JSON converter for test inputs
// Usage:
//   cargo run --package fhirpath-dev-tools --bin convert-fhir-xml -- <input.xml> <output.json>

use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn unescape_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[derive(Debug, Default)]
struct Node {
    name: String,
    value_attr: Option<String>,
    children: Map<String, Value>,
}

fn append_child(parent: &mut Node, child: Value, name: &str) {
    match parent.children.get_mut(name) {
        Some(existing) => {
            if existing.is_array() {
                existing.as_array_mut().unwrap().push(child);
            } else {
                let old = existing.clone();
                *existing = Value::Array(vec![old, child]);
            }
        }
        None => {
            parent.children.insert(name.to_string(), child);
        }
    }
}

fn from_xml(input: &str) -> Result<Value, String> {
    let mut reader = Reader::from_str(input);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut stack: Vec<Node> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let mut node = Node::default();
                node.name = String::from_utf8_lossy(e.name().into_inner()).to_string();
                // capture value attribute for FHIR primitive elements
                for attr in e.attributes().with_checks(false) {
                    if let Ok(a) = attr {
                        if a.key == QName(b"value") {
                            node.value_attr = Some(unescape_html_entities(&String::from_utf8_lossy(&a.value)));
                        }
                    }
                }
                stack.push(node);
            }
            Ok(Event::Empty(e)) => {
                let mut node = Node::default();
                node.name = String::from_utf8_lossy(e.name().into_inner()).to_string();
                for attr in e.attributes().with_checks(false) {
                    if let Ok(a) = attr {
                        if a.key == QName(b"value") {
                            node.value_attr = Some(unescape_html_entities(&String::from_utf8_lossy(&a.value)));
                        }
                    }
                }
                // Immediately attach to parent
                if let Some(parent) = stack.last_mut() {
                    let value = if let Some(v) = node.value_attr.take() {
                        Value::String(v)
                    } else {
                        Value::Object(Map::new())
                    };
                    append_child(parent, value, &node.name);
                } else {
                    // Empty root - unlikely; create object
                    let mut root_obj = Map::new();
                    root_obj.insert("resourceType".to_string(), Value::String(node.name));
                    return Ok(Value::Object(root_obj));
                }
            }
            Ok(Event::End(_e)) => {
                if let Some(mut node) = stack.pop() {
                    let value = if node.children.is_empty() {
                        if let Some(v) = node.value_attr.take() {
                            Value::String(v)
                        } else {
                            Value::Object(Map::new())
                        }
                    } else {
                        Value::Object(node.children)
                    };

                    if let Some(parent) = stack.last_mut() {
                        append_child(parent, value, &node.name);
                    } else {
                        // root
                        let mut root_obj = Map::new();
                        root_obj.insert("resourceType".to_string(), Value::String(node.name));
                        if let Value::Object(o) = value {
                            for (k, v) in o.into_iter() {
                                root_obj.insert(k, v);
                            }
                        }
                        return Ok(Value::Object(root_obj));
                    }
                }
            }
            Ok(Event::Eof) => break,
            Ok(Event::Text(_t)) => {
                // FHIR XML primitives typically use value= attributes; ignore text nodes
            }
            Err(e) => return Err(format!("XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Err("Unexpected end of XML".to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.xml> <output.json>", args[0]);
        std::process::exit(1);
    }
    let input_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);

    let xml = fs::read_to_string(&input_path)?;
    let json_value = from_xml(&xml).map_err(|e| format!("Conversion failed: {e}"))?;
    fs::write(&output_path, serde_json::to_string_pretty(&json_value)?)?;
    println!("✅ Converted {} -> {}", input_path.display(), output_path.display());
    Ok(())
}
