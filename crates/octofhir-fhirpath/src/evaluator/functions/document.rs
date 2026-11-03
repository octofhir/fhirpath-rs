//! HL7 Java document-navigation extensions, separate from standard FHIRPath.
//!
//! Nodes retain their shared identity when sections are returned. References are
//! resolved within the document only; a standalone Composition can use the host
//! server provider. No resource or choice-type catalogue is embedded here.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;

use crate::core::node::FhirNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, ProviderPureFunctionEvaluator,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Names of the supported HL7 document extensions.
pub const DOCUMENT_FUNCTIONS: &[&str] = &[
    "section",
    "entry",
    "entryIn",
    "resources",
    "resourcesIn",
    "sameSubject",
    "hasCoding",
];

/// A document function whose arguments are evaluated in the caller's scope.
pub struct DocumentFunction {
    metadata: FunctionMetadata,
}

impl DocumentFunction {
    /// Build the document extension evaluators for registration.
    pub fn evaluators() -> Vec<Arc<dyn ProviderPureFunctionEvaluator>> {
        DOCUMENT_FUNCTIONS
            .iter()
            .map(|name| {
                let (min, max, parameters, result) = match *name {
                    "section" | "entry" => (2, 4, vec!["system", "code", "min", "max"], "Resource"),
                    "entryIn" => (1, 3, vec!["valueSet", "min", "max"], "Resource"),
                    "resources" => (2, 3, vec!["system", "code", "type"], "Resource"),
                    "resourcesIn" => (1, 2, vec!["valueSet", "type"], "Resource"),
                    "sameSubject" => (0, 1, vec!["other"], "Boolean"),
                    _ => (2, 3, vec!["system", "code", "version"], "Boolean"),
                };
                Arc::new(Self {
                    metadata: FunctionMetadata {
                        name: (*name).into(),
                        description: format!("HL7 Java document extension: {name}()"),
                        signature: FunctionSignature {
                            input_type: "Any".into(),
                            parameters: parameters
                                .into_iter()
                                .enumerate()
                                .map(|(i, parameter)| FunctionParameter {
                                    name: parameter.into(),
                                    parameter_type: vec!["Any".into()],
                                    optional: i >= min,
                                    is_expression: false,
                                    description: parameter.into(),
                                    default_value: None,
                                })
                                .collect(),
                            return_type: result.into(),
                            polymorphic: true,
                            min_params: min,
                            max_params: Some(max),
                        },
                        argument_evaluation: ArgumentEvaluationStrategy::Current,
                        null_propagation: NullPropagationStrategy::None,
                        empty_propagation: EmptyPropagation::Custom,
                        deterministic: false,
                        category: FunctionCategory::TreeNavigation,
                        requires_terminology: false,
                        requires_model: true,
                    },
                }) as Arc<dyn ProviderPureFunctionEvaluator>
            })
            .collect()
    }
}

fn invalid(message: impl Into<String>) -> FhirPathError {
    FhirPathError::evaluation_error(crate::core::error_code::FP0053, message)
}

/// Whether the argument count is permitted, including paired min/max arguments.
pub fn valid_arity(name: &str, count: usize) -> bool {
    match name {
        "section" | "entry" => matches!(count, 2 | 4),
        "entryIn" => matches!(count, 1 | 3),
        "resources" | "hasCoding" => matches!(count, 2 | 3),
        "resourcesIn" => matches!(count, 1 | 2),
        "sameSubject" => count <= 1,
        _ => false,
    }
}

fn text_arg(args: &[Collection], index: usize) -> Option<String> {
    match args.get(index)?.first()? {
        FhirPathValue::String(s, ..) => Some(s.clone()),
        FhirPathValue::Integer(v, ..) => Some(v.to_string()),
        FhirPathValue::Decimal(v, ..) => Some(v.to_string()),
        FhirPathValue::Boolean(v, ..) => Some(v.to_string()),
        _ => None,
    }
}

fn nodes(node: &FhirNode, field: &str) -> Vec<FhirNode> {
    match node.get(field) {
        Some(FhirNode::Array(items)) => items.to_vec(),
        Some(FhirNode::Null) | None => vec![],
        Some(value) => vec![value.clone()],
    }
}

fn string<'a>(node: &'a FhirNode, field: &str) -> Option<&'a str> {
    node.get(field)?.as_str()
}

fn resource(value: &FhirPathValue) -> Option<&FhirNode> {
    match value {
        FhirPathValue::Resource(node, ..) => Some(node),
        _ => None,
    }
}

fn composition(node: &FhirNode) -> Option<FhirNode> {
    match string(node, "resourceType") {
        Some("Composition") => Some(node.clone()),
        Some("Bundle") => nodes(node, "entry")
            .iter()
            .filter_map(|entry| entry.get("resource"))
            .find(|r| string(r, "resourceType") == Some("Composition"))
            .cloned(),
        _ => None,
    }
}

fn sections(comp: &FhirNode) -> Vec<FhirNode> {
    let mut result = Vec::new();
    let mut pending = nodes(comp, "section");
    pending.reverse();
    while let Some(section) = pending.pop() {
        let mut children = nodes(&section, "section");
        children.reverse();
        pending.extend(children);
        result.push(section);
    }
    result
}

fn codings(concept: &FhirNode) -> Vec<FhirNode> {
    nodes(concept, "coding")
}

fn matches(
    coding: &FhirNode,
    system: Option<&str>,
    code: Option<&str>,
    version: Option<&str>,
) -> bool {
    [("system", system), ("code", code), ("version", version)]
        .iter()
        .all(|(field, expected)| expected.is_none_or(|v| string(coding, field) == Some(v)))
}

fn has_code(concept: &FhirNode, system: Option<&str>, code: Option<&str>) -> bool {
    codings(concept)
        .iter()
        .any(|coding| matches(coding, system, code, None))
}

fn resource_concepts(node: &FhirNode) -> Vec<FhirNode> {
    // These fields are the document extension's definition of a resource's code,
    // not a model-provider catalogue. Choice names come from their FHIR JSON form.
    [
        "code",
        "vaccineCode",
        "medication",
        "medicationCodeableConcept",
    ]
    .iter()
    .flat_map(|field| nodes(node, field))
    .filter_map(|value| {
        if value.get("coding").is_some() {
            Some(value)
        } else {
            value.get("concept").cloned()
        }
    })
    .collect()
}

#[derive(Clone, Copy)]
struct Bounds {
    min: usize,
    max: Option<usize>,
}

impl Bounds {
    fn read(args: &[Collection], offset: usize, section: bool) -> Result<Self> {
        let integer = |value: &str| -> Result<usize> {
            if value.is_empty() || value.len() > 9 || !value.bytes().all(|c| c.is_ascii_digit()) {
                return Err(invalid("Document bounds must be non-negative integers"));
            }
            value
                .parse()
                .map_err(|_| invalid("Document bound is out of range"))
        };
        let min = text_arg(args, offset)
            .map(|s| integer(&s))
            .transpose()?
            .unwrap_or(usize::from(section));
        let max = match text_arg(args, offset + 1).as_deref() {
            None => {
                if section {
                    Some(1)
                } else {
                    None
                }
            }
            Some("*") => None,
            Some(s) => Some(integer(s)?),
        };
        if max.is_some_and(|max| min > max) {
            return Err(invalid("Document min must not exceed max"));
        }
        Ok(Self { min, max })
    }
    fn accepts(self, count: usize) -> bool {
        count >= self.min && self.max.is_none_or(|max| count <= max)
    }
}

/// Per-call index. Ambiguous references are represented by more than one entry,
/// never by silently overwriting a previous resource.
struct Document {
    root: Option<FhirNode>,
    bundle: Option<FhirNode>,
    index: HashMap<String, Vec<FhirNode>>,
}

impl Document {
    fn new(context: &EvaluationContext, index_references: bool) -> Self {
        let root = context
            .root_resource_value()
            .as_ref()
            .and_then(resource)
            .cloned();
        let bundle = root
            .as_ref()
            .filter(|r| string(r, "resourceType") == Some("Bundle"))
            .cloned();
        let mut index: HashMap<String, Vec<FhirNode>> = HashMap::new();
        if let Some(bundle) = &bundle
            && index_references
        {
            for entry in nodes(bundle, "entry") {
                if let (Some(url), Some(r)) = (string(&entry, "fullUrl"), entry.get("resource")) {
                    index.entry(url.into()).or_default().push(r.clone());
                    if let Some(version) = r.get("meta").and_then(|m| string(m, "versionId")) {
                        index
                            .entry(format!("{url}/_history/{version}"))
                            .or_default()
                            .push(r.clone());
                    }
                }
            }
        }
        Self {
            root,
            bundle,
            index,
        }
    }

    fn owner(&self, section: &FhirNode) -> Option<(FhirNode, Option<String>)> {
        if let Some(bundle) = &self.bundle {
            for entry in nodes(bundle, "entry") {
                if let Some(comp) = entry.get("resource")
                    && string(comp, "resourceType") == Some("Composition")
                    && sections(comp)
                        .iter()
                        .any(|s| s.identity() == section.identity())
                {
                    return Some((comp.clone(), string(&entry, "fullUrl").map(str::to_string)));
                }
            }
        }
        self.root
            .as_ref()
            .filter(|root| string(root, "resourceType") == Some("Composition"))
            .filter(|comp| {
                sections(comp)
                    .iter()
                    .any(|s| s.identity() == section.identity())
            })
            .map(|comp| (comp.clone(), None))
    }

    async fn entries(&self, section: &FhirNode, context: &EvaluationContext) -> Vec<FhirNode> {
        if section.get("resourceType").is_some() {
            return vec![];
        }
        let owner = self.owner(section);
        let mut result = vec![];
        for reference in nodes(section, "entry") {
            let Some(reference) = string(&reference, "reference") else {
                continue;
            };
            if let Some(id) = reference.strip_prefix('#') {
                if let Some((comp, _)) = &owner {
                    result.extend(
                        nodes(comp, "contained")
                            .into_iter()
                            .filter(|r| string(r, "id") == Some(id))
                            .take(1),
                    );
                }
                continue;
            }
            let reference = reference.split('#').next().unwrap_or(reference);
            if self.bundle.is_some() {
                let key = if url::Url::parse(reference).is_ok() {
                    Some(reference.to_string())
                } else {
                    owner.as_ref().and_then(|(comp, full_url)| {
                        let full_url = full_url.as_deref()?;
                        let own = format!("Composition/{}", string(comp, "id")?);
                        let base = full_url.strip_suffix(&own)?;
                        if base.is_empty()
                            || url::Url::parse(base).is_ok_and(|u| {
                                matches!(u.scheme(), "http" | "https")
                                    && u.query().is_none()
                                    && u.fragment().is_none()
                            })
                        {
                            Some(format!("{base}{reference}"))
                        } else {
                            None
                        }
                    })
                };
                if let Some(matches) = key.as_ref().and_then(|key| self.index.get(key))
                    && matches.len() == 1
                {
                    result.push(matches[0].clone());
                }
            } else if let Some(server) = context.server_provider()
                && let Some((kind, id)) = reference.split_once('/')
                && let Ok(Some(value)) = server.read(kind, id).await
            {
                result.push(FhirNode::from_json(&value));
            }
        }
        result
    }
}

fn subject(node: &FhirNode) -> Option<FhirNode> {
    let mut values = nodes(node, "subject");
    if values.is_empty() {
        values = nodes(node, "patient");
    }
    if values.len() == 1 {
        values.pop()
    } else {
        None
    }
}

fn same_subject(input: &Collection, other: Option<&Collection>, doc: &Document) -> Option<bool> {
    if input.len() != 1 {
        return None;
    }
    let other = match other {
        Some(values) if values.len() == 1 => resource(values.first()?)?.clone(),
        Some(_) => return None,
        None => composition(
            doc.bundle
                .as_ref()
                .filter(|b| string(b, "type") == Some("document"))?,
        )?,
    };
    let a = subject(resource(input.first()?)?)?;
    let b = subject(&other)?;
    if let (Some(a), Some(b)) = (
        string(&a, "reference").filter(|s| !s.is_empty()),
        string(&b, "reference").filter(|s| !s.is_empty()),
    ) {
        return Some(a == b);
    }
    Some(match (a.get("identifier"), b.get("identifier")) {
        (Some(a), Some(b)) => {
            string(a, "value").is_some_and(|v| !v.is_empty() && Some(v) == string(b, "value"))
                && string(a, "system") == string(b, "system")
        }
        _ => false,
    })
}

async fn wrap(node: FhirNode, section: bool, context: &EvaluationContext) -> Result<FhirPathValue> {
    let model = context.model_provider();
    let info = if section {
        match model
            .get_type("Composition")
            .await
            .map_err(|e| invalid(e.to_string()))?
        {
            Some(comp) => model
                .get_element_type(&comp, "section")
                .await
                .map_err(|e| invalid(e.to_string()))?,
            None => None,
        }
    } else {
        match string(&node, "resourceType") {
            Some(kind) => model
                .get_type(kind)
                .await
                .map_err(|e| invalid(e.to_string()))?,
            None => None,
        }
    };
    Ok(match info {
        Some(info) => FhirPathValue::Resource(node, Arc::new(info), None),
        None => FhirPathValue::resource_from_node(node),
    })
}

#[async_trait]
impl ProviderPureFunctionEvaluator for DocumentFunction {
    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }

    async fn evaluate(
        &self,
        input: Collection,
        args: Vec<Collection>,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        let name = self.metadata.name.as_str();
        if !valid_arity(name, args.len()) {
            return Err(invalid(format!("Invalid argument count for {name}()")));
        }
        if name == "sameSubject" {
            return Ok(EvaluationResult::new(
                same_subject(&input, args.first(), &Document::new(context, false))
                    .map(|b| Collection::single(FhirPathValue::boolean(b)))
                    .unwrap_or_else(Collection::empty),
            ));
        }
        if args.iter().any(|a| a.len() > 1) {
            return Ok(EvaluationResult::new(Collection::empty()));
        }
        let system = text_arg(&args, 0);
        let code = text_arg(&args, 1);
        if name == "hasCoding" {
            let version = text_arg(&args, 2);
            let found = input.iter().any(|value| {
                let Some(node) = resource(value) else {
                    return false;
                };
                let info = value.type_info();
                let kind = info.name.as_deref().unwrap_or(&info.type_name);
                if kind == "CodeableConcept" || node.get("coding").is_some() {
                    codings(node)
                        .iter()
                        .any(|c| matches(c, system.as_deref(), code.as_deref(), version.as_deref()))
                } else {
                    (kind.trim_end_matches("[]").rsplit('.').next() == Some("Coding")
                        || (node.get("resourceType").is_none()
                            && node.get("code").is_some_and(|c| c.as_str().is_some())))
                        && matches(node, system.as_deref(), code.as_deref(), version.as_deref())
                }
            });
            return Ok(EvaluationResult::new(if input.is_empty() {
                Collection::empty()
            } else {
                Collection::single(FhirPathValue::boolean(found))
            }));
        }
        let doc = Document::new(context, matches!(name, "entry" | "entryIn"));
        let in_valueset = matches!(name, "entryIn" | "resourcesIn");
        let bounds = if matches!(name, "section" | "entry" | "entryIn") {
            Bounds::read(&args, if in_valueset { 1 } else { 2 }, name == "section")?
        } else {
            Bounds { min: 0, max: None }
        };
        let filter_type = if name.starts_with("resources") {
            text_arg(&args, if in_valueset { 1 } else { 2 })
        } else {
            None
        };
        if let Some(kind) = &filter_type {
            let names = context
                .model_provider()
                .get_resource_types()
                .await
                .map_err(|e| invalid(e.to_string()))?;
            if !names.contains(kind) {
                return Err(invalid(format!("Unknown resource type: {kind}")));
            }
        }
        if in_valueset {
            let Some(url) = system.as_deref() else {
                return Ok(EvaluationResult::new(Collection::empty()));
            };
            let canonical = url.split_once('|').map_or(url, |(url, _)| url);
            if url.matches('|').count() > 1
                || url.ends_with('|')
                || url.len() > 320
                || url.chars().any(char::is_whitespace)
                || url::Url::parse(canonical).is_err()
            {
                return Err(invalid("ValueSet must be a canonical URL"));
            }
        }
        let mut groups = vec![];
        let mut seen = HashSet::new();
        for item in input.iter().filter_map(resource) {
            let candidates = match name {
                "section" => composition(item).map(|c| sections(&c)).unwrap_or_default(),
                "entry" | "entryIn" => doc.entries(item, context).await,
                _ => {
                    let bundle = if string(item, "resourceType") == Some("Bundle") {
                        Some(item)
                    } else {
                        doc.bundle.as_ref()
                    };
                    let Some(bundle) = bundle else { continue };
                    if !seen.insert(bundle.identity()) {
                        continue;
                    }
                    nodes(bundle, "entry")
                        .iter()
                        .filter_map(|entry| entry.get("resource").cloned())
                        .collect()
                }
            };
            let mut found = vec![];
            for candidate in candidates {
                if filter_type
                    .as_ref()
                    .is_some_and(|t| string(&candidate, "resourceType") != Some(t.as_str()))
                {
                    continue;
                }
                let concepts = if name == "section" {
                    nodes(&candidate, "code")
                } else {
                    resource_concepts(&candidate)
                };
                let mut accepted = false;
                if in_valueset {
                    for coding in concepts.iter().flat_map(codings) {
                        let Some(code) = string(&coding, "code") else {
                            continue;
                        };
                        let provider = context.terminology_provider().ok_or_else(|| {
                            invalid("Document membership requires a terminology provider")
                        })?;
                        let answer = provider
                            .validate_code_vs(
                                system.as_deref().unwrap(),
                                string(&coding, "system"),
                                code,
                                None,
                            )
                            .await
                            .map_err(|e| invalid(format!("ValueSet membership failed: {e}")))?;
                        if answer.result {
                            accepted = true;
                            break;
                        }
                    }
                } else {
                    accepted = concepts
                        .iter()
                        .any(|c| has_code(c, system.as_deref(), code.as_deref()));
                }
                if accepted {
                    found.push(candidate);
                }
            }
            if bounds.accepts(found.len()) {
                groups.extend(found);
            }
        }
        let mut values = Vec::with_capacity(groups.len());
        for node in groups {
            values.push(wrap(node, name == "section", context).await?);
        }
        Ok(EvaluationResult::new(Collection::from_values(values)))
    }
}
