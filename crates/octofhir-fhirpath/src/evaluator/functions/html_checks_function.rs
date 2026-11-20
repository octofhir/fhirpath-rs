//! htmlChecks function implementation
//!
//! The htmlChecks function validates XHTML content according to FHIR narrative rules.
//! Returns true if the XHTML passes all validation checks.
//! Syntax: xhtml.htmlChecks()

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Allowed HTML tags per FHIR narrative rules
const ALLOWED_TAGS: &[&str] = &[
    "a",
    "abbr",
    "acronym",
    "b",
    "big",
    "blockquote",
    "br",
    "caption",
    "cite",
    "code",
    "col",
    "colgroup",
    "dd",
    "dfn",
    "div",
    "dl",
    "dt",
    "em",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "i",
    "img",
    "kbd",
    "li",
    "ol",
    "p",
    "pre",
    "q",
    "samp",
    "small",
    "span",
    "strong",
    "sub",
    "sup",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "tt",
    "ul",
    "var",
];

/// Allowed attributes per FHIR narrative rules
const ALLOWED_ATTRS: &[&str] = &[
    "abbr",
    "align",
    "alt",
    "axis",
    "bgcolor",
    "border",
    "cellpadding",
    "cellspacing",
    "char",
    "charoff",
    "class",
    "colspan",
    "dir",
    "headers",
    "height",
    "href",
    "id",
    "lang",
    "name",
    "rowspan",
    "scope",
    "src",
    "style",
    "title",
    "valign",
    "width",
    "xml:lang",
    "xmlns",
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

fn is_allowed(name: &str, allowed: &[&str]) -> bool {
    allowed.binary_search(&name).is_ok()
        || (name
            .bytes()
            .any(|b| !b.is_ascii() || b.is_ascii_uppercase())
            && allowed.binary_search(&name.to_lowercase().as_str()).is_ok())
}

/// Basic XHTML validation against FHIR narrative rules.
/// Checks for allowed tags and attributes using simple string scanning.
fn validate_xhtml(xhtml: &str, require_root_div: bool) -> bool {
    let input = xhtml.trim();
    if input.is_empty() {
        return true;
    }

    // A narrative (`xhtml` element) must start with <div xmlns="http://www.w3.org/1999/xhtml">
    // (or just <div); a plain string is checked as an HTML fragment.
    if require_root_div && !input.starts_with("<div") {
        return false;
    }

    // Simple tag scanner: find all <tagname and </tagname> patterns
    let mut pos = 0;
    let bytes = input.as_bytes();
    while let Some(offset) = input[pos..].find('<') {
        pos += offset;
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
            if !is_allowed(tag_name, ALLOWED_TAGS) {
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

                    if attr_pos >= bytes.len() || bytes[attr_pos] == b'>' || bytes[attr_pos] == b'/'
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
                        if !is_allowed(attr_name, ALLOWED_ATTRS) {
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

        let Some(end) = input[pos..].find('>') else {
            break;
        };
        pos += end + 1;
    }

    true
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for HtmlChecksFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
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
            let (xhtml, is_narrative) = match item {
                FhirPathValue::String(s, type_info, _) => (
                    s.as_str(),
                    type_info.name.as_deref() == Some("xhtml") || type_info.type_name == "xhtml",
                ),
                // When constraints are evaluated on primitive elements (like xhtml div),
                // the value may be wrapped as Resource(JsonValue::String(...))
                FhirPathValue::Resource(json, _, _) => {
                    if let Some(s) = json.as_str() {
                        (s, true)
                    } else {
                        return Ok(EvaluationResult {
                            value: crate::core::Collection::single(FhirPathValue::boolean(false)),
                        });
                    }
                }
                // Not a string: there is no HTML to check.
                _ => {
                    return Ok(EvaluationResult {
                        value: crate::core::Collection::empty(),
                    });
                }
            };

            if !validate_xhtml(xhtml, is_narrative) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_preserves_allowed_names_and_skips_text() {
        assert!(ALLOWED_TAGS.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(ALLOWED_ATTRS.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(validate_xhtml(
            "<div>текст<!-- ok --><P CLASS='body'>value</P></div>",
            true
        ));
        assert!(!validate_xhtml("<div><script></script></div>", true));
        assert!(!validate_xhtml(
            "<div><p unexpected='x'>value</p></div>",
            true
        ));
        assert!(!validate_xhtml("<div>text<!-- unclosed", true));
    }
}
