//! Hover documentation provider
//!
//! Provides rich hover information for:
//! - FHIRPath functions with full documentation
//! - FHIR properties with type information
//! - Keywords and operators
//! - Variables

use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Range};
use octofhir_fhirpath::evaluator::create_function_registry;

use crate::document::FhirPathDocument;

/// Generate hover information for the given document and position
pub fn generate_hover(document: &FhirPathDocument, position: Position) -> Option<Hover> {
    let offset = document.position_to_offset(position);

    // Find word at position
    let (word, word_range) = extract_word_at_offset(&document.text, offset, position)?;

    // Generate hover content based on word type
    let content = if let Some(func_hover) = get_function_hover(&word) {
        func_hover
    } else if let Some(prop_hover) = get_property_hover(&word) {
        prop_hover
    } else if let Some(kw_hover) = get_keyword_hover(&word) {
        kw_hover
    } else if let Some(var_hover) = get_variable_hover(&word) {
        var_hover
    } else {
        return None;
    };

    Some(Hover {
        contents: HoverContents::Markup(content),
        range: Some(word_range),
    })
}

/// Extract word and its range at the given offset
fn extract_word_at_offset(
    text: &str,
    offset: usize,
    position: Position,
) -> Option<(String, Range)> {
    // Find word boundaries
    let start = text[..offset]
        .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
        .map(|i| i + 1)
        .unwrap_or(0);

    let end = text[offset..]
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| offset + i)
        .unwrap_or(text.len());

    if start >= end {
        return None;
    }

    let word = text[start..end].to_string();

    // Calculate range for the word
    let line = position.line;
    let start_char = position.character.saturating_sub((offset - start) as u32);
    let end_char = start_char + (end - start) as u32;

    let range = Range::new(
        Position::new(line, start_char),
        Position::new(line, end_char),
    );

    Some((word, range))
}

/// Get hover information for a function
fn get_function_hover(name: &str) -> Option<MarkupContent> {
    let registry = create_function_registry();
    let metadata = registry.get_metadata(name)?;

    // Build signature string
    let params_str = metadata
        .signature
        .parameters
        .iter()
        .map(|p| {
            let opt = if p.optional { "?" } else { "" };
            let types = p.parameter_type.join(" | ");
            format!("{}{}: {}", p.name, opt, types)
        })
        .collect::<Vec<_>>()
        .join(", ");

    let signature = format!(
        "{}({}) -> {}",
        name, params_str, metadata.signature.return_type
    );

    // Build documentation
    let mut doc_parts = vec![
        format!("# {}", name),
        String::new(),
        metadata.description.clone(),
        String::new(),
        "## Signature".to_string(),
        format!("```fhirpath\n{}\n```", signature),
    ];

    // Add parameter details
    if !metadata.signature.parameters.is_empty() {
        doc_parts.push(String::new());
        doc_parts.push("## Parameters".to_string());
        doc_parts.push(String::new());
        for param in &metadata.signature.parameters {
            let opt = if param.optional { " *(optional)*" } else { "" };
            doc_parts.push(format!(
                "**`{}`**: `{}`{}",
                param.name,
                param.parameter_type.join(" | "),
                opt
            ));
            if !param.description.is_empty() {
                doc_parts.push(format!("  - {}", param.description));
            }
            doc_parts.push(String::new());
        }
    }

    // Add examples for common functions
    if let Some(example) = get_function_example(name) {
        doc_parts.push("## Example".to_string());
        doc_parts.push(format!("```fhirpath\n{}\n```", example));
    }

    // Add category
    doc_parts.push(String::new());
    doc_parts.push(format!("*Category: {:?}*", metadata.category));

    Some(MarkupContent {
        kind: MarkupKind::Markdown,
        value: doc_parts.join("\n"),
    })
}

/// Get example for common functions
fn get_function_example(name: &str) -> Option<&'static str> {
    match name {
        "where" => Some("Patient.name.where(use = 'official')"),
        "select" => Some("Patient.name.select(family)"),
        "first" => Some("Patient.name.first()"),
        "last" => Some("Patient.name.last()"),
        "count" => Some("Patient.name.count()"),
        "exists" => Some("Patient.name.exists()"),
        "empty" => Some("Patient.name.empty()"),
        "all" => Some("Patient.name.all(use = 'official')"),
        "substring" => Some("Patient.name.family.substring(0, 3)"),
        "contains" => Some("Patient.name.family.contains('Smith')"),
        "toString" => Some("Patient.birthDate.toString()"),
        "toInteger" => Some("Observation.value.toString().toInteger()"),
        _ => None,
    }
}

/// Get hover information for a property
fn get_property_hover(name: &str) -> Option<MarkupContent> {
    let (type_name, description) = get_property_info(name)?;

    let doc = format!(
        "# Property: {}\n\n**Type**: `{}`\n\n{}",
        name, type_name, description
    );

    Some(MarkupContent {
        kind: MarkupKind::Markdown,
        value: doc,
    })
}

/// Get property type and description
fn get_property_info(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        // Resource properties
        "name" => Some(("HumanName[]", "Patient or Practitioner name")),
        "given" => Some(("string[]", "Given names (first, middle, etc.)")),
        "family" => Some(("string", "Family name (surname)")),
        "birthDate" => Some(("date", "Date of birth")),
        "gender" => Some((
            "code",
            "Administrative gender: male | female | other | unknown",
        )),
        "active" => Some(("boolean", "Whether this record is in active use")),
        "identifier" => Some(("Identifier[]", "Business identifiers for this resource")),
        "telecom" => Some(("ContactPoint[]", "Contact details (phone, email, etc.)")),
        "address" => Some(("Address[]", "Physical addresses")),
        "id" => Some(("string", "Logical resource identifier")),
        "resourceType" => Some(("string", "Type name of this resource")),
        "meta" => Some((
            "Meta",
            "Metadata about the resource (version, lastUpdated, etc.)",
        )),
        "text" => Some(("Narrative", "Human-readable narrative")),
        "contained" => Some(("Resource[]", "Contained inline resources")),
        "extension" => Some((
            "Extension[]",
            "Additional content defined by implementations",
        )),
        "modifierExtension" => Some((
            "Extension[]",
            "Extensions that modify the meaning of the resource",
        )),

        // ContactPoint properties
        "system" => Some((
            "code",
            "Contact point system: phone | fax | email | pager | url | sms | other",
        )),
        "value" => Some(("string", "The actual contact point value")),
        "use" => Some(("code", "Purpose: home | work | temp | old | mobile")),
        "rank" => Some(("positiveInt", "Preference order for contacts (1 = highest)")),
        "period" => Some(("Period", "Time period when this contact point is/was valid")),

        // Address properties
        "line" => Some(("string[]", "Street name, number, direction & P.O. Box etc.")),
        "city" => Some(("string", "Name of city, town, etc.")),
        "state" => Some(("string", "Sub-unit of country (abbreviations ok)")),
        "postalCode" => Some(("string", "Postal code for area")),
        "country" => Some((
            "string",
            "Country (e.g. can be ISO 3166 2 or 3 letter code)",
        )),
        "type" => Some(("code", "Address type: postal | physical | both")),

        // Identifier properties
        "assigner" => Some(("Reference", "Organization that issued the identifier")),

        // Choice type properties
        "valueString" => Some(("string", "Value as string")),
        "valueBoolean" => Some(("boolean", "Value as boolean")),
        "valueInteger" => Some(("integer", "Value as integer")),
        "valueDecimal" => Some(("decimal", "Value as decimal")),
        "valueDate" => Some(("date", "Value as date")),
        "valueDateTime" => Some(("dateTime", "Value as dateTime")),
        "valueCode" => Some(("code", "Value as code")),
        "valueCoding" => Some(("Coding", "Value as Coding")),
        "valueCodeableConcept" => Some(("CodeableConcept", "Value as CodeableConcept")),
        "valueQuantity" => Some(("Quantity", "Value as Quantity")),
        "valueReference" => Some(("Reference", "Value as Reference")),

        _ => None,
    }
}

/// Get hover information for a keyword
fn get_keyword_hover(name: &str) -> Option<MarkupContent> {
    let (description, example) = match name {
        "and" => (
            "Logical AND operator. Returns true if both operands are true.",
            "Patient.active and Patient.deceased = false",
        ),
        "or" => (
            "Logical OR operator. Returns true if either operand is true.",
            "Patient.gender = 'male' or Patient.gender = 'female'",
        ),
        "xor" => (
            "Logical XOR (exclusive or) operator. Returns true if exactly one operand is true.",
            "Patient.active xor Patient.deceased",
        ),
        "implies" => (
            "Logical implication operator. Returns false only if left is true and right is false.",
            "Patient.active implies Patient.deceased = false",
        ),
        "div" => (
            "Integer division operator. Performs division and truncates to integer.",
            "Observation.value.toInteger() div 10",
        ),
        "mod" => (
            "Modulo operator. Returns remainder of division.",
            "Observation.value.toInteger() mod 10",
        ),
        "in" => (
            "Collection membership test. Returns true if left value is in right collection.",
            "Patient.gender in ('male' | 'female')",
        ),
        "contains" => (
            "Collection containment test. Returns true if left collection contains right value.",
            "('male' | 'female') contains Patient.gender",
        ),
        "is" => (
            "Type checking operator. Tests if value is of specified type.",
            "value is string",
        ),
        "as" => (
            "Type casting operator. Attempts to cast value to specified type.",
            "value as string",
        ),
        "true" => ("Boolean true literal value.", "Patient.active = true"),
        "false" => ("Boolean false literal value.", "Patient.deceased = false"),
        _ => return None,
    };

    let doc = format!(
        "# Keyword: `{}`\n\n{}\n\n## Example\n```fhirpath\n{}\n```",
        name, description, example
    );

    Some(MarkupContent {
        kind: MarkupKind::Markdown,
        value: doc,
    })
}

/// Get hover information for a variable
fn get_variable_hover(name: &str) -> Option<MarkupContent> {
    let (description, example) = match name {
        "$this" => (
            "The current context item in an iteration. Available in functions like where(), select(), all(), etc.",
            "Patient.name.where($this.use = 'official')",
        ),
        "$index" => (
            "Zero-based index of the current item in iteration. Available in where(), select(), etc.",
            "Patient.name.where($index > 0)",
        ),
        "$total" => (
            "Total number of items being iterated. Available in where(), select(), etc.",
            "Patient.name.where($index < $total - 1)",
        ),
        "$context" => (
            "The root context of the evaluation. Allows access to the initial input resource.",
            "Patient.name.where(family = $context.address.city)",
        ),
        _ => return None,
    };

    let doc = format!(
        "# Variable: `{}`\n\n{}\n\n## Example\n```fhirpath\n{}\n```",
        name, description, example
    );

    Some(MarkupContent {
        kind: MarkupKind::Markdown,
        value: doc,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_extract_word_at_offset() {
        let text = "Patient.name.where";
        let position = Position::new(0, 14); // Position at 'w' in 'where'
        let result = extract_word_at_offset(text, 14, position);

        assert!(result.is_some());
        let (word, _) = result.unwrap();
        assert_eq!(word, "where");
    }

    #[test]
    fn test_extract_variable() {
        let text = "where($this.name = 'John')";
        let position = Position::new(0, 7); // Position at '$'
        let result = extract_word_at_offset(text, 7, position);

        assert!(result.is_some());
        let (word, _) = result.unwrap();
        assert_eq!(word, "$this");
    }

    #[test]
    fn test_get_function_hover() {
        let hover = get_function_hover("where");
        assert!(hover.is_some());

        let content = hover.unwrap();
        assert!(content.value.contains("where"));
        assert!(content.value.contains("Signature"));
    }

    #[test]
    fn test_get_property_hover() {
        let hover = get_property_hover("name");
        assert!(hover.is_some());

        let content = hover.unwrap();
        assert!(content.value.contains("name"));
        assert!(content.value.contains("HumanName"));
    }

    #[test]
    fn test_get_keyword_hover() {
        let hover = get_keyword_hover("and");
        assert!(hover.is_some());

        let content = hover.unwrap();
        assert!(content.value.contains("and"));
        assert!(content.value.contains("Logical AND"));
    }

    #[test]
    fn test_get_variable_hover() {
        let hover = get_variable_hover("$this");
        assert!(hover.is_some());

        let content = hover.unwrap();
        assert!(content.value.contains("$this"));
        assert!(content.value.contains("current context"));
    }

    #[test]
    fn test_generate_hover_function() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "Patient.name.where(use = 'official')".to_string(),
            1,
        );

        // Position at 'where'
        let hover = generate_hover(&doc, Position::new(0, 14));
        assert!(hover.is_some());

        let h = hover.unwrap();
        if let HoverContents::Markup(content) = h.contents {
            assert!(content.value.contains("where"));
        }
    }

    #[test]
    fn test_generate_hover_keyword() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "Patient.active and Patient.deceased".to_string(),
            1,
        );

        // Position at 'and'
        let hover = generate_hover(&doc, Position::new(0, 16));
        assert!(hover.is_some());
    }

    #[test]
    fn test_generate_hover_no_match() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "Patient.unknownProperty".to_string(),
            1,
        );

        // Position at 'unknownProperty'
        let hover = generate_hover(&doc, Position::new(0, 10));
        assert!(hover.is_none());
    }
}
