//! Convert an official FHIRPath XML test suite (fhir-test-cases `tests-fhir-r5.xml`,
//! `tests-fhir-r6.xml`) into one JSON suite file per `<group>`.
//!
//! The conversion is lossless: every attribute and child element the test schema
//! defines is carried over, outputs keep their source text next to the typed value,
//! and an attribute or element the converter does not know is an error. After writing,
//! the suites are read back and checked against the XML test by test.
//!
//! Usage:
//!   convert-r5-xml-to-json <tests.xml> [--out DIR] [--fhir-version r5|r6] [--source TEXT]
//!
//! Without `--out` the suites are written next to the XML file.

use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const TESTS_NS: &str = "http://hl7.org/fhirpath/tests";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Capability {
    code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
}

/// An `<output>` exactly as written: its declared type and its text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct RawOutput {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    output_type: Option<String>,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct JsonTestCase {
    name: String,
    expression: String,
    input: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inputfile: Option<String>,
    expected: Value,
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(rename = "expectError", skip_serializing_if = "Option::is_none")]
    expect_error: Option<bool>,
    #[serde(rename = "invalidKind", skip_serializing_if = "Option::is_none")]
    invalid_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    predicate: Option<bool>,
    #[serde(rename = "skipStaticCheck", skip_serializing_if = "Option::is_none")]
    skip_static_check: Option<bool>,
    /// The test's `mode`, else its group's.
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ordered: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    testing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
    #[serde(rename = "contextType", skip_serializing_if = "Option::is_none")]
    context_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(rename = "versionTo", skip_serializing_if = "Option::is_none")]
    version_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    capabilities: Vec<Capability>,
    #[serde(rename = "outputTypes", skip_serializing_if = "Option::is_none")]
    output_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    outputs: Vec<RawOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonTestSuite {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(rename = "fhirVersion", skip_serializing_if = "Option::is_none")]
    fhir_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(rename = "versionTo", skip_serializing_if = "Option::is_none")]
    version_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    capabilities: Vec<Capability>,
    tests: Vec<JsonTestCase>,
}

struct Options {
    fhir_version: String,
    source: String,
}

/// The local name of an element in the tests namespace, or an error for anything else.
fn tag<'a>(node: Node<'a, '_>) -> Result<&'a str, String> {
    match node.tag_name().namespace() {
        Some(TESTS_NS) | None => Ok(node.tag_name().name()),
        Some(ns) => Err(format!(
            "element <{}> in unexpected namespace {ns}",
            node.tag_name().name()
        )),
    }
}

/// Reject any attribute of `node` not in `allowed`.
fn check_attributes(node: Node, allowed: &[&str], what: &str) -> Result<(), String> {
    for attr in node.attributes() {
        if attr.namespace().is_some() {
            continue; // xmlns:*, xsi:* and friends
        }
        if !allowed.contains(&attr.name()) {
            return Err(format!("unknown attribute '{}' on {what}", attr.name()));
        }
    }
    Ok(())
}

fn attr(node: Node, name: &str) -> Option<String> {
    node.attribute(name).map(str::to_string)
}

fn bool_attr(node: Node, name: &str) -> Result<Option<bool>, String> {
    match node.attribute(name) {
        None => Ok(None),
        Some("true") => Ok(Some(true)),
        Some("false") => Ok(Some(false)),
        Some(other) => Err(format!("attribute '{name}' is not a boolean: '{other}'")),
    }
}

/// All text inside an element (text and CDATA), unescaped.
fn text(node: Node) -> String {
    node.descendants()
        .filter(|n| n.is_text())
        .filter_map(|n| n.text())
        .collect()
}

fn capability(node: Node) -> Result<Capability, String> {
    check_attributes(node, &["code", "value"], "<capability>")?;
    Ok(Capability {
        code: attr(node, "code").ok_or("<capability> without code")?,
        value: attr(node, "value"),
    })
}

/// Typed JSON value for an `<output>`, as the runners compare it.
fn output_value(output_type: &str, text: &str) -> Value {
    let trimmed = text.trim();
    match output_type {
        "boolean" => match trimmed {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            _ => Value::String(trimmed.to_string()),
        },
        "integer" => trimmed
            .parse::<i64>()
            .map(|v| Value::Number(v.into()))
            .unwrap_or_else(|_| Value::String(trimmed.to_string())),
        "decimal" => trimmed
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(trimmed.to_string())),
        "date" | "dateTime" | "time" => {
            Value::String(trimmed.strip_prefix('@').unwrap_or(trimmed).to_string())
        }
        _ => Value::String(trimmed.to_string()),
    }
}

/// Input files are referenced by their XML names; the suites use the JSON conversions.
fn map_inputfile(inputfile: &str) -> String {
    match inputfile.strip_suffix(".xml") {
        Some(stem) => format!("{stem}.json"),
        None => inputfile.to_string(),
    }
}

fn parse_test(
    node: Node,
    group: &str,
    group_mode: Option<&str>,
    opts: &Options,
) -> Result<JsonTestCase, String> {
    let name = attr(node, "name").ok_or("<test> without name")?;
    let ctx = |e: String| format!("test '{name}': {e}");
    check_attributes(
        node,
        &[
            "name",
            "description",
            "inputfile",
            "predicate",
            "skipStaticCheck",
            "ordered",
            "mode",
            "version",
            "versionTo",
            "reference",
            "testing",
            "context",
            "contextType",
            "disabled",
        ],
        "<test>",
    )
    .map_err(ctx)?;

    let mut expression = None;
    let mut invalid_kind = None;
    let mut outputs = Vec::new();
    let mut notes = None;
    let mut capabilities = Vec::new();
    for child in node.children().filter(Node::is_element) {
        match tag(child).map_err(ctx)? {
            "expression" => {
                if expression.is_some() {
                    return Err(ctx("more than one <expression>".into()));
                }
                check_attributes(child, &["invalid"], "<expression>").map_err(ctx)?;
                invalid_kind = attr(child, "invalid");
                expression = Some(text(child).trim().to_string());
            }
            "output" => {
                check_attributes(child, &["type"], "<output>").map_err(ctx)?;
                outputs.push(RawOutput {
                    output_type: attr(child, "type"),
                    value: text(child),
                });
            }
            "notes" => notes = Some(text(child)),
            "capability" => capabilities.push(capability(child).map_err(ctx)?),
            other => return Err(ctx(format!("unknown element <{other}>"))),
        }
    }

    let expected = Value::Array(
        outputs
            .iter()
            .map(|o| output_value(o.output_type.as_deref().unwrap_or("string"), &o.value))
            .collect(),
    );
    let output_types: Vec<String> = outputs
        .iter()
        .map(|o| o.output_type.clone().unwrap_or_else(|| "string".into()))
        .collect();

    Ok(JsonTestCase {
        expression: expression.ok_or_else(|| ctx("no <expression>".into()))?,
        input: None,
        inputfile: attr(node, "inputfile").map(|f| map_inputfile(&f)),
        expected,
        tags: vec![format!("official-{}", opts.fhir_version), group.to_string()],
        description: attr(node, "description"),
        expect_error: invalid_kind.is_some().then_some(true),
        invalid_kind,
        disabled: bool_attr(node, "disabled").map_err(ctx)?,
        predicate: bool_attr(node, "predicate").map_err(ctx)?,
        skip_static_check: bool_attr(node, "skipStaticCheck").map_err(ctx)?,
        mode: attr(node, "mode").or_else(|| group_mode.map(str::to_string)),
        ordered: bool_attr(node, "ordered").map_err(ctx)?,
        testing: attr(node, "testing"),
        context: attr(node, "context"),
        context_type: attr(node, "contextType"),
        version: attr(node, "version"),
        version_to: attr(node, "versionTo"),
        reference: attr(node, "reference"),
        notes,
        capabilities,
        output_types: (!output_types.is_empty()).then_some(output_types),
        outputs,
        name,
    })
}

fn parse_suites(xml: &str, opts: &Options) -> Result<Vec<JsonTestSuite>, String> {
    let doc = Document::parse(xml).map_err(|e| format!("XML parse error: {e}"))?;
    let root = doc.root_element();
    if tag(root)? != "tests" {
        return Err("root element is not <tests>".into());
    }
    check_attributes(
        root,
        &["name", "description", "reference", "version", "versionTo"],
        "<tests>",
    )?;

    let mut suites = Vec::new();
    let mut test_names = HashSet::new();
    for group in root.children().filter(Node::is_element) {
        match tag(group)? {
            "group" => {}
            // Suite-level notes/capabilities have no per-group home and are not used.
            "notes" | "capability" => {
                return Err(format!(
                    "top-level <{}> is not supported by the converter",
                    tag(group)?
                ));
            }
            other => return Err(format!("unknown element <{other}> in <tests>")),
        }
        check_attributes(
            group,
            &[
                "name",
                "description",
                "reference",
                "mode",
                "version",
                "versionTo",
            ],
            "<group>",
        )?;
        let name = attr(group, "name").ok_or("<group> without name")?;
        let mode = attr(group, "mode");
        let mut suite = JsonTestSuite {
            name: name.clone(),
            description: attr(group, "description"),
            source: Some(opts.source.clone()),
            fhir_version: Some(opts.fhir_version.clone()),
            mode: mode.clone(),
            version: attr(group, "version"),
            version_to: attr(group, "versionTo"),
            reference: attr(group, "reference"),
            notes: None,
            capabilities: Vec::new(),
            tests: Vec::new(),
        };
        for child in group.children().filter(Node::is_element) {
            match tag(child)? {
                "test" => {
                    let test = parse_test(child, &name, mode.as_deref(), opts)?;
                    if !test_names.insert(test.name.clone()) {
                        return Err(format!("duplicate test name '{}'", test.name));
                    }
                    suite.tests.push(test);
                }
                "notes" => suite.notes = Some(text(child)),
                "capability" => suite.capabilities.push(capability(child)?),
                other => return Err(format!("unknown element <{other}> in group '{name}'")),
            }
        }
        suites.push(suite);
    }
    Ok(suites)
}

fn file_name(group: &str) -> String {
    let safe: String = group
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("{safe}.json")
}

/// Read the written suites back and compare them with what was parsed from the XML.
fn verify(out_dir: &Path, suites: &[JsonTestSuite], xml: &str) -> Result<usize, String> {
    let declared = Document::parse(xml)
        .map_err(|e| e.to_string())?
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "test")
        .count();
    let mut total = 0;
    for suite in suites {
        let path = out_dir.join(file_name(&suite.name));
        let back: JsonTestSuite = serde_json::from_str(
            &fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?,
        )
        .map_err(|e| format!("{}: {e}", path.display()))?;
        if back.tests != suite.tests {
            return Err(format!("{} does not read back identically", path.display()));
        }
        total += back.tests.len();
    }
    if total != declared {
        return Err(format!(
            "XML declares {declared} <test> elements but {total} were written"
        ));
    }
    Ok(total)
}

fn main() {
    if let Err(e) = run() {
        eprintln!("❌ {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut xml_path = None;
    let mut out_dir = None;
    let mut fhir_version = None;
    let mut source = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out_dir = args.next().map(PathBuf::from),
            "--fhir-version" => fhir_version = args.next(),
            "--source" => source = args.next(),
            _ if xml_path.is_none() => xml_path = Some(PathBuf::from(arg)),
            _ => return Err(format!("unexpected argument '{arg}'")),
        }
    }
    let xml_path = xml_path.ok_or(
        "usage: convert-r5-xml-to-json <tests.xml> [--out DIR] [--fhir-version r5|r6] [--source TEXT]",
    )?;
    let fhir_version = fhir_version.unwrap_or_else(|| "r5".into());
    let opts = Options {
        source: source.unwrap_or_else(|| format!("fhir-test-cases {fhir_version}")),
        fhir_version,
    };
    let out_dir = out_dir.unwrap_or_else(|| {
        xml_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    });

    println!("📖 Converting {}", xml_path.display());
    let xml = fs::read_to_string(&xml_path).map_err(|e| format!("{}: {e}", xml_path.display()))?;
    let suites = parse_suites(&xml, &opts)?;

    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let mut files = BTreeMap::new();
    for suite in &suites {
        let name = file_name(&suite.name);
        if files.insert(name.clone(), suite.name.clone()).is_some() {
            return Err(format!("two groups map to the file name {name}"));
        }
        let json = serde_json::to_string_pretty(suite).map_err(|e| e.to_string())?;
        fs::write(out_dir.join(&name), json + "\n").map_err(|e| e.to_string())?;
    }

    let total = verify(&out_dir, &suites, &xml)?;
    println!(
        "✅ Wrote {} suites ({total} tests) to {}, verified against the XML",
        suites.len(),
        out_dir.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> Options {
        Options {
            fhir_version: "r5".into(),
            source: "test".into(),
        }
    }

    const XML: &str = r#"<tests xmlns="http://hl7.org/fhirpath/tests" name="t">
      <group name="g1" mode="strict" description="d">
        <test name="a" testing="=" inputfile="patient-example.xml" context="Observation.effective" contextType="Period" ordered="false" version="2.1.0">
          <expression invalid="semantic">'1&lt;2' &amp; 'x'</expression>
          <output type="decimal">1.50</output>
          <output type="date">@2014</output>
        </test>
        <test name="b" mode="tx"><expression><![CDATA[1 < 2]]></expression><output type="boolean">true</output></test>
      </group>
    </tests>"#;

    #[test]
    fn carries_every_attribute_and_keeps_output_text() {
        let suites = parse_suites(XML, &opts()).unwrap();
        let a = &suites[0].tests[0];
        assert_eq!(a.expression, "'1<2' & 'x'");
        assert_eq!(a.invalid_kind.as_deref(), Some("semantic"));
        assert_eq!(a.expect_error, Some(true));
        assert_eq!(a.mode.as_deref(), Some("strict"));
        assert_eq!(a.context.as_deref(), Some("Observation.effective"));
        assert_eq!(a.context_type.as_deref(), Some("Period"));
        assert_eq!(a.ordered, Some(false));
        assert_eq!(a.testing.as_deref(), Some("="));
        assert_eq!(a.version.as_deref(), Some("2.1.0"));
        assert_eq!(a.inputfile.as_deref(), Some("patient-example.json"));
        assert_eq!(a.outputs[0].value, "1.50");
        assert_eq!(a.expected, serde_json::json!([1.5, "2014"]));
        let b = &suites[0].tests[1];
        assert_eq!(b.expression, "1 < 2");
        assert_eq!(b.mode.as_deref(), Some("tx"));
    }

    #[test]
    fn unknown_attributes_and_elements_are_errors() {
        let attr = XML.replace("version=\"2.1.0\"", "surprise=\"1\"");
        assert!(parse_suites(&attr, &opts()).is_err());
        let elem = XML.replace("<output type=\"date\">@2014</output>", "<extra/>");
        assert!(parse_suites(&elem, &opts()).is_err());
    }
}
