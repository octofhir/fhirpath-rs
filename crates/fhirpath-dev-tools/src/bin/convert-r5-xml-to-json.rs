// Convert the official R5 XML FHIRPath tests into grouped JSON suites
// Usage:
//   cargo run --bin convert-r5-xml-to-json -- specs/fhirpath/tests/tests-fhir-r5.xml

use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonTestCase {
    name: String,
    expression: String,
    #[serde(default)]
    input: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")] 
    inputfile: Option<String>,
    expected: Value,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")] 
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] 
    expecterror: Option<bool>,
    #[serde(rename = "expectError", skip_serializing_if = "Option::is_none")] 
    expect_error_alias: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")] 
    disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")] 
    predicate: Option<bool>,
    #[serde(rename = "skipStaticCheck", skip_serializing_if = "Option::is_none")] 
    skip_static_check: Option<bool>,
    #[serde(rename = "invalidKind", skip_serializing_if = "Option::is_none")] 
    invalid_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] 
    mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonTestSuite {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    source: Option<String>,
    tests: Vec<JsonTestCase>,
}

fn unescape_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn xml_text_to_value(ty: &str, text: &str) -> Value {
    let trimmed = text.trim();
    match ty {
        "boolean" => match trimmed {
            "true" | "True" | "TRUE" => Value::Bool(true),
            "false" | "False" | "FALSE" => Value::Bool(false),
            _ => Value::Null,
        },
        "integer" => trimmed.parse::<i64>().map(|v| Value::Number(v.into())).unwrap_or(Value::Null),
        "decimal" => serde_json::Number::from_f64(trimmed.parse::<f64>().unwrap_or(0.0))
            .map(Value::Number)
            .unwrap_or(Value::Null),
        // Strip '@' leading for date types
        "date" | "dateTime" | "time" => Value::String(unescape_html_entities(trimmed.strip_prefix('@').unwrap_or(trimmed))),
        "code" | "string" => Value::String(unescape_html_entities(trimmed)),
        _ => Value::String(unescape_html_entities(trimmed)),
    }
}

fn map_inputfile(inputfile: &str) -> String {
    if inputfile.ends_with(".xml") {
        format!("{}", inputfile.trim_end_matches(".xml").to_string() + ".json")
    } else {
        inputfile.to_string()
    }
}

fn as_bool(s: &str) -> Option<bool> {
    match s.trim() {
        "true" | "True" | "TRUE" => Some(true),
        "false" | "False" | "FALSE" => Some(false),
        _ => None,
    }
}

fn sanitize_group_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

fn parse_groups(xml_path: &Path) -> Result<HashMap<String, JsonTestSuite>, String> {
    let bytes = fs::read(xml_path).map_err(|e| format!("read {}: {}", xml_path.display(), e))?;
    let mut reader = Reader::from_reader(&bytes[..]);
    // Trim whitespace in text nodes for simpler parsing
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut in_group = false;
    let mut in_test = false;
    // We will read expression/output text via read_text, no flags needed

    let mut current_group_name = String::new();
    let mut current_group_desc: Option<String> = None;

    let mut current_test_name = String::new();
    let mut current_test_desc: Option<String> = None;
    let mut current_inputfile: Option<String> = None;
    let mut current_expression = String::new();
    let mut current_expect_error = false;
    let mut current_output_type: Option<String> = None;
    let mut current_expected: Vec<Value> = Vec::new();
    let mut current_disabled: Option<bool> = None;
    let mut current_predicate: Option<bool> = None;
    let mut current_skip_static: Option<bool> = None;
    let mut current_invalid_kind: Option<String> = None;
    let mut current_expr_mode: Option<String> = None;

    let mut groups: HashMap<String, JsonTestSuite> = HashMap::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "group" => {
                        in_group = true;
                        current_group_name.clear();
                        current_group_desc = None;
                        for a in e.attributes().flatten() {
                            if let Ok(k) = std::str::from_utf8(a.key.as_ref()) {
                                let v = a.unescape_value().unwrap_or_default().to_string();
                                match k {
                                    "name" => current_group_name = v,
                                    "description" => current_group_desc = Some(v),
                                    _ => {}
                                }
                            }
                        }
                        let key = current_group_name.clone();
                        groups.entry(key.clone()).or_insert(JsonTestSuite {
                            name: current_group_name.clone(),
                            description: current_group_desc.clone(),
                            source: Some("fhir-test-cases r5".to_string()),
                            tests: Vec::new(),
                        });
                    }
                    "test" => {
                        in_test = true;
                        current_test_name.clear();
                        current_test_desc = None;
                        current_inputfile = None;
                        current_expression.clear();
                        current_expect_error = false;
                        current_expected.clear();
                        current_disabled = None;
                        current_predicate = None;
                        current_skip_static = None;
                        current_invalid_kind = None;
                        current_expr_mode = None;
                        for a in e.attributes().flatten() {
                            if let Ok(k) = std::str::from_utf8(a.key.as_ref()) {
                                let v = a.unescape_value().unwrap_or_default().to_string();
                                match k {
                                    "name" => current_test_name = v,
                                    "description" => current_test_desc = Some(v),
                                    "inputfile" => current_inputfile = Some(map_inputfile(&v)),
                                    "disabled" => current_disabled = as_bool(&v),
                                    "predicate" => current_predicate = as_bool(&v),
                                    "skipStaticCheck" => current_skip_static = as_bool(&v),
                                    _ => {}
                                }
                            }
                        }
                    }
                    "expression" => {
                        // Capture expression attributes
                        for a in e.attributes().flatten() {
                            if let Ok(k) = std::str::from_utf8(a.key.as_ref()) {
                                if k == "invalid" {
                                    current_expect_error = true;
                                    current_invalid_kind = Some(
                                        a.unescape_value().unwrap_or_default().to_string(),
                                    );
                                } else if k == "mode" {
                                    current_expr_mode = Some(
                                        a.unescape_value().unwrap_or_default().to_string(),
                                    );
                                }
                            }
                        }

                        // Read expression text content (unescaped and trimmed)
                        let expr_text = reader
                            .read_text(QName(b"expression"))
                            .unwrap_or_default()
                            .into_owned();
                        current_expression = unescape_html_entities(expr_text.trim());
                    }
                    "output" => {
                        // Capture output type
                        current_output_type = None;
                        for a in e.attributes().flatten() {
                            if let Ok(k) = std::str::from_utf8(a.key.as_ref()) {
                                if k == "type" {
                                    current_output_type = Some(
                                        a.unescape_value().unwrap_or_default().to_string(),
                                    );
                                }
                            }
                        }

                        // Read output text in one shot and convert
                        let out_text = reader
                            .read_text(QName(b"output"))
                            .unwrap_or_default()
                            .into_owned();
                        let ty = current_output_type.as_deref().unwrap_or("string");
                        current_expected.push(xml_text_to_value(ty, &out_text));
                        current_output_type = None;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "test" => {
                        if in_test {
                            // build case
                            let expected = Value::Array(current_expected.clone());
                            let tags = vec!["r5-xml".to_string(), current_group_name.clone()];
                            let case = JsonTestCase {
                                name: current_test_name.clone(),
                                expression: current_expression.clone(),
                                input: Some(Value::Null),
                                inputfile: current_inputfile.clone(),
                                expected,
                                tags,
                                description: current_test_desc.clone(),
                                // Standardize on camelCase key only to avoid duplicates
                                expecterror: None,
                                expect_error_alias: current_expect_error.then_some(true),
                                disabled: current_disabled,
                                predicate: current_predicate,
                                skip_static_check: current_skip_static,
                                invalid_kind: current_invalid_kind.clone(),
                                mode: current_expr_mode.clone(),
                            };

                            if let Some(suite) = groups.get_mut(&current_group_name) {
                                suite.tests.push(case);
                            }
                            in_test = false;
                        }
                    }
                    "group" => {
                        in_group = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(_)) => { /* handled via read_text_into */ }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }

    Ok(groups)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!(
            "Usage: {} <path-to-tests-fhir-r5.xml>",
            args[0]
        );
        std::process::exit(1);
    }
    let xml_path = Path::new(&args[1]);
    println!("📖 Converting XML: {}", xml_path.display());

    let groups = parse_groups(xml_path).map_err(|e| format!("Parse failed: {e}"))?;

    // Write JSON suites into the same directory as the XML file
    let out_dir = xml_path.parent().unwrap_or_else(|| Path::new("."));
    let mut files_written = 0usize;
    for (group_name, suite) in groups {
        let file_name = format!("{}.json", sanitize_group_name(&group_name));
        let path = out_dir.join(file_name);
        let json = serde_json::to_string_pretty(&suite)?;
        fs::write(&path, json)?;
        files_written += 1;
        println!("📝 Wrote {} ({} tests)", path.display(), suite.tests.len());
    }

    println!("✅ Done. Wrote {} group files.", files_written);
    Ok(())
}
