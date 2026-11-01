//! Convert FHIR XML resources to FHIR JSON, driven by the embedded FHIR schemas.
//!
//! The element definitions decide what a JSON property looks like: whether it is an
//! array, whether a primitive is a JSON boolean, number or string, and where a nested
//! resource or an XHTML narrative starts. Primitive `id`/`extension` go to the `_name`
//! sibling property, decimals keep the digits they were written with, and the narrative
//! `div` is copied from the source text unchanged.
//!
//! An element the schema does not know is an error, not a guess, so a conversion never
//! silently drops or reshapes data.
//!
//! Usage:
//!   convert-fhir-xml [--fhir-version r5] <input.xml> <output.json>
//!   convert-fhir-xml [--fhir-version r5] <source_dir> <target_dir>

use octofhir_fhirschema::{FhirSchema, FhirSchemaElement, FhirVersion, get_schema};
use roxmltree::{Document, Node};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const FHIR_NS: &str = "http://hl7.org/fhir";
/// Prefix marking a decimal whose source text is spliced into the JSON output verbatim.
const DECIMAL_MARK: &str = "\u{0}decimal:";

type Elements = HashMap<String, FhirSchemaElement>;

/// Where child element definitions are looked up.
#[derive(Clone, Copy)]
enum Scope<'a> {
    /// A named type: its own elements, then those of its base types.
    Type(&'a str),
    /// An inline backbone element: its own elements, then those of `base`.
    Backbone(&'a Elements, &'a str),
}

struct Converter {
    version: FhirVersion,
}

impl Converter {
    fn schema(&self, name: &str) -> Result<&'static FhirSchema, String> {
        get_schema(self.version, name)
            .ok_or_else(|| format!("no {} schema for type '{name}'", self.version.as_str()))
    }

    /// Find the definition of element `name` for `type_name` or one of its base types.
    fn lookup_in_type(
        &self,
        type_name: &str,
        name: &str,
    ) -> Result<Option<&'static FhirSchemaElement>, String> {
        let mut current = Some(self.schema(type_name)?);
        while let Some(schema) = current {
            if let Some(element) = schema.elements.as_ref().and_then(|e| e.get(name)) {
                return Ok(Some(element));
            }
            current = match schema.base.as_deref() {
                Some(base) => Some(self.schema(base.rsplit('/').next().unwrap_or(base))?),
                None => None,
            };
        }
        Ok(None)
    }

    fn lookup(
        &self,
        scope: Scope<'static>,
        name: &str,
    ) -> Result<&'static FhirSchemaElement, String> {
        let found = match scope {
            Scope::Type(type_name) => self.lookup_in_type(type_name, name)?,
            Scope::Backbone(elements, base) => match elements.get(name) {
                Some(element) => Some(element),
                None => self.lookup_in_type(base, name)?,
            },
        };
        let element = found.ok_or_else(|| format!("unknown element '{name}'"))?;
        match &element.element_reference {
            Some(path) => self.resolve_reference(path),
            None => Ok(element),
        }
    }

    /// Resolve a `contentReference` path such as `[<url>, "elements", "item"]`.
    fn resolve_reference(&self, path: &[String]) -> Result<&'static FhirSchemaElement, String> {
        let url = path.first().ok_or("empty element reference")?;
        let schema = self.schema(url.rsplit('/').next().unwrap_or(url))?;
        let mut elements = schema.elements.as_ref();
        let mut element = None;
        for key in path[1..].iter().filter(|k| *k != "elements") {
            element = elements.and_then(|e| e.get(key));
            elements = element.and_then(|e| e.elements.as_ref());
        }
        element.ok_or_else(|| format!("cannot resolve element reference {path:?}"))
    }

    fn is_primitive(&self, type_name: &str) -> Result<bool, String> {
        Ok(self.schema(type_name)?.kind == "primitive-type")
    }

    fn convert_resource(&self, node: Node, source: &str) -> Result<Value, String> {
        let resource_type = node.tag_name().name();
        let schema = self.schema(resource_type)?;
        if schema.kind != "resource" {
            return Err(format!("'{resource_type}' is not a resource"));
        }
        let mut obj = Map::new();
        obj.insert("resourceType".into(), Value::String(resource_type.into()));
        let type_name: &'static str = schema.name.as_str();
        self.convert_children(node, Scope::Type(type_name), &mut obj, source)?;
        Ok(Value::Object(obj))
    }

    /// Convert the child elements of `node` into properties of `obj`.
    fn convert_children(
        &self,
        node: Node,
        scope: Scope<'static>,
        obj: &mut Map<String, Value>,
        source: &str,
    ) -> Result<(), String> {
        // Group repeated elements, keeping document order of first appearance.
        let mut order: Vec<&str> = Vec::new();
        let mut groups: HashMap<&str, Vec<Node>> = HashMap::new();
        for child in node.children().filter(Node::is_element) {
            let name = child.tag_name().name();
            if !groups.contains_key(name) {
                order.push(name);
            }
            groups.entry(name).or_default().push(child);
        }

        for name in order {
            let nodes = &groups[name];
            let element = self
                .lookup(scope, name)
                .map_err(|e| format!("{e} (in <{}>)", node.tag_name().name()))?;
            let is_array = element.array.unwrap_or(false);
            if !is_array && nodes.len() > 1 {
                return Err(format!("element '{name}' repeats but is not an array"));
            }
            let type_name = element
                .type_name
                .as_deref()
                .ok_or_else(|| format!("element '{name}' has no type"))?;

            if type_name == "xhtml" {
                let values: Vec<Value> = nodes
                    .iter()
                    .map(|n| Value::String(source[n.range()].to_string()))
                    .collect();
                obj.insert(name.into(), pack(values, is_array));
            } else if self.is_primitive(type_name)? {
                let mut values = Vec::new();
                let mut extras = Vec::new();
                for n in nodes {
                    values.push(match n.attribute("value") {
                        Some(text) => primitive_value(type_name, text)?,
                        None => Value::Null,
                    });
                    let mut extra = Map::new();
                    if let Some(id) = n.attribute("id") {
                        extra.insert("id".into(), Value::String(id.into()));
                    }
                    self.convert_children(*n, Scope::Type("Element"), &mut extra, source)?;
                    extras.push(if extra.is_empty() {
                        Value::Null
                    } else {
                        Value::Object(extra)
                    });
                }
                if values.iter().any(|v| !v.is_null()) {
                    obj.insert(name.into(), pack(values, is_array));
                }
                if extras.iter().any(|v| !v.is_null()) {
                    obj.insert(format!("_{name}"), pack(extras, is_array));
                }
            } else {
                let mut values = Vec::new();
                for n in nodes {
                    values.push(self.convert_complex(*n, element, type_name, source)?);
                }
                obj.insert(name.into(), pack(values, is_array));
            }
        }
        Ok(())
    }

    fn convert_complex(
        &self,
        node: Node,
        element: &'static FhirSchemaElement,
        type_name: &'static str,
        source: &str,
    ) -> Result<Value, String> {
        if type_name == "Resource" {
            let inner: Vec<Node> = node.children().filter(Node::is_element).collect();
            return match inner.as_slice() {
                [resource] => self.convert_resource(*resource, source),
                _ => Err(format!(
                    "<{}> must hold exactly one resource",
                    node.tag_name().name()
                )),
            };
        }
        let mut obj = Map::new();
        for attr in node.attributes() {
            if attr.namespace().is_none() && matches!(attr.name(), "id" | "url") {
                obj.insert(attr.name().into(), Value::String(attr.value().into()));
            }
        }
        let scope = match &element.elements {
            Some(elements) => Scope::Backbone(elements, type_name),
            None => Scope::Type(type_name),
        };
        self.convert_children(node, scope, &mut obj, source)?;
        Ok(Value::Object(obj))
    }
}

fn pack(mut values: Vec<Value>, is_array: bool) -> Value {
    if is_array {
        Value::Array(values)
    } else {
        values.pop().unwrap_or(Value::Null)
    }
}

/// The JSON form of a primitive's `value` attribute.
fn primitive_value(type_name: &str, text: &str) -> Result<Value, String> {
    let bad = || format!("invalid {type_name} value '{text}'");
    Ok(match type_name {
        "boolean" => match text {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            _ => return Err(bad()),
        },
        "integer" | "positiveInt" | "unsignedInt" => {
            Value::Number(text.parse::<i64>().map_err(|_| bad())?.into())
        }
        "decimal" => {
            if !is_json_number(text) {
                return Err(bad());
            }
            Value::String(format!("{DECIMAL_MARK}{text}"))
        }
        // integer64 is a JSON string in R5+, as are all other primitives
        _ => Value::String(text.into()),
    })
}

/// Whether `text` is a JSON number literal (FHIR decimals are a subset of these).
fn is_json_number(text: &str) -> bool {
    let s = text.strip_prefix('-').unwrap_or(text);
    let (mantissa, exponent) = match s.find(['e', 'E']) {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    };
    let (int, frac) = match mantissa.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (mantissa, None),
    };
    let digits = |d: &str| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit());
    let int_ok = digits(int) && (int == "0" || !int.starts_with('0'));
    let frac_ok = frac.is_none_or(digits);
    let exp_ok = exponent.is_none_or(|e| digits(e.strip_prefix(['+', '-']).unwrap_or(e)));
    int_ok && frac_ok && exp_ok
}

/// Pretty-print, then splice marked decimals back in as bare JSON numbers.
fn to_json_text(value: &Value) -> Result<String, String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    let mark = format!(
        "\"{}",
        serde_json::to_string(DECIMAL_MARK)
            .unwrap()
            .trim_matches('"')
    );
    let mut out = String::with_capacity(text.len());
    let mut rest = text.as_str();
    while let Some(start) = rest.find(&mark) {
        out.push_str(&rest[..start]);
        let after = &rest[start + mark.len()..];
        let end = after.find('"').ok_or("unterminated decimal marker")?;
        out.push_str(&after[..end]);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn from_xml(input: &str, version: FhirVersion) -> Result<String, String> {
    let doc = Document::parse(input).map_err(|e| format!("XML parse error: {e}"))?;
    let root = doc.root_element();
    if root.tag_name().namespace() != Some(FHIR_NS) {
        return Err(format!("root element is not in the {FHIR_NS} namespace"));
    }
    let value = Converter { version }.convert_resource(root, input)?;
    to_json_text(&value)
}

fn convert_file(input_path: &Path, output_path: &Path, version: FhirVersion) -> Result<(), String> {
    let xml = fs::read_to_string(input_path).map_err(|e| e.to_string())?;
    let json = from_xml(&xml, version)?;
    fs::write(output_path, json + "\n").map_err(|e| e.to_string())
}

fn convert_directory(
    source_dir: &Path,
    target_dir: &Path,
    version: FhirVersion,
) -> Result<(), String> {
    fs::create_dir_all(target_dir).map_err(|e| e.to_string())?;
    let mut paths: Vec<PathBuf> = fs::read_dir(source_dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|ext| ext == "xml"))
        .collect();
    paths.sort();

    let mut failed = 0;
    for path in &paths {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let output_path = target_dir.join(format!("{stem}.json"));
        match convert_file(path, &output_path, version) {
            Ok(()) => println!("✅ {} -> {}", path.display(), output_path.display()),
            Err(e) => {
                println!("❌ {}: {e}", path.display());
                failed += 1;
            }
        }
    }
    println!("{} converted, {failed} failed", paths.len() - failed);
    if failed > 0 {
        return Err(format!("{failed} file(s) failed to convert"));
    }
    Ok(())
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut version = FhirVersion::R5;
    if let Some(i) = args.iter().position(|a| a == "--fhir-version") {
        let value = args.get(i + 1).cloned().unwrap_or_default();
        version = FhirVersion::parse(&value).unwrap_or_else(|| {
            eprintln!("unknown FHIR version '{value}'");
            std::process::exit(2);
        });
        args.drain(i..=i + 1);
    }
    if args.len() != 2 {
        eprintln!("Usage:");
        eprintln!("  convert-fhir-xml [--fhir-version r5] <input.xml> <output.json>");
        eprintln!("  convert-fhir-xml [--fhir-version r5] <source_dir> <target_dir>");
        std::process::exit(2);
    }
    let source = PathBuf::from(&args[0]);
    let target = PathBuf::from(&args[1]);

    let result = if source.is_dir() {
        convert_directory(&source, &target, version)
    } else {
        convert_file(&source, &target, version)
            .map(|()| println!("✅ {} -> {}", source.display(), target.display()))
    };
    if let Err(e) = result {
        eprintln!("❌ {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn convert(xml: &str) -> Value {
        serde_json::from_str(&from_xml(xml, FhirVersion::R5).unwrap()).unwrap()
    }

    #[test]
    fn primitives_take_their_json_types_and_decimals_keep_their_digits() {
        let xml = r#"<Parameters xmlns="http://hl7.org/fhir">
            <parameter><name value="a"/><valueDecimal value="1.50"/></parameter>
            <parameter><name value="b"/><valueInteger value="2"/></parameter>
            <parameter><name value="c"/><valueBoolean value="true"/></parameter>
        </Parameters>"#;
        let text = from_xml(xml, FhirVersion::R5).unwrap();
        assert!(text.contains("\"valueDecimal\": 1.50"), "{text}");
        let json = convert(xml);
        assert_eq!(json["parameter"][0]["name"], "a");
        assert_eq!(json["parameter"][1]["valueInteger"], 2);
        assert_eq!(json["parameter"][2]["valueBoolean"], true);
    }

    #[test]
    fn primitive_extensions_go_to_the_underscore_property() {
        let json = convert(
            r#"<Patient xmlns="http://hl7.org/fhir">
                <name><given value="A"/><given id="g2"><extension url="http://x"><valueString value="v"/></extension></given></name>
            </Patient>"#,
        );
        assert_eq!(json["name"][0]["given"], serde_json::json!(["A", null]));
        assert_eq!(json["name"][0]["_given"][0], Value::Null);
        assert_eq!(json["name"][0]["_given"][1]["id"], "g2");
        assert_eq!(
            json["name"][0]["_given"][1]["extension"][0]["url"],
            "http://x"
        );
    }

    #[test]
    fn contained_resources_and_narrative() {
        let json = convert(
            r#"<Patient xmlns="http://hl7.org/fhir">
                <text><status value="generated"/><div xmlns="http://www.w3.org/1999/xhtml"><p class="x">Hi</p></div></text>
                <contained><Organization><id value="o"/></Organization></contained>
            </Patient>"#,
        );
        assert_eq!(
            json["text"]["div"],
            r#"<div xmlns="http://www.w3.org/1999/xhtml"><p class="x">Hi</p></div>"#
        );
        assert_eq!(json["contained"][0]["resourceType"], "Organization");
    }

    #[test]
    fn unknown_elements_are_an_error() {
        assert!(
            from_xml(
                r#"<Patient xmlns="http://hl7.org/fhir"><bogus value="1"/></Patient>"#,
                FhirVersion::R5
            )
            .is_err()
        );
    }

    #[test]
    fn json_number_check() {
        for ok in ["0", "1.50", "-9.5e2", "1.5E-3", "10"] {
            assert!(is_json_number(ok), "{ok}");
        }
        for bad in ["01", "1.", ".5", "+1", "1e", "abc"] {
            assert!(!is_json_number(bad), "{bad}");
        }
    }
}
