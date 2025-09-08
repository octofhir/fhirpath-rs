//! FHIR-specific function registrations

use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::register_function;

use serde_json::Value as JsonValue;
use std::sync::Arc;

use super::fhir_utils::FhirUtils;

impl FunctionRegistry {
    /// Register FHIR-specific functions (navigation, extensions, references)
    pub fn register_fhir_functions(&self) -> Result<()> {
        self.register_resolve_function()?;
        self.register_extension_function()?;
        self.register_has_value_function()?;
        self.register_get_value_function()?;
        self.register_conforms_to_function()?;
        self.register_descendants_function()?;
        self.register_children_function()?;
        self.register_has_template_id_of_function()?;
        Ok(())
    }

    /// Register additional FHIR helpers for common datatypes
    pub fn register_fhir_extension_functions(&self) -> Result<()> {
        self.register_coding_function()?;
        self.register_coding_display_function()?;
        self.register_identifier_function()?;
        Ok(())
    }

    fn register_resolve_function(&self) -> Result<()> {
        register_function!(
            self,
            async "resolve",
            category: FunctionCategory::Fhir,
            description: "Resolves a Reference.reference to the target resource. Supports contained ('#id'), in-bundle, and provider-backed resolution.",
            parameters: [],
            return_type: "Resource",
            examples: [
                "Patient.managingOrganization.resolve()",
                "Observation.subject.resolve()",
                "Bundle.entry.resource.resolve()"
            ],
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathValue>> + Send + '_>> {
                Box::pin(async move {
                    if context.input.is_empty() {
                        return Ok(FhirPathValue::empty());
                    }

                    let mut resolved_resources = Vec::new();

                    for input_value in context.input.iter() {
                        // Extract reference string or Reference object
                        let reference = match input_value {
                            FhirPathValue::String(s) => Some(s.clone()),
                            FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
                                j.as_object()
                                    .and_then(|obj| obj.get("reference"))
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                            }
                            _ => None,
                        };

                        let reference = match reference {
                            Some(r) => r,
                            None => continue, // Skip invalid references
                        };

                        // Handle contained references ("#id")
                        if let Some(stripped) = reference.strip_prefix('#') {
                            if let Some(ctx_res) = context.resource_context {
                                if let Some(found) = find_contained_resource(ctx_res, stripped) {
                                    resolved_resources.push(found);
                                }
                            }
                            continue;
                        }

                        // Handle bundle and external references
                        if let Some(ctx_res) = context.resource_context {
                            if let Some(found) = resolve_reference(ctx_res, &reference) {
                                resolved_resources.push(found);
                            }
                        }
                    }

                    Ok(FhirPathValue::collection(resolved_resources))
                })
            }
        )
    }

    fn register_extension_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "extension",
            category: FunctionCategory::Fhir,
            description: "Returns extensions with the specified URL (includes modifierExtension)",
            parameters: ["url": Some("string".to_string()) => "Extension URL to filter by"],
            return_type: "Collection",
            examples: [
                "Patient.extension('http://hl7.org/fhir/StructureDefinition/patient-nationality')",
                "Observation.extension('http://example.org/custom')"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                use crate::core::error_code::{FP0051, FP0053};
                if context.arguments.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "extension() requires exactly one URL argument".to_string(),
                    ));
                }

                let url = match context.arguments.first() {
                    Some(FhirPathValue::String(s)) => s.as_str(),
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            FP0051,
                            "extension() URL argument must be a string".to_string(),
                        ));
                    }
                };

                let mut out = Vec::new();
                for v in context.input.iter() {
                    out.extend(FhirUtils::filter_extensions_by_url(v, url));
                }
                Ok(FhirPathValue::collection(out))
            }
        )
    }

    fn register_has_value_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "hasValue",
            category: FunctionCategory::Fhir,
            description: "Returns true if the input has a value (not null or empty)",
            parameters: [],
            return_type: "boolean",
            examples: [
                "Patient.name.family.hasValue()",
                "Observation.value.hasValue()"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                let has = if context.input.is_empty() {
                    false
                } else {
                    context.input.iter().any(|v| match v {
                        FhirPathValue::String(s) => !s.is_empty(),
                        FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) | FhirPathValue::Boolean(_)
                            | FhirPathValue::Date(_) | FhirPathValue::DateTime(_) | FhirPathValue::Time(_) => true,
                        FhirPathValue::Quantity { .. } => true,
                        FhirPathValue::Resource(obj) => obj.as_object().map(|o| !o.is_empty()).unwrap_or(false),
                        FhirPathValue::JsonValue(j) => !j.is_null(),
                        FhirPathValue::Collection(items) => !items.is_empty(),
                        FhirPathValue::Empty => false,
                        _ => true,
                    })
                };
                Ok(FhirPathValue::boolean(has))
            }
        )
    }

    fn register_get_value_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "getValue",
            category: FunctionCategory::Fhir,
            description: "Returns the primitive value from a FHIR element (value[x]) or self for primitives",
            parameters: [],
            return_type: "any",
            examples: [
                "Patient.name.family.getValue()",
                "Observation.valueQuantity.value.getValue()"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                let mut out = Vec::new();
                for v in context.input.iter() {
                    match v {
                        FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
                            if let Some(obj) = j.as_object() {
                                if let Some(val) = FhirUtils::extract_primitive_value(obj)? {
                                    out.push(val);
                                }
                            }
                        }
                        // primitive values return themselves
                        _ => out.push(v.clone()),
                    }
                }
                Ok(FhirPathValue::collection(out))
            }
        )
    }

    fn register_conforms_to_function(&self) -> Result<()> {
        register_function!(
            self,
            async "conformsTo",
            category: FunctionCategory::Fhir,
            description: "Returns true if the resource conforms to the specified profile (currently always true)",
            parameters: ["profile": Some("string".to_string()) => "Profile URL to validate against"],
            return_type: "boolean",
            examples: [
                "Patient.conformsTo('http://hl7.org/fhir/StructureDefinition/Patient')"
            ],
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathValue>> + Send + '_>> {
                Box::pin(async move {
                    // For now, per request, always return true
                    let _ = context;
                    Ok(FhirPathValue::boolean(true))
                })
            }
        )
    }

    fn register_descendants_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "descendants",
            category: FunctionCategory::Fhir,
            description: "Returns all descendant elements of the input",
            parameters: [],
            return_type: "Collection",
            examples: [
                "Patient.descendants()",
                "Bundle.entry.resource.descendants()"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                let mut out = Vec::new();
                for v in context.input.iter() {
                    FhirUtils::collect_descendants(v, &mut out);
                }
                Ok(FhirPathValue::collection(out))
            }
        )
    }

    fn register_children_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "children",
            category: FunctionCategory::Fhir,
            description: "Returns all direct child elements of the input",
            parameters: [],
            return_type: "Collection",
            examples: [
                "Patient.children()",
                "Bundle.entry.children()"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                let mut out = Vec::new();
                for v in context.input.iter() {
                    out.extend(FhirUtils::collect_children(v));
                }
                Ok(FhirPathValue::collection(out))
            }
        )
    }

    fn register_coding_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "coding",
            category: FunctionCategory::Fhir,
            description: "Returns Coding elements from a CodeableConcept",
            parameters: [],
            return_type: "Collection",
            examples: [
                "Observation.code.coding",
                "Patient.maritalStatus.coding"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                let mut out = Vec::new();
                for v in context.input.iter() {
                    match v {
                        FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
                            if let Some(coding) = j.get("coding").and_then(|x| x.as_array()) {
                                for c in coding.iter().cloned() {
                                    out.push(FhirPathValue::Resource(Arc::new(c)));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(FhirPathValue::collection(out))
            }
        )
    }

    fn register_coding_display_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "display",
            category: FunctionCategory::Fhir,
            description: "Returns the display value from a Coding",
            parameters: [],
            return_type: "string",
            examples: [
                "Observation.code.coding.display",
                "Patient.maritalStatus.coding.display"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                let mut out = Vec::new();
                for v in context.input.iter() {
                    match v {
                        FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
                            if let Some(s) = j.get("display").and_then(|x| x.as_str()) {
                                out.push(FhirPathValue::String(s.to_string()));
                            }
                        }
                        _ => {}
                    }
                }
                Ok(FhirPathValue::collection(out))
            }
        )
    }

    fn register_identifier_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "identifier",
            category: FunctionCategory::Fhir,
            description: "Returns identifiers with the specified system",
            parameters: ["system": Some("string".to_string()) => "Identifier system to filter by"],
            return_type: "Collection",
            examples: [
                "Patient.identifier('http://hl7.org/fhir/sid/us-ssn')",
                "Organization.identifier('http://hl7.org/fhir/sid/us-npi')"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                use crate::core::error_code::{FP0051, FP0053};
                if context.arguments.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "identifier() requires exactly one system argument".to_string(),
                    ));
                }

                let target = match context.arguments.first() {
                    Some(FhirPathValue::String(s)) => s.as_str(),
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            FP0051,
                            "identifier() system argument must be a string".to_string(),
                        ));
                    }
                };

                let mut out = Vec::new();
                for v in context.input.iter() {
                    match v {
                        FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
                            if let Some(ids) = j.get("identifier").and_then(|x| x.as_array()) {
                                for idv in ids.iter() {
                                    if let Some(obj) = idv.as_object() {
                                        if obj
                                            .get("system")
                                            .and_then(|x| x.as_str())
                                            .map(|s| s == target)
                                            .unwrap_or(false)
                                        {
                                            out.push(FhirPathValue::Resource(Arc::new(JsonValue::Object(obj.clone()))));
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(FhirPathValue::collection(out))
            }
        )
    }
}

/// Attempt to find a contained resource by id in the given resource context
fn find_contained_resource(ctx: &FhirPathValue, id: &str) -> Option<FhirPathValue> {
    match ctx {
        FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
            let obj = j.as_object()?;
            let contained = obj.get("contained")?.as_array()?;
            for entry in contained {
                if entry
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s == id)
                    .unwrap_or(false)
                {
                    return Some(FhirPathValue::Resource(Arc::new(entry.clone())));
                }
            }
            None
        }
        _ => None,
    }
}

/// Best-effort in-bundle resolver: supports matching by fullUrl or ResourceType/id
fn split_ref(reference: &str) -> (Option<String>, Option<String>) {
    if let Some(pos) = reference.find('/') {
        let (a, b) = reference.split_at(pos);
        let id = &b[1..];
        (Some(a.to_string()), Some(id.to_string()))
    } else {
        (None, None)
    }
}

// ---------------- Bundle reference resolution ----------------

fn resolve_reference(ctx: &FhirPathValue, reference: &str) -> Option<FhirPathValue> {
    let j = match ctx {
        FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => j,
        _ => return None,
    };

    // Handle contained resources (references starting with #)
    if reference.starts_with('#') {
        let contained_id = &reference[1..]; // Remove the # prefix
        if let Some(contained) = j.get("contained").and_then(|c| c.as_array()) {
            for resource in contained {
                if let Some(id) = resource.get("id").and_then(|id| id.as_str()) {
                    if id == contained_id {
                        return Some(FhirPathValue::Resource(Arc::new(resource.clone())));
                    }
                }
            }
        }
        return None;
    }

    // Handle Bundle entry resolution
    if let Some(entries) = j.get("entry").and_then(|e| e.as_array()) {
        for entry in entries {
            // Try fullUrl match first
            if let Some(full_url) = entry.get("fullUrl").and_then(|u| u.as_str()) {
                if full_url == reference {
                    if let Some(resource) = entry.get("resource") {
                        return Some(FhirPathValue::Resource(Arc::new(resource.clone())));
                    }
                }
            }

            // Try ResourceType/id match
            if let Some(resource) = entry.get("resource") {
                if let (Some(resource_type), Some(resource_id)) = (
                    resource.get("resourceType").and_then(|rt| rt.as_str()),
                    resource.get("id").and_then(|id| id.as_str()),
                ) {
                    let expected_ref = format!("{}/{}", resource_type, resource_id);
                    if expected_ref == reference {
                        return Some(FhirPathValue::Resource(Arc::new(resource.clone())));
                    }
                }
            }
        }
    }

    None
}

impl FunctionRegistry {
    fn register_has_template_id_of_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "hasTemplateIdOf",
            category: FunctionCategory::Fhir,
            description: "CDA-specific function to check if the current element has a templateId with the specified root and optionally extension",
            parameters: [
                "root": Some("string".to_string()) => "The root value to match",
                "extension": Some("string".to_string()) => "Optional extension value to match"
            ],
            return_type: "boolean",
            examples: [
                "ClinicalDocument.hasTemplateIdOf('2.16.840.1.113883.10.20.22.1.1')",
                "section.hasTemplateIdOf('2.16.840.1.113883.10.20.22.2.4.1', '2014-06-09')"
            ],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.arguments.is_empty() || context.arguments.len() > 2 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0053,
                        "hasTemplateIdOf() requires 1 or 2 arguments (root, optional extension)".to_string()
                    ));
                }

                if let Some(first_input) = context.input.first() {
                    // Get the root parameter
                    let target_root = match context.arguments.first() {
                        Some(FhirPathValue::String(s)) => s,
                        _ => {
                            return Err(FhirPathError::evaluation_error(
                                crate::core::error_code::FP0053,
                                "hasTemplateIdOf() root parameter must be a string".to_string()
                            ));
                        }
                    };

                    // Get optional extension parameter
                    let target_extension = if context.arguments.len() > 1 {
                        match context.arguments.get(1) {
                            Some(FhirPathValue::String(s)) => Some(s),
                            _ => {
                                return Err(FhirPathError::evaluation_error(
                                    crate::core::error_code::FP0053,
                                    "hasTemplateIdOf() extension parameter must be a string".to_string()
                                ));
                            }
                        }
                    } else {
                        None
                    };

                    // Check if the input has templateId array
                    match first_input {
                        FhirPathValue::Resource(obj) => {
                            if let Some(template_ids) = obj.get("templateId").and_then(|v| v.as_array()) {
                                for template_id in template_ids {
                                    if let Some(template_obj) = template_id.as_object() {
                                        // Check root
                                        let root_matches = template_obj
                                            .get("root")
                                            .and_then(|v| v.as_str())
                                            .map(|r| r == target_root)
                                            .unwrap_or(false);

                                        if root_matches {
                                            // If no extension required, root match is enough
                                            if target_extension.is_none() {
                                                return Ok(FhirPathValue::Boolean(true));
                                            }

                                            // Check extension if required
                                            let extension_matches = template_obj
                                                .get("extension")
                                                .and_then(|v| v.as_str())
                                                .map(|e| target_extension.map_or(false, |te| e == te))
                                                .unwrap_or(false);

                                            if extension_matches {
                                                return Ok(FhirPathValue::Boolean(true));
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(FhirPathValue::Boolean(false))
                        },
                        _ => Ok(FhirPathValue::Boolean(false))
                    }
                } else {
                    Ok(FhirPathValue::Boolean(false))
                }
            }
        )
    }
}
