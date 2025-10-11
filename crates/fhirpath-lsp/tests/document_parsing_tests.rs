use fhirpath_lsp::FhirPathDocument;
use url::Url;

#[test]
fn test_document_with_directives() {
    let text = r#"
/** @input {"resourceType": "Patient", "id": "test"} */

Patient.name.family;
Patient.birthDate
"#;

    let doc = FhirPathDocument::new(
        Url::parse("file:///test.fhirpath").unwrap(),
        text.to_string(),
        1,
    );

    assert_eq!(doc.directives.len(), 1);
    assert_eq!(doc.expressions.len(), 2);
}

#[test]
fn test_multiple_directives() {
    let text = r#"
/** @input {"resourceType": "Patient"} */
/** @input-file ./patient.json */

Patient.name
"#;

    let doc = FhirPathDocument::new(
        Url::parse("file:///test.fhirpath").unwrap(),
        text.to_string(),
        1,
    );

    assert_eq!(doc.directives.len(), 2);
    assert_eq!(doc.expressions.len(), 1);
}

#[test]
fn test_document_without_directives() {
    let text = "Patient.name.family; Patient.birthDate";

    let doc = FhirPathDocument::new(
        Url::parse("file:///test.fhirpath").unwrap(),
        text.to_string(),
        1,
    );

    assert_eq!(doc.directives.len(), 0);
    assert_eq!(doc.expressions.len(), 2);
}

#[test]
fn test_single_expression_no_semicolon() {
    let text = "Patient.name.family";

    let doc = FhirPathDocument::new(
        Url::parse("file:///test.fhirpath").unwrap(),
        text.to_string(),
        1,
    );

    assert_eq!(doc.expressions.len(), 1);
    assert_eq!(doc.expressions[0].text, "Patient.name.family");
}
