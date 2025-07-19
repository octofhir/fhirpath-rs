use fhirpath_core::evaluator::{evaluate_expression, EvaluationContext};
use fhirpath_core::model::{FhirPathValue, FhirResource};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH, Instant};

/// Rust FHIRPath Test Runner
///
/// This struct runs FHIRPath tests using the fhirpath-core implementation
/// and outputs results in a standardized format for comparison.
#[derive(Debug)]
pub struct RustTestRunner {
    test_data_dir: String,
    test_cases_dir: String,
    results_dir: String,
    test_config: TestConfig,
}

#[derive(Debug, Deserialize)]
struct TestConfig {
    #[serde(rename = "testData")]
    test_data: TestData,
    #[serde(rename = "sampleTests")]
    sample_tests: Vec<TestCase>,
    #[serde(rename = "benchmarkTests")]
    benchmark_tests: Vec<BenchmarkTest>,
}

#[derive(Debug, Deserialize)]
struct TestData {
    #[serde(rename = "inputFiles")]
    input_files: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TestCase {
    name: String,
    description: String,
    #[serde(rename = "inputFile")]
    input_file: String,
    expression: String,
    #[serde(rename = "expectedOutput")]
    expected_output: Option<Vec<ExpectedOutput>>,
    invalid: Option<bool>,
    group: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BenchmarkTest {
    name: String,
    description: String,
    #[serde(rename = "inputFile")]
    input_file: Option<String>,
    expression: String,
    iterations: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ExpectedOutput {
    #[serde(rename = "type")]
    output_type: String,
    value: Value,
}

#[derive(Debug, Serialize)]
struct TestResult {
    name: String,
    description: String,
    expression: String,
    status: String,
    execution_time_ms: f64,
    expected: Vec<Value>,
    actual: Option<Vec<Value>>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TestResults {
    language: String,
    timestamp: f64,
    tests: Vec<TestResult>,
    summary: TestSummary,
}

#[derive(Debug, Serialize)]
struct TestSummary {
    total: u32,
    passed: u32,
    failed: u32,
    errors: u32,
}

#[derive(Debug, Serialize)]
struct BenchmarkResult {
    name: String,
    description: String,
    expression: String,
    iterations: u32,
    avg_time_ms: f64,
    min_time_ms: f64,
    max_time_ms: f64,
    ops_per_second: f64,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkResults {
    language: String,
    timestamp: f64,
    benchmarks: Vec<BenchmarkResult>,
    system_info: SystemInfo,
}

#[derive(Debug, Serialize)]
struct SystemInfo {
    platform: String,
    rust_version: String,
    fhirpath_version: String,
}

impl RustTestRunner {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let current_dir = std::env::current_dir()?;
        // Point to the official test suites in fhirpath-core
        let test_data_dir = current_dir.join("../../../fhirpath-core/tests/official-tests/r4/input").to_string_lossy().to_string();
        let test_cases_dir = current_dir.join("../../../fhirpath-core/tests/official-tests/r4").to_string_lossy().to_string();
        let results_dir = current_dir.join("../../results").to_string_lossy().to_string();

        // Ensure results directory exists
        fs::create_dir_all(&results_dir)?;

        // Create default test configuration since official tests don't have a config file
        let test_config = TestConfig {
            test_data: TestData {
                input_files: vec![
                    "patient-example.xml".to_string(),
                    "observation-example.xml".to_string(),
                    "questionnaire-example.xml".to_string(),
                    "valueset-example-expansion.xml".to_string(),
                ],
            },
            sample_tests: Vec::new(),
            benchmark_tests: Vec::new(),
        };

        Ok(RustTestRunner {
            test_data_dir,
            test_cases_dir,
            results_dir,
            test_config,
        })
    }

    /// Load test data from XML file and convert to FhirResource.
    fn load_test_data(&self, filename: &str) -> Option<FhirResource> {
        let file_path = Path::new(&self.test_data_dir).join(filename);

        if !file_path.exists() {
            println!("⚠️  Test data file not found: {}", filename);
            return None;
        }

        match fs::read_to_string(&file_path) {
            Ok(xml_content) => {
                match self.convert_xml_to_json(&xml_content) {
                    Ok(json_value) => {
                        match FhirResource::from_json(json_value) {
                            Ok(resource) => Some(resource),
                            Err(e) => {
                                println!("⚠️  Error creating FhirResource from JSON {}: {}", filename, e);
                                None
                            }
                        }
                    },
                    Err(e) => {
                        println!("⚠️  Error converting XML to JSON {}: {}", filename, e);
                        None
                    }
                }
            }
            Err(e) => {
                println!("⚠️  Error reading test data {}: {}", filename, e);
                None
            }
        }
    }

    /// Check if an element name represents a FHIR polymorphic property
    /// In FHIR, polymorphic properties have names like "valueString", "valueInteger", etc.
    /// where "value" is the base name and "String" or "Integer" is the type
    fn is_fhir_polymorphic_property(&self, element_name: &str) -> bool {
        // Common FHIR polymorphic properties
        let polymorphic_bases = [
            "value", "component", "onset", "abatement", "asserted", "recorded",
            "onset", "offset", "target", "entity", "detail", "reason", "performer"
        ];

        // Check if the element name starts with any of the known polymorphic bases
        // and has a capital letter after the base (indicating a type)
        for base in polymorphic_bases {
            if element_name.starts_with(base) &&
               element_name.len() > base.len() &&
               element_name.chars().nth(base.len()).map_or(false, |c| c.is_uppercase()) {
                return true;
            }
        }

        false
    }

    /// Extract the base name and type name from a FHIR polymorphic property name
    /// For example, "valueString" would return ("value", "String")
    fn extract_polymorphic_parts(&self, element_name: &str) -> (String, String) {
        // Find the position of the first uppercase letter
        if let Some(pos) = element_name.chars().position(|c| c.is_uppercase()) {
            let base_name = element_name[..pos].to_string();
            let type_name = element_name[pos..].to_string();
            (base_name, type_name)
        } else {
            // Fallback if no uppercase letter is found
            (element_name.to_string(), String::new())
        }
    }

    /// Convert XML content to JSON following FHIR conventions
    fn convert_xml_to_json(&self, xml_content: &str) -> Result<Value, Box<dyn std::error::Error>> {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let mut reader = Reader::from_str(xml_content);
        reader.trim_text(true);

        let mut buf = Vec::new();
        let mut json_obj = serde_json::Map::new();
        let mut element_stack: Vec<(String, serde_json::Map<String, Value>, Option<String>)> = Vec::new();
        let mut root_element_name = String::new();
        let mut in_root = false;
        let mut event_count = 0;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    event_count += 1;
                    let element_name = String::from_utf8(e.name().as_ref().to_vec())?;
                    let mut current_obj = serde_json::Map::new();

                    // Handle attributes
                    for attr in e.attributes() {
                        let attr = attr?;
                        let attr_name = String::from_utf8(attr.key.as_ref().to_vec())?;
                        let attr_value = String::from_utf8(attr.value.to_vec())?;

                        // Skip xmlns attributes as they're not part of FHIR JSON
                        if attr_name.starts_with("xmlns") {
                            continue;
                        }

                        current_obj.insert(attr_name, Value::String(attr_value));
                    }

                    // Handle root element
                    if !in_root {
                        root_element_name = element_name.clone();
                        json_obj.insert("resourceType".to_string(), Value::String(element_name.clone()));
                        in_root = true;
                        // Push root element to stack so children can be processed
                        element_stack.push((element_name, current_obj, None));
                    } else {
                        element_stack.push((element_name, current_obj, None));
                    }
                }
                Ok(Event::End(ref e)) => {
                    event_count += 1;
                    let element_name = String::from_utf8(e.name().as_ref().to_vec())?;

                    if let Some((stack_element_name, mut current_obj, text_content)) = element_stack.pop() {
                        // Sanity check - element names should match
                        if stack_element_name != element_name {
                            return Err(format!("XML structure error: expected {}, got {}", stack_element_name, element_name).into());
                        }

                        // If this is the root element, process its children and break
                        if element_name == root_element_name {
                            // Add all accumulated children to the main json object
                            for (key, value) in current_obj {
                                self.add_to_object(&mut json_obj, key, value);
                            }
                            break;
                        }

                        // Handle text content
                        if let Some(text) = text_content {
                            // For FHIR, text content in most elements should be preserved as-is
                            // Special handling for div elements in narrative text
                            if element_name == "div" {
                                current_obj.insert("div".to_string(), Value::String(text));
                            } else {
                                // For other elements, if they have text content, it's usually the value
                                if current_obj.is_empty() {
                                    // Element has only text content, use it directly as a string value
                                    current_obj.insert("value".to_string(), Value::String(text));
                                } else {
                                    // Element has both attributes and text content
                                    current_obj.insert("value".to_string(), Value::String(text));
                                }
                            }
                        }

                        // Determine the final value for this element
                        let current_value = if current_obj.len() == 1 && current_obj.contains_key("value") {
                            // For FHIR elements with only a "value" attribute, use the value directly
                            current_obj.get("value").unwrap().clone()
                        } else if current_obj.is_empty() {
                            // For elements with no attributes or text content, create an empty object
                            // They might still have child elements that will be added later
                            Value::Object(current_obj)
                        } else {
                            Value::Object(current_obj)
                        };

                        // Add to parent or root
                        if element_stack.is_empty() {
                            // Direct child of root - add to main object

                            // Special handling for FHIR polymorphic properties
                            if self.is_fhir_polymorphic_property(&element_name) {
                                let (base_name, type_name) = self.extract_polymorphic_parts(&element_name);

                                // Create a new object with the type information
                                let mut typed_obj = serde_json::Map::new();

                                // If current_value is an object, extract its properties
                                if let Value::Object(obj) = current_value {
                                    for (k, v) in obj {
                                        typed_obj.insert(k, v);
                                    }
                                } else {
                                    // If it's not an object, use it as is
                                    typed_obj.insert("value".to_string(), current_value);
                                }

                                // Add type information
                                typed_obj.insert("type".to_string(), Value::String(type_name));

                                // Add to the main object with the base name
                                self.add_to_object(&mut json_obj, base_name, Value::Object(typed_obj));
                            } else {
                                // Regular property
                                self.add_to_object(&mut json_obj, element_name, current_value);
                            }
                        } else {
                            // Nested element - add to parent
                            let parent = &mut element_stack.last_mut().unwrap().1;

                            // Special handling for FHIR polymorphic properties
                            if self.is_fhir_polymorphic_property(&element_name) {
                                let (base_name, type_name) = self.extract_polymorphic_parts(&element_name);

                                // Create a new object with the type information
                                let mut typed_obj = serde_json::Map::new();

                                // If current_value is an object, extract its properties
                                if let Value::Object(obj) = current_value {
                                    for (k, v) in obj {
                                        typed_obj.insert(k, v);
                                    }
                                } else {
                                    // If it's not an object, use it as is
                                    typed_obj.insert("value".to_string(), current_value);
                                }

                                // Add type information
                                typed_obj.insert("type".to_string(), Value::String(type_name));

                                // Add to the parent with the base name
                                self.add_to_object(parent, base_name, Value::Object(typed_obj));
                            } else {
                                // Regular property
                                self.add_to_object(parent, element_name, current_value);
                            }
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    event_count += 1;
                    let element_name = String::from_utf8(e.name().as_ref().to_vec())?;
                    let mut current_obj = serde_json::Map::new();

                    // Handle attributes for self-closing elements
                    for attr in e.attributes() {
                        let attr = attr?;
                        let attr_name = String::from_utf8(attr.key.as_ref().to_vec())?;
                        let attr_value = String::from_utf8(attr.value.to_vec())?;

                        // Skip xmlns attributes as they're not part of FHIR JSON
                        if attr_name.starts_with("xmlns") {
                            continue;
                        }

                        current_obj.insert(attr_name, Value::String(attr_value));
                    }

                    // Determine the final value for this self-closing element
                    let current_value = if current_obj.len() == 1 && current_obj.contains_key("value") {
                        // For FHIR elements with only a "value" attribute, use the value directly
                        current_obj.get("value").unwrap().clone()
                    } else if current_obj.is_empty() {
                        // For elements with no attributes, create an empty object
                        Value::Object(current_obj)
                    } else {
                        Value::Object(current_obj)
                    };

                    // Add to parent or root
                    if element_stack.is_empty() {
                        // Direct child of root - add to main object
                        self.add_to_object(&mut json_obj, element_name, current_value);
                    } else {
                        // Nested element - add to parent
                        let parent = &mut element_stack.last_mut().unwrap().1;
                        self.add_to_object(parent, element_name, current_value);
                    }
                }
                Ok(Event::Text(e)) => {
                    if let Some((_element_name, _current_obj, text_content)) = element_stack.last_mut() {
                        let text = e.unescape()?.into_owned();
                        if !text.trim().is_empty() {
                            // Accumulate text content (in case there are multiple text nodes)
                            if let Some(existing_text) = text_content {
                                existing_text.push_str(&text);
                            } else {
                                *text_content = Some(text);
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(format!("XML parsing error: {:?}", e).into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(Value::Object(json_obj))
    }

    /// Helper function to add a value to an object, handling arrays properly
    fn add_to_object(&self, obj: &mut serde_json::Map<String, Value>, key: String, value: Value) {
        if let Some(existing) = obj.get_mut(&key) {
            match existing {
                Value::Array(arr) => {
                    arr.push(value);
                }
                _ => {
                    let old_value = existing.clone();
                    *existing = Value::Array(vec![old_value, value]);
                }
            }
        } else {
            obj.insert(key, value);
        }
    }

    /// Run a single test case and return results.
    fn run_single_test(&self, test_case: &TestCase, test_data: &FhirResource) -> TestResult {
        let start_time = Instant::now();
        let is_invalid_test = test_case.invalid.unwrap_or(false);

        let (status, actual, error) = match self.evaluate_expression(&test_case.expression, test_data) {
            Ok(result) => {
                if is_invalid_test {
                    // Invalid test should have failed but didn't - this is a failure
                    ("failed".to_string(), None, Some("Expected error but expression succeeded".to_string()))
                } else {
                    let actual_values = self.fhirpath_value_to_json_array(&result);
                    ("passed".to_string(), Some(actual_values), None)
                }
            }
            Err(e) => {
                if is_invalid_test {
                    // Invalid test correctly produced an error - this is a pass
                    ("passed".to_string(), None, None)
                } else {
                    ("error".to_string(), None, Some(e.to_string()))
                }
            }
        };

        let execution_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        let expected = test_case
            .expected_output
            .as_ref()
            .map(|outputs| {
                outputs
                    .iter()
                    .map(|output| output.value.clone())
                    .collect()
            })
            .unwrap_or_default();

        TestResult {
            name: test_case.name.clone(),
            description: test_case.description.clone(),
            expression: test_case.expression.clone(),
            status,
            execution_time_ms,
            expected,
            actual,
            error,
        }
    }

    /// Evaluate FHIRPath expression using fhirpath-core.
    fn evaluate_expression(&self, expression: &str, resource: &FhirResource) -> Result<FhirPathValue, Box<dyn std::error::Error>> {
        // Convert FhirResource to serde_json::Value
        let json_value = serde_json::to_value(resource)?;
        evaluate_expression(expression, json_value).map_err(|e| e.into())
    }

    /// Convert FhirPathValue to JSON array for standardized output.
    fn fhirpath_value_to_json_array(&self, value: &FhirPathValue) -> Vec<Value> {
        match value {
            FhirPathValue::Collection(items) => {
                items.iter().map(|item| self.fhirpath_value_to_json(item)).collect()
            }
            _ => vec![self.fhirpath_value_to_json(value)],
        }
    }

    /// Convert single FhirPathValue to JSON.
    fn fhirpath_value_to_json(&self, value: &FhirPathValue) -> Value {
        match value {
            FhirPathValue::String(s) => json!(s),
            FhirPathValue::Integer(i) => json!(i),
            FhirPathValue::Decimal(d) => json!(d),
            FhirPathValue::Boolean(b) => json!(b),
            FhirPathValue::Date(d) => json!(d),
            FhirPathValue::DateTime(dt) => json!(dt),
            FhirPathValue::Time(t) => json!(t),
            FhirPathValue::Collection(items) => {
                json!(items.iter().map(|item| self.fhirpath_value_to_json(item)).collect::<Vec<_>>())
            }
            FhirPathValue::Empty => json!(null),
            _ => json!("unknown"), // Fallback for unsupported types
        }
    }

    /// Load official FHIRPath test cases from XML file.
    fn load_official_tests(&self) -> Result<Vec<TestCase>, Box<dyn std::error::Error>> {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        #[derive(Debug)]
        struct XmlTestOutput {
            output_type: String,
            value: String,
        }

        #[derive(Debug)]
        struct XmlTestExpression {
            invalid: Option<String>,
            value: String,
        }

        #[derive(Debug)]
        struct XmlOfficialTest {
            name: String,
            description: Option<String>,
            input_file: String,
            predicate: Option<String>,
            mode: Option<String>,
            expression: XmlTestExpression,
            outputs: Vec<XmlTestOutput>,
        }

        #[derive(Debug)]
        struct XmlTestGroup {
            name: String,
            description: Option<String>,
            tests: Vec<XmlOfficialTest>,
        }

        let xml_path = Path::new(&self.test_cases_dir).join("tests-fhir-r4.xml");
        let mut xml_content = fs::read_to_string(&xml_path)?;

        // Fix malformed XML: replace </o> with </output>
        xml_content = xml_content.replace("</o>", "</output>");

        let mut reader = Reader::from_str(&xml_content);
        reader.trim_text(true);

        let mut buf = Vec::new();
        let mut test_cases = Vec::new();
        let mut current_group: Option<XmlTestGroup> = None;
        let mut current_test: Option<XmlOfficialTest> = None;
        let mut current_expression: Option<XmlTestExpression> = None;
        let mut current_output: Option<XmlTestOutput> = None;
        let mut text_content = String::new();
        let mut in_expression = false;
        let mut in_output = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"group" => {
                            let mut group_name = String::new();
                            let mut group_description = None;

                            for attr in e.attributes() {
                                let attr = attr?;
                                match attr.key.as_ref() {
                                    b"name" => group_name = String::from_utf8(attr.value.to_vec())?,
                                    b"description" => group_description = Some(String::from_utf8(attr.value.to_vec())?),
                                    _ => {}
                                }
                            }

                            current_group = Some(XmlTestGroup {
                                name: group_name,
                                description: group_description,
                                tests: Vec::new(),
                            });
                        }
                        b"test" => {
                            let mut test_name = String::new();
                            let mut test_description = None;
                            let mut input_file = String::new();
                            let mut predicate = None;
                            let mut mode = None;

                            for attr in e.attributes() {
                                let attr = attr?;
                                match attr.key.as_ref() {
                                    b"name" => test_name = String::from_utf8(attr.value.to_vec())?,
                                    b"description" => test_description = Some(String::from_utf8(attr.value.to_vec())?),
                                    b"inputfile" => input_file = String::from_utf8(attr.value.to_vec())?,
                                    b"predicate" => predicate = Some(String::from_utf8(attr.value.to_vec())?),
                                    b"mode" => mode = Some(String::from_utf8(attr.value.to_vec())?),
                                    _ => {}
                                }
                            }

                            current_test = Some(XmlOfficialTest {
                                name: test_name,
                                description: test_description,
                                input_file,
                                predicate,
                                mode,
                                expression: XmlTestExpression { invalid: None, value: String::new() },
                                outputs: Vec::new(),
                            });
                        }
                        b"expression" => {
                            let mut invalid = None;

                            for attr in e.attributes() {
                                let attr = attr?;
                                if attr.key.as_ref() == b"invalid" {
                                    invalid = Some(String::from_utf8(attr.value.to_vec())?);
                                }
                            }

                            current_expression = Some(XmlTestExpression {
                                invalid,
                                value: String::new(),
                            });
                            in_expression = true;
                            text_content.clear();
                        }
                        b"output" => {
                            let mut output_type = String::new();

                            for attr in e.attributes() {
                                let attr = attr?;
                                if attr.key.as_ref() == b"type" {
                                    output_type = String::from_utf8(attr.value.to_vec())?;
                                }
                            }

                            current_output = Some(XmlTestOutput {
                                output_type,
                                value: String::new(),
                            });
                            in_output = true;
                            text_content.clear();
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_expression || in_output {
                        let text = e.unescape()?.into_owned().trim().to_string();
                        if !text.is_empty() {
                            text_content = text;
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    match e.name().as_ref() {
                        b"expression" => {
                            if let Some(ref mut expr) = current_expression {
                                expr.value = text_content.clone();
                            }
                            if let Some(ref mut test) = current_test {
                                if let Some(expr) = current_expression.take() {
                                    test.expression = expr;
                                }
                            }
                            in_expression = false;
                            text_content.clear();
                        }
                        b"output" => {
                            if let Some(ref mut output) = current_output {
                                output.value = text_content.clone();
                            }
                            if let Some(ref mut test) = current_test {
                                if let Some(output) = current_output.take() {
                                    test.outputs.push(output);
                                }
                            }
                            in_output = false;
                            text_content.clear();
                        }
                        b"test" => {
                            if let Some(test) = current_test.take() {
                                if let Some(ref mut group) = current_group {
                                    group.tests.push(test);
                                }
                            }
                        }
                        b"group" => {
                            if let Some(group) = current_group.take() {
                                // Process all tests in this group
                                for test in group.tests {
                                    let expected_output = test.outputs.iter().map(|output| {
                                        let value = match output.output_type.as_str() {
                                            "boolean" => {
                                                if output.value == "true" {
                                                    json!(true)
                                                } else {
                                                    json!(false)
                                                }
                                            }
                                            "integer" => {
                                                json!(output.value)
                                            }
                                            "decimal" => {
                                                json!(output.value)
                                            }
                                            _ => json!(output.value),
                                        };

                                        ExpectedOutput {
                                            output_type: output.output_type.clone(),
                                            value,
                                        }
                                    }).collect();

                                    let invalid = test.expression.invalid.is_some();

                                    test_cases.push(TestCase {
                                        name: test.name,
                                        description: test.description.unwrap_or_default(),
                                        input_file: test.input_file,
                                        expression: test.expression.value,
                                        expected_output: Some(expected_output),
                                        invalid: Some(invalid),
                                        group: Some(group.name.clone()),
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(format!("XML parsing error: {:?}", e).into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(test_cases)
    }

    /// Run all tests and return results.
    pub fn run_tests(&self) -> Result<TestResults, Box<dyn std::error::Error>> {
        println!("🧪 Running Rust FHIRPath tests...");

        let mut results = TestResults {
            language: "rust".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64(),
            tests: Vec::new(),
            summary: TestSummary {
                total: 0,
                passed: 0,
                failed: 0,
                errors: 0,
            },
        };

        // Load test data files
        let mut test_data_cache = HashMap::new();
        for input_file in &self.test_config.test_data.input_files {
            if let Some(test_data) = self.load_test_data(input_file) {
                test_data_cache.insert(input_file.clone(), test_data);
            }
        }

        // Load and run official tests
        println!("📋 Loading official FHIRPath test suite...");
        let official_tests = self.load_official_tests()?;
        println!("📊 Found {} official test cases", official_tests.len());

        for test_case in official_tests {
            if let Some(test_data) = test_data_cache.get(&test_case.input_file) {
                let test_result = self.run_single_test(&test_case, test_data);

                results.summary.total += 1;
                match test_result.status.as_str() {
                    "passed" => results.summary.passed += 1,
                    "error" => results.summary.errors += 1,
                    _ => results.summary.failed += 1,
                }

                let status_icon = if test_result.status == "passed" {
                    "✅"
                } else if test_result.status == "error" {
                    "💥"
                } else {
                    "❌"
                };
                println!("  {} {} ({:.2}ms) [{}]", status_icon, test_result.name, test_result.execution_time_ms, test_case.group.as_deref().unwrap_or("unknown"));

                results.tests.push(test_result);
            } else {
                println!("⚠️  Skipping test {} - test data not available: {}", test_case.name, test_case.input_file);
            }
        }

        // Save results
        let results_file = Path::new(&self.results_dir).join("rust_test_results.json");
        let results_json = serde_json::to_string_pretty(&results)?;
        fs::write(&results_file, results_json)?;

        println!("📊 Results saved to: {}", results_file.display());
        println!("📈 Summary: {}/{} tests passed", results.summary.passed, results.summary.total);

        Ok(results)
    }

    /// Run benchmarks and return results.
    pub fn run_benchmarks(&self) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        println!("⚡ Running Rust FHIRPath benchmarks...");

        let mut results = BenchmarkResults {
            language: "rust".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64(),
            benchmarks: Vec::new(),
            system_info: SystemInfo {
                platform: std::env::consts::OS.to_string(),
                rust_version: "1.70+".to_string(), // Static version for compatibility
                fhirpath_version: "fhirpath-core".to_string(),
            },
        };

        // Load test data
        let mut test_data_cache = HashMap::new();
        for input_file in &self.test_config.test_data.input_files {
            if let Some(test_data) = self.load_test_data(input_file) {
                test_data_cache.insert(input_file.clone(), test_data);
            }
        }

        // Run benchmarks
        for benchmark in &self.test_config.benchmark_tests {
            let default_file = "patient-example.xml".to_string();
            let input_file = benchmark.input_file.as_ref().unwrap_or(&default_file);

            if let Some(test_data) = test_data_cache.get(input_file) {
                println!("  🏃 Running {}...", benchmark.name);

                let iterations = benchmark.iterations.unwrap_or(1000);
                let mut times = Vec::new();

                // Warm up
                for _ in 0..10 {
                    let _ = self.evaluate_expression(&benchmark.expression, test_data);
                }

                // Actual benchmark
                for _ in 0..iterations {
                    let start_time = Instant::now();
                    let _ = self.evaluate_expression(&benchmark.expression, test_data);
                    let elapsed = start_time.elapsed().as_secs_f64() * 1000.0; // Convert to milliseconds
                    times.push(elapsed);
                }

                if !times.is_empty() {
                    let avg_time = times.iter().sum::<f64>() / times.len() as f64;
                    let min_time = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let max_time = times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                    let ops_per_second = if avg_time > 0.0 { 1000.0 / avg_time } else { 0.0 };

                    let benchmark_result = BenchmarkResult {
                        name: benchmark.name.clone(),
                        description: benchmark.description.clone(),
                        expression: benchmark.expression.clone(),
                        iterations,
                        avg_time_ms: avg_time,
                        min_time_ms: min_time,
                        max_time_ms: max_time,
                        ops_per_second,
                    };

                    results.benchmarks.push(benchmark_result);
                    println!("    ⏱️  {:.2}ms avg ({:.1} ops/sec)", avg_time, ops_per_second);
                }
            } else {
                println!("⚠️  Skipping benchmark {} - test data not available", benchmark.name);
            }
        }

        // Save results
        let results_file = Path::new(&self.results_dir).join("rust_benchmark_results.json");
        let results_json = serde_json::to_string_pretty(&results)?;
        fs::write(&results_file, results_json)?;

        println!("📊 Benchmark results saved to: {}", results_file.display());

        Ok(results)
    }
}

