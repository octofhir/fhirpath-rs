//! `hasTemplateIdOf(profileUrl)`: the FHIRPath function the CDA core specification adds for CDA
//! profiles (hl7.cda.uv.core, "FHIRPath Supplements").
//!
//! It is true when the element has a `templateId` corresponding to the identifier of the profile
//! the URL names: a profile with the identifier `urn:hl7ii:2.16.840.1.113883.10.20.22.4.7:2014-06-09`
//! matches `<templateId root="2.16.840.1.113883.10.20.22.4.7" extension="2014-06-09"/>`. The
//! specification gives it as a shorthand for
//! `templateId.where(root = '…' and extension = '…').exists()`.
//!
//! Profiles are known to the host, not to FHIRPath: a [`TemplateIdResolver`] gives the template
//! ids of a profile URL. Without one, or for a URL it does not know, the argument may itself be a
//! template identifier (`urn:hl7ii:root:extension`, `urn:oid:root`, or a bare OID). A template
//! without an extension matches its root whatever the extension.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::node::FhirNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// A template identifier: `root` and, when the template is versioned, `extension`.
pub type TemplateId = (String, Option<String>);

/// Gives the template ids of a profile, by its canonical URL.
pub trait TemplateIdResolver: Send + Sync {
    fn template_ids(&self, profile_url: &str) -> Vec<TemplateId>;
}

impl TemplateIdResolver for HashMap<String, Vec<TemplateId>> {
    fn template_ids(&self, profile_url: &str) -> Vec<TemplateId> {
        let url = profile_url.split('|').next().unwrap_or(profile_url);
        self.get(url).cloned().unwrap_or_default()
    }
}

/// Reads a template identifier: `urn:hl7ii:root[:extension]`, `urn:oid:root` or a bare OID.
pub fn parse_template_id(text: &str) -> Option<TemplateId> {
    if let Some(rest) = text.strip_prefix("urn:hl7ii:") {
        let (root, extension) = match rest.split_once(':') {
            Some((root, extension)) => (root, Some(extension.to_owned())),
            None => (rest, None),
        };
        return (!root.is_empty()).then(|| (root.to_owned(), extension));
    }
    let root = text.strip_prefix("urn:oid:").unwrap_or(text);
    let is_oid = !root.is_empty()
        && root.bytes().all(|b| b.is_ascii_digit() || b == b'.')
        && root.contains('.');
    is_oid.then(|| (root.to_owned(), None))
}

/// The `hasTemplateIdOf` function.
pub struct HasTemplateIdOfFunctionEvaluator {
    metadata: FunctionMetadata,
    resolver: Option<Arc<dyn TemplateIdResolver>>,
}

impl HasTemplateIdOfFunctionEvaluator {
    /// Without a resolver: the argument must be a template identifier.
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self::new(None))
    }

    /// With a resolver for profile URLs. Register it in place of the default one:
    /// `registry.register_pure_function(HasTemplateIdOfFunctionEvaluator::with_resolver(r))`.
    pub fn with_resolver(resolver: Arc<dyn TemplateIdResolver>) -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self::new(Some(resolver)))
    }

    fn new(resolver: Option<Arc<dyn TemplateIdResolver>>) -> Self {
        Self {
            metadata: FunctionMetadata {
                name: "hasTemplateIdOf".to_string(),
                description: "Whether a CDA element has a templateId of the profile the URL names"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "profileUrl".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The canonical URL of the profile, or a template identifier"
                            .to_string(),
                        default_value: None,
                    }],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::CDA,
                requires_terminology: false,
                requires_model: false,
            },
            resolver,
        }
    }

    fn templates(&self, argument: &str) -> Vec<TemplateId> {
        if let Some(resolver) = &self.resolver {
            let found = resolver.template_ids(argument);
            if !found.is_empty() {
                return found;
            }
        }
        parse_template_id(argument).into_iter().collect()
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for HasTemplateIdOfFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        let argument = match args.as_slice() {
            [values] if values.len() == 1 => match &values[0] {
                FhirPathValue::String(s, _, _) => s.clone(),
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0054,
                        "hasTemplateIdOf needs a string: the profile URL".to_string(),
                    ));
                }
            },
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0053,
                    "hasTemplateIdOf takes one argument: the profile URL".to_string(),
                ));
            }
        };
        let templates = self.templates(&argument);
        let found = !templates.is_empty()
            && input.iter().any(|element| match element {
                FhirPathValue::Resource(json, _, _) => has_template(json, &templates),
                _ => false,
            });
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::boolean(found)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

/// Whether one of the element's `templateId`s is one of `templates`.
fn has_template(element: &FhirNode, templates: &[TemplateId]) -> bool {
    let Some(ids) = element.get("templateId") else {
        return false;
    };
    let ids: Vec<&FhirNode> = match ids.as_array() {
        Some(items) => items.iter().collect(),
        None => vec![ids],
    };
    ids.into_iter().any(|id| {
        let Some(root) = id.get("root").and_then(FhirNode::as_str) else {
            return false;
        };
        let extension = id.get("extension").and_then(FhirNode::as_str);
        templates.iter().any(|(wanted_root, wanted_extension)| {
            wanted_root == root
                && match wanted_extension {
                    Some(wanted) => extension == Some(wanted.as_str()),
                    None => true,
                }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_template_identifiers() {
        assert_eq!(
            parse_template_id("urn:hl7ii:2.16.840.1.113883.10.20.22.4.7:2014-06-09"),
            Some((
                "2.16.840.1.113883.10.20.22.4.7".to_owned(),
                Some("2014-06-09".to_owned())
            ))
        );
        assert_eq!(
            parse_template_id("urn:oid:2.16.840.1.113883.10.20.22.4.7"),
            Some(("2.16.840.1.113883.10.20.22.4.7".to_owned(), None))
        );
        assert_eq!(
            parse_template_id("2.16.840.1.113883.10.20.22.4.7"),
            Some(("2.16.840.1.113883.10.20.22.4.7".to_owned(), None))
        );
        assert_eq!(
            parse_template_id("http://hl7.org/cda/us/ccda/StructureDefinition/X"),
            None
        );
    }
}
