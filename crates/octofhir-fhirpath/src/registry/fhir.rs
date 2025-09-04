//! FHIR-specific function registrations

use crate::core::{FhirPathError, FhirPathValue, Result};
use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::register_function;

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use lru::LruCache;

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
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FhirPathValue>>> + Send + '_>> {
                Box::pin(async move {
                    use crate::core::error_code::{FP0053, FP0051};

                    if context.input.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "resolve() requires a singleton input".to_string(),
                        ));
                    }

                    // Extract reference string or Reference object
                    let (reference_opt, ref_obj_opt): (Option<String>, Option<&serde_json::Map<String, serde_json::Value>>) = match &context.input[0] {
                        FhirPathValue::String(s) => (Some(s.clone()), None),
                        FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
                            if let Some(obj) = j.as_object() {
                                if let Some(r) = obj.get("reference").and_then(|v| v.as_str()) {
                                    (Some(r.to_string()), Some(obj))
                                } else {
                                    (None, Some(obj))
                                }
                            } else {
                                (None, None)
                            }
                        }
                        _ => (None, None),
                    };

                    let reference = match reference_opt {
                        Some(r) => r,
                        None => {
                            return Err(FhirPathError::evaluation_error(
                                FP0051,
                                "resolve() expects a Reference or reference string".to_string(),
                            ));
                        }
                    };

                    // 1) Contained reference: "#id"
                    if let Some(stripped) = reference.strip_prefix('#') {
                        if let Some(ctx_res) = context.resource_context {
                            if let Some(found) = find_contained_resource(ctx_res, stripped) {
                                return Ok(vec![found]);
                            }
                        }
                        // Not found -> empty per spec
                        return Ok(Vec::new());
                    }

                    // 2) In-bundle resolution (best effort, index-backed)
                    if let Some(ctx_res) = context.resource_context {
                        if let Some(found) = resolve_in_bundle_indexed(ctx_res, &reference) {
                            return Ok(vec![found]);
                        }
                    }

                    // 3) Not found -> empty per spec
                    Ok(Vec::new())
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                use crate::core::error_code::{FP0051, FP0053};
                if context.arguments.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "extension() requires exactly one URL argument".to_string(),
                    ));
                }

                let url = match &context.arguments[0] {
                    FhirPathValue::String(s) => s.as_str(),
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
                Ok(out)
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
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
                Ok(vec![FhirPathValue::boolean(has)])
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
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
                Ok(out)
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
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FhirPathValue>>> + Send + '_>> {
                Box::pin(async move {
                    // For now, per request, always return true
                    let _ = context;
                    Ok(vec![FhirPathValue::boolean(true)])
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                let mut out = Vec::new();
                for v in context.input.iter() {
                    FhirUtils::collect_descendants(v, &mut out);
                }
                Ok(out)
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                let mut out = Vec::new();
                for v in context.input.iter() {
                    out.extend(FhirUtils::collect_children(v));
                }
                Ok(out)
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                let mut out = Vec::new();
                for v in context.input.iter() {
                    match v {
                        FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
                            if let Some(coding) = j.get("coding").and_then(|x| x.as_array()) {
                                for c in coding.iter().cloned() {
                                    out.push(FhirPathValue::Resource(c));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(out)
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
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
                Ok(out)
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                use crate::core::error_code::{FP0051, FP0053};
                if context.arguments.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "identifier() requires exactly one system argument".to_string(),
                    ));
                }

                let target = match &context.arguments[0] {
                    FhirPathValue::String(s) => s.as_str(),
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
                                            out.push(FhirPathValue::Resource(JsonValue::Object(obj.clone())));
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(out)
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
                    return Some(FhirPathValue::Resource(entry.clone()));
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

// ---------------- Bundle index for fast lookups ----------------

#[derive(Debug)]
struct BundleIndex {
    by_full_url: HashMap<String, usize>,
    by_type_id: HashMap<(String, String), usize>,
    entries_len: usize,
}

static BUNDLE_INDEX_CACHE: Lazy<Mutex<LruCache<usize, BundleIndex>>> = Lazy::new(|| Mutex::new(LruCache::new(std::num::NonZeroUsize::new(8).unwrap())));

fn resolve_in_bundle_indexed(ctx: &FhirPathValue, reference: &str) -> Option<FhirPathValue> {
    let j = match ctx {
        FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => j,
        _ => return None,
    };
    let obj = j.as_object()?;
    if obj.get("resourceType").and_then(|v| v.as_str()) != Some("Bundle") {
        return None;
    }

    let key = (j as *const JsonValue) as usize;
    // First: try cached index under lock
    {
        let mut cache = BUNDLE_INDEX_CACHE.lock();
        if let Some(idx) = cache.get(&key) {
            if let Some(&entry_idx) = idx.by_full_url.get(reference) {
                return bundle_entry_resource_by_index(obj, entry_idx).map(FhirPathValue::Resource);
            }
            if let (Some(rt), Some(id)) = split_ref(reference) {
                if let Some(&entry_idx) = idx.by_type_id.get(&(rt, id)) {
                    return bundle_entry_resource_by_index(obj, entry_idx).map(FhirPathValue::Resource);
                }
            }
            return None;
        }
    }

    // Build index once and cache
    let entries = obj.get("entry")?.as_array()?;
    let mut by_full_url = HashMap::with_capacity(entries.len());
    let mut by_type_id = HashMap::with_capacity(entries.len());

    for (i, e) in entries.iter().enumerate() {
        if let Some(full_url) = e.get("fullUrl").and_then(|v| v.as_str()) {
            by_full_url.entry(full_url.to_string()).or_insert(i);
        }
        if let Some(res) = e.get("resource") {
            if let (Some(rt), Some(id)) = (
                res.get("resourceType").and_then(|v| v.as_str()),
                res.get("id").and_then(|v| v.as_str()),
            ) {
                by_type_id.entry((rt.to_string(), id.to_string())).or_insert(i);
            }
        }
    }

    let built = BundleIndex { by_full_url, by_type_id, entries_len: entries.len() };
    // lookup after building
    let (rt_opt, id_opt) = split_ref(reference);
    let res = if let Some(&entry_idx) = built.by_full_url.get(reference) {
        bundle_entry_resource_by_index(obj, entry_idx).map(FhirPathValue::Resource)
    } else if let (Some(rt), Some(id)) = (rt_opt, id_opt) {
        if let Some(&entry_idx) = built.by_type_id.get(&(rt.clone(), id.clone())) {
            bundle_entry_resource_by_index(obj, entry_idx).map(FhirPathValue::Resource)
        } else { None }
    } else { None };

    // Insert into cache under lock
    {
        let mut cache = BUNDLE_INDEX_CACHE.lock();
        cache.put(key, built);
    }
    res
}

fn bundle_entry_resource_by_index(bundle_obj: &serde_json::Map<String, JsonValue>, idx: usize) -> Option<JsonValue> {
    let entries = bundle_obj.get("entry")?.as_array()?;
    let e = entries.get(idx)?;
    e.get("resource").cloned()
}
