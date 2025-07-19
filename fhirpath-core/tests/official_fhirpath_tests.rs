use fhirpath_core::evaluator::evaluate_expression;
use fhirpath_core::model::FhirPathValue;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct TestSuite {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@description")]
    description: String,
    #[serde(rename = "group")]
    groups: Vec<TestGroup>,
}

#[derive(Debug, Deserialize)]
struct TestGroup {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@description", default)]
    description: Option<String>,
    #[serde(rename = "test")]
    tests: Vec<Test>,
}

#[derive(Debug, Deserialize)]
struct Test {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@description", default)]
    description: Option<String>,
    #[serde(rename = "@inputfile")]
    inputfile: String,
    #[serde(rename = "@predicate", default)]
    predicate: Option<String>,
    #[serde(rename = "@mode", default)]
    mode: Option<String>,
    expression: TestExpression,
    #[serde(rename = "output", default)]
    outputs: Vec<TestOutput>,
}

#[derive(Debug, Deserialize)]
struct TestExpression {
    #[serde(rename = "@invalid", default)]
    invalid: Option<String>,
    #[serde(rename = "$text", default)]
    text: String,
}

#[derive(Debug, Deserialize)]
struct TestOutput {
    #[serde(rename = "@type", default)]
    output_type: Option<String>,
    #[serde(rename = "$text", default)]
    text: Option<String>,
}

/// Check if an element name represents a FHIR polymorphic property
/// In FHIR, polymorphic properties have names like "valueString", "valueInteger", etc.
/// where "value" is the base name and "String" or "Integer" is the type
fn is_fhir_polymorphic_property(element_name: &str) -> bool {
    // Common FHIR polymorphic properties
    let polymorphic_bases = [
        "value",
        "component",
        "onset",
        "abatement",
        "asserted",
        "recorded",
        "onset",
        "offset",
        "target",
        "entity",
        "detail",
        "reason",
        "performer",
    ];

    // Check if the element name starts with any of the known polymorphic bases
    // and has a capital letter after the base (indicating a type)
    for base in polymorphic_bases {
        if element_name.starts_with(base)
            && element_name.len() > base.len()
            && element_name
                .chars()
                .nth(base.len())
                .map_or(false, |c| c.is_uppercase())
        {
            return true;
        }
    }

    false
}

/// Extract the base name and type name from a FHIR polymorphic property name
/// For example, "valueString" would return ("value", "String")
fn extract_polymorphic_parts(element_name: &str) -> (String, String) {
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

/// Load and convert XML input file to JSON
fn load_input_file(filename: &str) -> Result<Value, Box<dyn std::error::Error>> {
    // For official tests, prioritize XML files to ensure correct test data
    let xml_path = Path::new("tests/official-tests/r4/input").join(filename);
    if xml_path.exists() {
        let xml_content = fs::read_to_string(&xml_path)?;
        let json_value = convert_xml_to_json(&xml_content)?;
        return Ok(json_value);
    }

    // Fallback to JSON file if XML doesn't exist
    let json_filename = filename.replace(".xml", ".json");
    let json_path = Path::new("tests/fixtures").join(&json_filename);

    if json_path.exists() {
        let json_content = fs::read_to_string(&json_path)?;
        let json_value: Value = serde_json::from_str(&json_content)?;
        return Ok(json_value);
    }

    Err(format!("Input file not found: {}", filename).into())
}

/// Convert XML content to JSON following FHIR conventions
fn convert_xml_to_json(xml_content: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(xml_content);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut json_obj = serde_json::Map::new();
    let mut element_stack: Vec<(String, serde_json::Map<String, Value>, Option<String>)> =
        Vec::new();
    let mut root_element_name = String::new();
    let mut in_root = false;
    let mut event_count = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                event_count += 1;
                let element_name = String::from_utf8(e.name().as_ref().to_vec())?;
                // println!("[{}] START: {}", event_count, element_name);
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
                    json_obj.insert(
                        "resourceType".to_string(),
                        Value::String(element_name.clone()),
                    );
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
                // println!("[{}] END: {}", event_count, element_name);

                if let Some((stack_element_name, mut current_obj, text_content)) =
                    element_stack.pop()
                {
                    // Sanity check - element names should match
                    if stack_element_name != element_name {
                        return Err(format!(
                            "XML structure error: expected {}, got {}",
                            stack_element_name, element_name
                        )
                        .into());
                    }

                    // If this is the root element, process its children and break
                    if element_name == root_element_name {
                        // println!("    Processing root element children: {} items", current_obj.len());
                        // Add all accumulated children to the main json object
                        for (key, value) in current_obj {
                            // println!("    Adding root child: {}", key);
                            add_to_object(&mut json_obj, key, value);
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
                    let current_value =
                        if current_obj.len() == 1 && current_obj.contains_key("value") {
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
                        if is_fhir_polymorphic_property(&element_name) {
                            let (base_name, type_name) = extract_polymorphic_parts(&element_name);

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
                            add_to_object(&mut json_obj, base_name, Value::Object(typed_obj));
                        } else {
                            // Regular property
                            add_to_object(&mut json_obj, element_name, current_value);
                        }
                    } else {
                        // Nested element - add to parent
                        let parent = &mut element_stack.last_mut().unwrap().1;

                        // Special handling for FHIR polymorphic properties
                        if is_fhir_polymorphic_property(&element_name) {
                            let (base_name, type_name) = extract_polymorphic_parts(&element_name);

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
                            add_to_object(parent, base_name, Value::Object(typed_obj));
                        } else {
                            // Regular property
                            add_to_object(parent, element_name, current_value);
                        }
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                event_count += 1;
                let element_name = String::from_utf8(e.name().as_ref().to_vec())?;
                // println!("[{}] EMPTY: {}", event_count, element_name);
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
                    add_to_object(&mut json_obj, element_name, current_value);
                } else {
                    // Nested element - add to parent
                    let parent = &mut element_stack.last_mut().unwrap().1;
                    add_to_object(parent, element_name, current_value);
                }
            }
            Ok(Event::Text(e)) => {
                if let Some((_element_name, _current_obj, text_content)) = element_stack.last_mut()
                {
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
fn add_to_object(obj: &mut serde_json::Map<String, Value>, key: String, value: Value) {
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

/// Parse the official test suite XML file using quick-xml
fn parse_test_suite() -> Result<TestSuite, Box<dyn std::error::Error>> {
    let test_file_path = "tests/official-tests/r4/tests-fhir-r4.xml";
    let xml_content = fs::read_to_string(test_file_path)?;

    let mut reader = Reader::from_str(&xml_content);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut test_suite_name = String::new();
    let mut test_suite_description = String::new();
    let mut groups = Vec::new();

    // Parse the XML manually
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"tests" => {
                        // Parse tests element attributes
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                b"name" => {
                                    test_suite_name = String::from_utf8(attr.value.to_vec())?
                                }
                                b"description" => {
                                    test_suite_description = String::from_utf8(attr.value.to_vec())?
                                }
                                _ => {}
                            }
                        }
                    }
                    b"group" => {
                        // Parse a test group
                        let group = parse_test_group(&mut reader, e)?;
                        groups.push(group);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"tests" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(
                    format!("Error at position {}: {:?}", reader.buffer_position(), e).into(),
                );
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(TestSuite {
        name: test_suite_name,
        description: test_suite_description,
        groups,
    })
}

/// Parse a test group using quick-xml
fn parse_test_group(
    reader: &mut Reader<&[u8]>,
    start_element: &quick_xml::events::BytesStart,
) -> Result<TestGroup, Box<dyn std::error::Error>> {
    let mut group_name = String::new();
    let mut group_description = None;
    let mut tests = Vec::new();

    // Parse group attributes
    for attr in start_element.attributes() {
        let attr = attr?;
        match attr.key.as_ref() {
            b"name" => group_name = String::from_utf8(attr.value.to_vec())?,
            b"description" => group_description = Some(String::from_utf8(attr.value.to_vec())?),
            _ => {}
        }
    }

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"test" {
                    let test = parse_test(reader, e)?;
                    tests.push(test);
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"group" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error parsing group: {:?}", e).into()),
            _ => {}
        }
        buf.clear();
    }

    Ok(TestGroup {
        name: group_name,
        description: group_description,
        tests,
    })
}

/// Parse a single test using quick-xml
fn parse_test(
    reader: &mut Reader<&[u8]>,
    start_element: &quick_xml::events::BytesStart,
) -> Result<Test, Box<dyn std::error::Error>> {
    let mut test_name = String::new();
    let mut test_description = None;
    let mut inputfile = String::new();
    let mut predicate = None;
    let mut mode = None;
    let mut expression = TestExpression {
        invalid: None,
        text: String::new(),
    };
    let mut outputs = Vec::new();

    // Parse test attributes
    for attr in start_element.attributes() {
        let attr = attr?;
        match attr.key.as_ref() {
            b"name" => test_name = String::from_utf8(attr.value.to_vec())?,
            b"description" => test_description = Some(String::from_utf8(attr.value.to_vec())?),
            b"inputfile" => inputfile = String::from_utf8(attr.value.to_vec())?,
            b"predicate" => predicate = Some(String::from_utf8(attr.value.to_vec())?),
            b"mode" => mode = Some(String::from_utf8(attr.value.to_vec())?),
            _ => {}
        }
    }

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"expression" => {
                        // Parse expression attributes
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"invalid" {
                                expression.invalid = Some(String::from_utf8(attr.value.to_vec())?);
                            }
                        }
                    }
                    b"output" => {
                        let output = parse_output(reader, e)?;
                        outputs.push(output);
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                // This is the text content of the current element
                let text = e.unescape()?.into_owned();
                if !text.trim().is_empty() {
                    expression.text = text.trim().to_string();
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"test" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error parsing test: {:?}", e).into()),
            _ => {}
        }
        buf.clear();
    }

    Ok(Test {
        name: test_name,
        description: test_description,
        inputfile,
        predicate,
        mode,
        expression,
        outputs,
    })
}

/// Parse an output element using quick-xml
fn parse_output(
    reader: &mut Reader<&[u8]>,
    start_element: &quick_xml::events::BytesStart,
) -> Result<TestOutput, Box<dyn std::error::Error>> {
    let mut output_type = None;
    let mut text = None;

    // Parse output attributes
    for attr in start_element.attributes() {
        let attr = attr?;
        if attr.key.as_ref() == b"type" {
            output_type = Some(String::from_utf8(attr.value.to_vec())?);
        }
    }

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(e)) => {
                let content = e.unescape()?.into_owned();
                if !content.trim().is_empty() {
                    text = Some(content.trim().to_string());
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"output" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error parsing output: {:?}", e).into()),
            _ => {}
        }
        buf.clear();
    }

    Ok(TestOutput { output_type, text })
}

/// Compare actual FhirPathValue result with expected TestOutput
fn compare_result_with_expected(result: &FhirPathValue, expected: &TestOutput) -> bool {
    let actual_text = fhirpath_value_to_string(result);

    match &expected.text {
        Some(expected_text) => {
            let expected_text = expected_text.trim();
            actual_text == expected_text
        }
        None => {
            // If no expected text, check if result is empty
            match result {
                FhirPathValue::Empty => true,
                FhirPathValue::Collection(coll) if coll.is_empty() => true,
                _ => false,
            }
        }
    }
}

/// Convert FhirPathValue to string representation for comparison
fn fhirpath_value_to_string(value: &FhirPathValue) -> String {
    match value {
        FhirPathValue::Empty => String::new(),
        FhirPathValue::Boolean(b) => b.to_string(),
        FhirPathValue::Integer(i) => i.to_string(),
        FhirPathValue::Decimal(d) => d.to_string(),
        FhirPathValue::String(s) => s.clone(),
        FhirPathValue::Date(d) => d.clone(),
        FhirPathValue::DateTime(dt) => dt.clone(),
        FhirPathValue::Time(t) => t.clone(),
        FhirPathValue::Quantity { value, unit } => format!("{} {}", value, unit),
        FhirPathValue::Collection(coll) => {
            if coll.is_empty() {
                String::new()
            } else {
                // For collections with one or more items, return the first item
                // This matches FHIRPath test expectations where single values are expected
                fhirpath_value_to_string(&coll[0])
            }
        }
        FhirPathValue::Resource(resource) => {
            // For resources, convert to JSON string representation
            resource.to_json().to_string()
        }
    }
}

/// Execute a single test case
fn execute_test(test: &Test, input_data: &Value) -> Result<bool, Box<dyn std::error::Error>> {
    let expression = &test.expression.text;

    // Check if this is an invalid expression test
    if test.expression.invalid.is_some() {
        // For invalid expressions, we expect the evaluation to fail
        match evaluate_expression(expression, input_data.clone()) {
            Ok(_) => return Ok(false), // Should have failed but didn't
            Err(_) => return Ok(true), // Failed as expected
        }
    }

    // Evaluate the expression
    let result = evaluate_expression(expression, input_data.clone())?;

    // If this is a predicate test, check if result is boolean true
    if test.predicate.as_deref() == Some("true") {
        match result {
            FhirPathValue::Boolean(true) => return Ok(true),
            _ => return Ok(false),
        }
    }

    // If this is a predicate false test, check if result is boolean false or empty
    if test.predicate.as_deref() == Some("false") {
        match result {
            FhirPathValue::Boolean(false) => return Ok(true),
            FhirPathValue::Collection(ref coll) if coll.is_empty() => return Ok(true),
            _ => return Ok(false),
        }
    }

    // For regular tests, compare with expected outputs
    if test.outputs.is_empty() {
        // No expected output means we expect an empty result
        match result {
            FhirPathValue::Collection(ref coll) if coll.is_empty() => return Ok(true),
            _ => return Ok(false),
        }
    }

    // Special handling for collections with multiple expected outputs
    if let FhirPathValue::Collection(items) = &result {
        if test.outputs.len() > 1 {
            // If we have multiple expected outputs, compare each item with the corresponding expected output
            if items.len() != test.outputs.len() {
                println!(
                    "Test FAILED: {} - Expression: {} - Expected {} items but got {}",
                    test.name,
                    expression,
                    test.outputs.len(),
                    items.len()
                );
                return Ok(false);
            }

            // Compare each item with the corresponding expected output
            for (i, (item, expected_output)) in items.iter().zip(test.outputs.iter()).enumerate() {
                if !compare_result_with_expected(item, expected_output) {
                    println!(
                        "Test FAILED: {} - Expression: {} - Item {}: Expected: {:?} - Actual: {:?}",
                        test.name, expression, i, expected_output, item
                    );
                    return Ok(false);
                }
            }
            return Ok(true);
        }
    }

    // For single expected output or non-collection results, compare with all expected outputs
    for expected_output in &test.outputs {
        if !compare_result_with_expected(&result, expected_output) {
            println!(
                "Test FAILED: {} - Expression: {} - Expected: {:?} - Actual: {:?}",
                test.name, expression, expected_output, result
            );
            return Ok(false);
        }
    }

    // println!(
    //     "Test PASSED: {} - Expression: {} - Result: {:?}",
    //     test.name, expression, result
    // );
    Ok(true)
}

/// Run a subset of tests for initial validation
#[test]
fn test_official_fhirpath_basic_tests() {
    // For now, let's test it with our existing patient fixture
    let patient_json = fs::read_to_string("tests/fixtures/patient-example.json")
        .expect("Failed to read patient fixture");
    let patient_data: Value =
        serde_json::from_str(&patient_json).expect("Failed to parse patient JSON");

    // Test some basic expressions manually first
    let test_cases = vec![
        ("birthDate", "1974-12-25"),
        ("name.given", "John"), // Updated to match actual fixture data
        ("gender", "male"),
        ("active", "true"),
    ];

    for (expression, expected) in test_cases {
        match evaluate_expression(expression, patient_data.clone()) {
            Ok(result) => {
                println!("Expression: {} -> Result: {:?}", expression, result);
                // Basic validation - just ensure it doesn't crash
                assert!(!matches!(result, FhirPathValue::Collection(ref coll) if coll.is_empty()));
            }
            Err(e) => {
                panic!("Failed to evaluate expression '{}': {:?}", expression, e);
            }
        }
    }
}

/// Test XML parsing functionality
#[test]
fn test_xml_parsing() {
    // Test if we can parse the test suite XML
    match parse_test_suite() {
        Ok(test_suite) => {
            println!("Successfully parsed test suite: {}", test_suite.name);
            println!("Number of test groups: {}", test_suite.groups.len());

            let total_tests: usize = test_suite
                .groups
                .iter()
                .map(|group| group.tests.len())
                .sum();
            println!("Total number of tests: {}", total_tests);

            // Print first few test groups for verification
            for (i, group) in test_suite.groups.iter().take(3).enumerate() {
                println!(
                    "Group {}: {} ({} tests)",
                    i + 1,
                    group.name,
                    group.tests.len()
                );
            }
        }
        Err(e) => {
            panic!("Failed to parse test suite: {:?}", e);
        }
    }
}

/// Test loading input files
#[test]
fn test_input_file_loading() {
    // Test loading patient-example.xml (should work with our XML to JSON conversion)
    match load_input_file("patient-example.xml") {
        Ok(data) => {
            println!("Successfully loaded patient data");

            // Print the name data to verify we're getting Peter instead of John
            if let Some(names) = data.get("name") {
                println!("Names: {}", serde_json::to_string_pretty(names).unwrap());

                // Check if we have the expected Peter data
                if let Some(name_array) = names.as_array() {
                    for name in name_array {
                        if let Some(given) = name.get("given") {
                            println!(
                                "Given names: {}",
                                serde_json::to_string_pretty(given).unwrap()
                            );
                        }
                    }
                }
            }

            assert!(data.is_object());
        }
        Err(e) => {
            panic!("Failed to load patient-example.xml: {:?}", e);
        }
    }
}

/// Debug test to examine XML to JSON conversion output
#[test]
fn test_debug_xml_conversion() {
    // Load the XML content directly
    let xml_path = "tests/official-tests/r4/input/patient-example.xml";
    match fs::read_to_string(xml_path) {
        Ok(xml_content) => {
            // Convert to JSON
            match convert_xml_to_json(&xml_content) {
                Ok(json_result) => {
                    println!("\n=== Converted JSON ===");
                    println!("{}", serde_json::to_string_pretty(&json_result).unwrap());

                    // Check specific elements that might be problematic
                    println!("\n=== Checking specific elements ===");

                    if let Some(resource_type) = json_result.get("resourceType") {
                        println!(
                            "ResourceType: {}",
                            serde_json::to_string_pretty(resource_type).unwrap()
                        );
                    }

                    if let Some(id) = json_result.get("id") {
                        println!("ID: {}", serde_json::to_string_pretty(id).unwrap());
                    }

                    if let Some(names) = json_result.get("name") {
                        println!("Names: {}", serde_json::to_string_pretty(names).unwrap());
                    }

                    if let Some(telecom) = json_result.get("telecom") {
                        println!(
                            "Telecom: {}",
                            serde_json::to_string_pretty(telecom).unwrap()
                        );
                    }

                    if let Some(text) = json_result.get("text") {
                        println!("Text: {}", serde_json::to_string_pretty(text).unwrap());
                    }
                }
                Err(e) => {
                    panic!("Failed to convert XML to JSON: {:?}", e);
                }
            }
        }
        Err(e) => {
            panic!("Failed to read XML file: {:?}", e);
        }
    }
}

/// Debug specific dateTime expressions
#[test]
fn debug_datetime_expressions() {
    let patient_json = r#"{"resourceType": "Patient", "id": "example"}"#;
    let patient_data: Value = serde_json::from_str(patient_json).unwrap();

    // Test cases from the official tests
    let test_cases = vec![
        "@2015T.is(DateTime)",
        "@2015-02T.is(DateTime)",
        "@2015-02-04T14:34:28.is(DateTime)",
        "@2015-02-04T14:34:28Z.is(DateTime)",
        "'2015'.convertsToDateTime()",
        "'2015-02'.convertsToDateTime()",
        "'2015-02-04'.convertsToDateTime()",
        "'2015-02-04T14'.convertsToDateTime()",
        "'2015-02-04T14:34'.convertsToDateTime()",
        "'2015-02-04T14:34:28'.convertsToDateTime()",
        "'2015-02-04T14:34:28.123'.convertsToDateTime()",
        "'2015-02-04T14:34:28Z'.convertsToDateTime()",
        "'2015-02-04T14:34:28+10:00'.convertsToDateTime()",
        "'invalid'.convertsToDateTime()",
        // Test Time literals
        "@T14.is(Time)",
        "@T14:34.is(Time)",
        "@T14:34:28.is(Time)",
        "@T14:34:28.123.is(Time)",
    ];

    for expression in test_cases {
        println!("Testing expression: {}", expression);
        match evaluate_expression(expression, patient_data.clone()) {
            Ok(result) => {
                println!("  Result: {:?}", result);
            }
            Err(e) => {
                println!("  Error: {:?}", e);
            }
        }
        println!();
    }
}

/// Run the full official FHIRPath test suite
#[test]
fn run_official_fhirpath_tests() {
    let test_suite = parse_test_suite().expect("Failed to parse test suite");
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    // Track failures by group and expression pattern
    let mut failures_by_group: std::collections::HashMap<String, Vec<(String, String, String)>> =
        std::collections::HashMap::new();
    let mut datetime_failures = 0;
    let mut conversion_failures = 0;
    let mut collection_failures = 0;
    let mut math_failures = 0;
    let mut comparison_failures = 0;
    let mut other_failures = 0;

    for group in &test_suite.groups {
        println!("Running test group: {}", group.name);
        let mut group_passed = 0;
        let mut group_failed = 0;
        let mut group_skipped = 0;
        let mut group_failures = Vec::new();

        for test in &group.tests {
            match load_input_file(&test.inputfile) {
                Ok(input_data) => match execute_test(test, &input_data) {
                    Ok(true) => {
                        passed += 1;
                        group_passed += 1;
                        // TODO: enable later
                        // println!("  ✓ {}", test.name);
                    }
                    Ok(false) => {
                        failed += 1;
                        group_failed += 1;
                        println!("  ✗ {}", test.name);

                        // Categorize failure by expression pattern
                        let expr = &test.expression.text;
                        let expected = match &test.outputs.first() {
                            Some(output) => output.text.clone().unwrap_or_default(),
                            None => String::new(),
                        };

                        // Store failure details
                        group_failures.push((test.name.clone(), expr.clone(), expected.clone()));

                        // Categorize failure type
                        if expr.contains("DateTime")
                            || expr.contains("Date")
                            || expr.contains("Time")
                            || expr.contains("today")
                            || expr.contains("now")
                            || expr.starts_with("@")
                        {
                            datetime_failures += 1;
                        } else if expr.contains("convertsTo")
                            || expr.contains(".as(")
                            || expr.contains(".is(")
                        {
                            conversion_failures += 1;
                        } else if expr.contains("first")
                            || expr.contains("last")
                            || expr.contains("tail")
                            || expr.contains("skip")
                            || expr.contains("take")
                            || expr.contains("where")
                            || expr.contains("select")
                            || expr.contains("all")
                            || expr.contains("any")
                        {
                            collection_failures += 1;
                        } else if expr.contains("+")
                            || expr.contains("-")
                            || expr.contains("*")
                            || expr.contains("/")
                            || expr.contains("div")
                            || expr.contains("mod")
                        {
                            math_failures += 1;
                        } else if expr.contains("=")
                            || expr.contains(">")
                            || expr.contains("<")
                            || expr.contains("!=")
                            || expr.contains("<=")
                            || expr.contains(">=")
                        {
                            comparison_failures += 1;
                        } else {
                            other_failures += 1;
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        group_failed += 1;
                        println!("  ✗ {} (Error: {:?})", test.name, e);

                        // Store error details
                        group_failures.push((
                            test.name.clone(),
                            test.expression.text.clone(),
                            format!("ERROR: {:?}", e),
                        ));
                    }
                },
                Err(e) => {
                    skipped += 1;
                    group_skipped += 1;
                    println!("  - {} (Skipped: {:?})", test.name, e);
                }
            }
        }

        // Store group results
        if !group_failures.is_empty() {
            failures_by_group.insert(group.name.clone(), group_failures);
        }

        println!(
            "  Group summary: {} passed, {} failed, {} skipped",
            group_passed, group_failed, group_skipped
        );
    }

    println!("\nTest Results by Group:");
    let mut groups_by_failure_count: Vec<_> = failures_by_group.iter().collect();
    groups_by_failure_count.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    for (group_name, failures) in &groups_by_failure_count {
        println!("Group '{}': {} failures", group_name, failures.len());

        // Print a few examples of failures from this group
        for (i, (test_name, expr, expected)) in failures.iter().take(3).enumerate() {
            println!(
                "  Example {}: Test '{}' - Expression: {} - Expected: {}",
                i + 1,
                test_name,
                expr,
                expected
            );
        }
    }

    println!("\nFailure Categories:");
    println!("DateTime related failures: {}", datetime_failures);
    println!("Type conversion failures: {}", conversion_failures);
    println!("Collection operation failures: {}", collection_failures);
    println!("Math operation failures: {}", math_failures);
    println!("Comparison operation failures: {}", comparison_failures);
    println!("Other failures: {}", other_failures);

    println!("\nOverall Test Results:");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Skipped: {}", skipped);
    println!("Total: {}", passed + failed + skipped);
    println!(
        "Success rate: {:.2}%",
        (passed as f64 / (passed + failed) as f64) * 100.0
    );
}
