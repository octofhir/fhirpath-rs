//! htmlChecks function implementation
//!
//! The htmlChecks function validates XHTML content according to FHIR narrative rules.
//! Returns true if the XHTML passes all validation checks.
//! Syntax: xhtml.htmlChecks()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Allowed HTML tags per FHIR narrative rules
const ALLOWED_TAGS: &[&str] = &[
    "div",
    "p",
    "b",
    "i",
    "em",
    "strong",
    "small",
    "big",
    "tt",
    "code",
    "q",
    "sub",
    "sup",
    "samp",
    "kbd",
    "var",
    "cite",
    "dfn",
    "abbr",
    "acronym",
    "span",
    "br",
    "hr",
    "a",
    "img",
    "table",
    "thead",
    "tbody",
    "tfoot",
    "tr",
    "th",
    "td",
    "col",
    "colgroup",
    "caption",
    "ul",
    "ol",
    "li",
    "dl",
    "dt",
    "dd",
    "pre",
    "blockquote",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
];

/// Allowed attributes per FHIR narrative rules
const ALLOWED_ATTRS: &[&str] = &[
    "id",
    "class",
    "style",
    "lang",
    "xml:lang",
    "dir",
    "title",
    "xmlns",
    "src",
    "href",
    "name",
    "alt",
    "colspan",
    "rowspan",
    "headers",
    "scope",
    "border",
    "cellpadding",
    "cellspacing",
    "width",
    "height",
    "align",
    "valign",
    "char",
    "charoff",
    "abbr",
    "axis",
    "bgcolor",
];

/// HtmlChecks function evaluator
pub struct HtmlChecksFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl HtmlChecksFunctionEvaluator {
    /// Create a new htmlChecks function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "htmlChecks".to_string(),
                description: "Validates XHTML content according to FHIR narrative rules"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

/// Basic XHTML validation against FHIR narrative rules.
/// Checks for allowed tags and attributes using simple string scanning.
fn validate_xhtml(xhtml: &str) -> bool {
    let input = xhtml.trim();
    if input.is_empty() {
        return true;
    }

    // Must start with <div xmlns="http://www.w3.org/1999/xhtml"> (or just <div)
    // The FHIR spec requires a root div element
    if !input.starts_with("<div") {
        return false;
    }

    // Simple tag scanner: find all <tagname and </tagname> patterns
    let mut pos = 0;
    let bytes = input.as_bytes();
    while pos < bytes.len() {
        if bytes[pos] == b'<' {
            // Skip comments
            if input[pos..].starts_with("<!--") {
                if let Some(end) = input[pos..].find("-->") {
                    pos += end + 3;
                    continue;
                }
                return false; // Unclosed comment
            }

            // Skip processing instructions
            if input[pos..].starts_with("<?") {
                if let Some(end) = input[pos..].find("?>") {
                    pos += end + 2;
                    continue;
                }
                return false;
            }

            // Skip closing tags (we trust structure if tags are allowed)
            let is_closing = pos + 1 < bytes.len() && bytes[pos + 1] == b'/';
            let tag_start = if is_closing { pos + 2 } else { pos + 1 };

            // Extract tag name
            let mut tag_end = tag_start;
            while tag_end < bytes.len()
                && bytes[tag_end] != b' '
                && bytes[tag_end] != b'>'
                && bytes[tag_end] != b'/'
            {
                tag_end += 1;
            }

            if tag_end > tag_start {
                let tag_name = &input[tag_start..tag_end];
                let tag_lower = tag_name.to_lowercase();
                if !ALLOWED_TAGS.contains(&tag_lower.as_str()) {
                    return false;
                }

                // If not a closing tag, check attributes
                if !is_closing {
                    let mut attr_pos = tag_end;
                    while attr_pos < bytes.len() && bytes[attr_pos] != b'>' {
                        // Skip whitespace
                        while attr_pos < bytes.len()
                            && (bytes[attr_pos] == b' '
                                || bytes[attr_pos] == b'\n'
                                || bytes[attr_pos] == b'\r'
                                || bytes[attr_pos] == b'\t')
                        {
                            attr_pos += 1;
                        }

                        if attr_pos >= bytes.len()
                            || bytes[attr_pos] == b'>'
                            || bytes[attr_pos] == b'/'
                        {
                            break;
                        }

                        // Extract attribute name
                        let attr_start = attr_pos;
                        while attr_pos < bytes.len()
                            && bytes[attr_pos] != b'='
                            && bytes[attr_pos] != b' '
                            && bytes[attr_pos] != b'>'
                            && bytes[attr_pos] != b'/'
                        {
                            attr_pos += 1;
                        }

                        if attr_pos > attr_start {
                            let attr_name = &input[attr_start..attr_pos];
                            let attr_lower = attr_name.to_lowercase();
                            if !ALLOWED_ATTRS.contains(&attr_lower.as_str()) {
                                return false;
                            }
                        }

                        // Skip the attribute value
                        if attr_pos < bytes.len() && bytes[attr_pos] == b'=' {
                            attr_pos += 1;
                            // Skip whitespace
                            while attr_pos < bytes.len() && bytes[attr_pos] == b' ' {
                                attr_pos += 1;
                            }
                            if attr_pos < bytes.len() {
                                if bytes[attr_pos] == b'"' {
                                    attr_pos += 1;
                                    while attr_pos < bytes.len() && bytes[attr_pos] != b'"' {
                                        attr_pos += 1;
                                    }
                                    if attr_pos < bytes.len() {
                                        attr_pos += 1;
                                    }
                                } else if bytes[attr_pos] == b'\'' {
                                    attr_pos += 1;
                                    while attr_pos < bytes.len() && bytes[attr_pos] != b'\'' {
                                        attr_pos += 1;
                                    }
                                    if attr_pos < bytes.len() {
                                        attr_pos += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Skip to end of tag
            while pos < bytes.len() && bytes[pos] != b'>' {
                pos += 1;
            }
        }
        pos += 1;
    }

    true
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for HtmlChecksFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "htmlChecks function takes no arguments".to_string(),
            ));
        }

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        for item in &input {
            let xhtml = match item {
                FhirPathValue::String(s, _, _) => s.as_str(),
                _ => {
                    return Ok(EvaluationResult {
                        value: crate::core::Collection::single(FhirPathValue::boolean(false)),
                    });
                }
            };

            if !validate_xhtml(xhtml) {
                return Ok(EvaluationResult {
                    value: crate::core::Collection::single(FhirPathValue::boolean(false)),
                });
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::boolean(true)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
