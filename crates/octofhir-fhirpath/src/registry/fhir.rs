//! FHIR-specific function registrations

use crate::core::{FhirPathError, FhirPathValue, Result};
use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::register_function;

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Default)]
struct PerformanceMetrics {
    input_processing_us: u64,
    context_access_us: u64,
    batch_optimization_us: u64,
    reference_extraction_us: u64,
    resolution_us: u64,
    total_us: u64,
}

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
        self.register_hasTemplateIdOf_function()?;
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
                    let total_start = std::time::Instant::now();
                    use crate::core::error_code::{FP0053, FP0051};
                    
                    // Performance profiling structure
                    let mut metrics = PerformanceMetrics {
                        input_processing_us: 0,
                        context_access_us: 0,
                        batch_optimization_us: 0,
                        reference_extraction_us: 0,
                        resolution_us: 0,
                        total_us: 0,
                    };
                    

                    // Phase 1: Input processing
                    let phase_start = std::time::Instant::now();
                    if context.input.is_empty() {
                        return Ok(Vec::new());
                    }
                    let input_len = context.input.len();
                    metrics.input_processing_us = phase_start.elapsed().as_micros() as u64;
                    
                    let mut resolved_resources = Vec::new();

                    // Phase 2: Context access and Bundle detection
                    let phase_start = std::time::Instant::now();
                    if let Some(ctx_res) = context.resource_context {
                        if let FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) = ctx_res {
                            if let Some(obj) = j.as_object() {
                                if obj.get("resourceType").and_then(|v| v.as_str()) == Some("Bundle") {
                                    metrics.context_access_us = phase_start.elapsed().as_micros() as u64;
                                    
                                    // Phase 3: Batch optimization
                                    let phase_start = std::time::Instant::now();
                                    let result = resolve_batch_optimized_with_metrics(context.input, obj, &mut metrics);
                                    metrics.batch_optimization_us = phase_start.elapsed().as_micros() as u64;
                                    
                                    // Final metrics
                                    metrics.total_us = total_start.elapsed().as_micros() as u64;
                                    print_performance_metrics(&metrics, input_len);
                                    return Ok(result);
                                }
                            }
                        }
                    }
                    metrics.context_access_us = phase_start.elapsed().as_micros() as u64;
                    
                    // Fallback: process each input reference individually
                    for input_value in context.input.iter() {
                        // Extract reference string or Reference object
                        let (reference_opt, _ref_obj_opt): (Option<String>, Option<&serde_json::Map<String, serde_json::Value>>) = match input_value {
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
                                // Skip invalid references but don't fail the entire operation
                                continue;
                            }
                        };

                        // 1) Contained reference: "#id"
                        if let Some(stripped) = reference.strip_prefix('#') {
                            if let Some(ctx_res) = context.resource_context {
                                if let Some(found) = find_contained_resource(ctx_res, stripped) {
                                    resolved_resources.push(found);
                                }
                            }
                            continue;
                        }

                        // 2) In-bundle resolution (best effort, index-backed)
                        if let Some(ctx_res) = context.resource_context {
                            if let Some(found) = resolve_in_bundle_indexed(ctx_res, &reference) {
                                resolved_resources.push(found);
                            }
                        }
                    }

                    metrics.total_us = total_start.elapsed().as_micros() as u64;
                    print_performance_metrics(&metrics, input_len);
                    Ok(resolved_resources)
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

// ---------------- Bundle reference resolution ----------------

/// Resolves FHIR references within Bundle resources using adaptive indexing strategies
fn resolve_in_bundle_indexed(ctx: &FhirPathValue, reference: &str) -> Option<FhirPathValue> {
    let j = match ctx {
        FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => j,
        _ => return None,
    };
    let obj = j.as_object()?;
    if obj.get("resourceType").and_then(|v| v.as_str()) != Some("Bundle") {
        return None;
    }

    let entries = obj.get("entry")?.as_array()?;
    
    // Use linear scan for small bundles to avoid indexing overhead
    if entries.len() < 50 {
        return resolve_linear_scan(entries, reference);
    }
    
    // Use indexed lookup for larger bundles to improve O(n) to O(1) access
    resolve_with_index(entries, reference)
}

/// Linear search through Bundle entries for reference resolution.
/// Efficient for small bundles as it avoids indexing overhead.
#[inline(always)]
fn resolve_linear_scan(entries: &[JsonValue], reference: &str) -> Option<FhirPathValue> {
    // Split reference once upfront
    let (rt_opt, id_opt) = split_ref(reference);
    
    for entry in entries {
        // Check fullUrl first (most common case)
        if let Some(full_url) = entry.get("fullUrl").and_then(|v| v.as_str()) {
            if full_url == reference {
                return entry.get("resource").cloned().map(FhirPathValue::Resource);
            }
        }
        
        // Check ResourceType/ID pattern
        if let (Some(rt), Some(id)) = (&rt_opt, &id_opt) {
            if let Some(resource) = entry.get("resource") {
                if let (Some(res_type), Some(res_id)) = (
                    resource.get("resourceType").and_then(|v| v.as_str()),
                    resource.get("id").and_then(|v| v.as_str()),
                ) {
                    if res_type == rt && res_id == id {
                        return Some(FhirPathValue::Resource(resource.clone()));
                    }
                }
            }
        }
    }
    
    None
}

/// Global cache with better performance characteristics for index storage
static BUNDLE_CACHE: Lazy<RwLock<HashMap<u64, Arc<BundleIndex>>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Index structure for Bundle entry lookup by URL and ResourceType/ID
#[derive(Debug)]
struct BundleIndex {
    /// Maps fullUrl strings to Bundle entry indices
    by_full_url: HashMap<Box<str>, usize>,
    /// Maps (resourceType, id) pairs to Bundle entry indices
    by_type_id: HashMap<(Box<str>, Box<str>), usize>,
}

/// Resolves references using a cached index for larger Bundle resources.
/// Uses global caching to avoid repeated index construction.
fn resolve_with_index(entries: &[JsonValue], reference: &str) -> Option<FhirPathValue> {
    // Create hash for cache key
    let mut hasher = DefaultHasher::new();
    entries.len().hash(&mut hasher);
    // Hash sample of entries for cache key
    for (i, entry) in entries.iter().enumerate() {
        if i < 3 || i >= entries.len().saturating_sub(3) {
            entry.get("fullUrl").hash(&mut hasher);
        }
    }
    let cache_key = hasher.finish();
    
    // Try to get cached index
    let index = {
        let cache = BUNDLE_CACHE.read();
        cache.get(&cache_key).cloned()
    };
    
    let index = if let Some(cached_index) = index {
        cached_index
    } else {
        // Build new index
        let mut by_full_url = HashMap::with_capacity(entries.len());
        let mut by_type_id = HashMap::with_capacity(entries.len());
        
        for (i, entry) in entries.iter().enumerate() {
            if let Some(full_url) = entry.get("fullUrl").and_then(|v| v.as_str()) {
                by_full_url.insert(full_url.into(), i);
            }
            
            if let Some(resource) = entry.get("resource") {
                if let (Some(rt), Some(id)) = (
                    resource.get("resourceType").and_then(|v| v.as_str()),
                    resource.get("id").and_then(|v| v.as_str()),
                ) {
                    by_type_id.insert((rt.into(), id.into()), i);
                }
            }
        }
        
        let new_index = Arc::new(BundleIndex { by_full_url, by_type_id });
        
        // Cache the index
        {
            let mut cache = BUNDLE_CACHE.write();
            if cache.len() > 10 {
                cache.clear();
            }
            cache.insert(cache_key, new_index.clone());
        }
        
        new_index
    };
    
    // Perform lookup
    if let Some(&entry_idx) = index.by_full_url.get(reference) {
        return entries.get(entry_idx)
            .and_then(|e| e.get("resource"))
            .cloned()
            .map(FhirPathValue::Resource);
    }
    
    if let (Some(rt), Some(id)) = split_ref(reference) {
        if let Some(&entry_idx) = index.by_type_id.get(&(rt.as_str().into(), id.as_str().into())) {
            return entries.get(entry_idx)
                .and_then(|e| e.get("resource"))
                .cloned()
                .map(FhirPathValue::Resource);
        }
    }
    
    None
}

/// Batch resolver that processes multiple references in a single pass.
/// Avoids repeated JSON traversals and index building for multiple resolve operations.
fn resolve_batch_optimized(input_refs: &[FhirPathValue], bundle_obj: &serde_json::Map<String, serde_json::Value>) -> Vec<FhirPathValue> {
    let entries = match bundle_obj.get("entry").and_then(|v| v.as_array()) {
        Some(entries) => entries,
        None => return Vec::new(),
    };
    
    // Extract all reference strings first
    let mut references = Vec::with_capacity(input_refs.len());
    for input_value in input_refs {
        if let Some(reference) = extract_reference_string(input_value) {
            references.push(reference);
        }
    }
    
    if references.is_empty() {
        return Vec::new();
    }
    
    let mut resolved = Vec::with_capacity(references.len());
    
    // For small bundles or small reference counts: use linear scan
    if entries.len() < 100 || references.len() < 10 {
        for reference in references {
            if let Some(resource) = resolve_linear_scan(entries, &reference) {
                resolved.push(resource);
            }
        }
        return resolved;
    }
    
    // For larger operations: build index once and resolve all
    let mut by_full_url = HashMap::with_capacity(entries.len());
    let mut by_type_id = HashMap::with_capacity(entries.len());
    
    // Build index by iterating through all entries
    for (i, entry) in entries.iter().enumerate() {
        if let Some(full_url) = entry.get("fullUrl").and_then(|v| v.as_str()) {
            by_full_url.insert(full_url, i);
        }
        
        if let Some(resource) = entry.get("resource") {
            if let (Some(rt), Some(id)) = (
                resource.get("resourceType").and_then(|v| v.as_str()),
                resource.get("id").and_then(|v| v.as_str()),
            ) {
                by_type_id.insert((rt, id), i);
            }
        }
    }
    
    // Resolve all references using the index
    for reference in references {
        let entry_idx = if let Some(&idx) = by_full_url.get(reference.as_str()) {
            Some(idx)
        } else if let (Some(rt), Some(id)) = split_ref(&reference) {
            by_type_id.get(&(rt.as_str(), id.as_str())).copied()
        } else {
            None
        };
        
        if let Some(idx) = entry_idx {
            if let Some(resource) = entries.get(idx).and_then(|e| e.get("resource")) {
                resolved.push(FhirPathValue::Resource(resource.clone()));
            }
        }
    }
    
    resolved
}

/// Prints performance metrics for resolve operations (only for slow operations)
fn print_performance_metrics(metrics: &PerformanceMetrics, input_count: usize) {
    // Only print metrics for slow operations (>5ms) or when debugging
    if metrics.total_us > 5000 || std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
        eprintln!("🔍 RESOLVE PERFORMANCE METRICS ({} references):", input_count);
        eprintln!("  Input Processing:    {:>6}μs", metrics.input_processing_us);
        eprintln!("  Context Access:      {:>6}μs", metrics.context_access_us);
        eprintln!("  Reference Extraction:{:>6}μs", metrics.reference_extraction_us);
        eprintln!("  Bundle Resolution:   {:>6}μs", metrics.resolution_us);
        eprintln!("  Batch Optimization:  {:>6}μs", metrics.batch_optimization_us);
        eprintln!("  TOTAL TIME:          {:>6}μs", metrics.total_us);
        eprintln!("  Per Reference:       {:>6}μs", metrics.total_us / (input_count as u64).max(1));
        eprintln!();
    }
}

/// Performance-instrumented batch resolver with detailed metrics
fn resolve_batch_optimized_with_metrics(
    input_refs: &[FhirPathValue], 
    bundle_obj: &serde_json::Map<String, serde_json::Value>,
    metrics: &mut PerformanceMetrics
) -> Vec<FhirPathValue> {
    let phase_start = std::time::Instant::now();
    let entries = match bundle_obj.get("entry").and_then(|v| v.as_array()) {
        Some(entries) => entries,
        None => return Vec::new(),
    };
    
    // Extract all reference strings first
    let mut references = Vec::with_capacity(input_refs.len());
    for input_value in input_refs {
        if let Some(reference) = extract_reference_string(input_value) {
            references.push(reference);
        }
    }
    metrics.reference_extraction_us = phase_start.elapsed().as_micros() as u64;
    
    if references.is_empty() {
        return Vec::new();
    }
    
    // Resolution phase with detailed timing
    let phase_start = std::time::Instant::now();
    let mut resolved = Vec::with_capacity(references.len());
    
    // For small bundles or small reference counts: use linear scan
    if entries.len() < 100 || references.len() < 10 {
        if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
            eprintln!("🔍 Using LINEAR SCAN: {} entries, {} references", entries.len(), references.len());
        }
        for reference in references {
            if let Some(resource) = resolve_linear_scan(entries, &reference) {
                resolved.push(resource);
            }
        }
    } else {
        let debug_perf = std::env::var("FHIRPATH_DEBUG_PERF").is_ok();
        if debug_perf {
            eprintln!("🔍 Using INDEXED LOOKUP: {} entries, {} references", entries.len(), references.len());
        }
        
        // Create hash of bundle entries for caching
        let cache_start = std::time::Instant::now();
        let mut hasher = DefaultHasher::new();
        // Hash first and last few entries for cache key (efficient approximation)
        for (i, entry) in entries.iter().enumerate() {
            if i < 5 || i >= entries.len().saturating_sub(5) {
                entry.get("fullUrl").hash(&mut hasher);
                if let Some(resource) = entry.get("resource") {
                    resource.get("resourceType").hash(&mut hasher);
                    resource.get("id").hash(&mut hasher);
                }
            }
        }
        entries.len().hash(&mut hasher);
        let cache_key = hasher.finish();
        
        if debug_perf {
            let cache_time = cache_start.elapsed().as_micros();
            eprintln!("🔍 Cache key generation: {}μs", cache_time);
        }
        
        // Try to get cached index first
        let index_lookup_start = std::time::Instant::now();
        let index = {
            let cache = BUNDLE_CACHE.read();
            cache.get(&cache_key).cloned()
        };
        
        let index = if let Some(cached_index) = index {
            if debug_perf {
                let lookup_time = index_lookup_start.elapsed().as_micros();
                eprintln!("🔍 Cache HIT - Index lookup: {}μs", lookup_time);
            }
            cached_index
        } else {
            if debug_perf {
                eprintln!("🔍 Cache MISS - Building index...");
            }
            let index_build_start = std::time::Instant::now();
            
            let mut by_full_url = HashMap::with_capacity(entries.len());
            let mut by_type_id = HashMap::with_capacity(entries.len());
            
            // Build index by iterating through all entries
            for (i, entry) in entries.iter().enumerate() {
                if let Some(full_url) = entry.get("fullUrl").and_then(|v| v.as_str()) {
                    by_full_url.insert(full_url.to_string(), i);
                }
                
                if let Some(resource) = entry.get("resource") {
                    if let (Some(rt), Some(id)) = (
                        resource.get("resourceType").and_then(|v| v.as_str()),
                        resource.get("id").and_then(|v| v.as_str()),
                    ) {
                        by_type_id.insert((rt.to_string(), id.to_string()), i);
                    }
                }
            }
            
            let new_index = Arc::new(BundleIndex {
                by_full_url: by_full_url.into_iter().map(|(k, v)| (k.into_boxed_str(), v)).collect(),
                by_type_id: by_type_id.into_iter().map(|((rt, id), v)| ((rt.into_boxed_str(), id.into_boxed_str()), v)).collect(),
            });
            
            if debug_perf {
                let index_time = index_build_start.elapsed().as_micros();
                eprintln!("🔍 Index build time: {}μs", index_time);
            }
            
            // Cache the new index
            let mut cache = BUNDLE_CACHE.write();
            if cache.len() > 20 { // Limit cache size
                cache.clear();
            }
            cache.insert(cache_key, new_index.clone());
            
            new_index
        };
        
        // Resolve all references using the cached index
        let lookup_start = std::time::Instant::now();
        for reference in references {
            let entry_idx = if let Some(&idx) = index.by_full_url.get(reference.as_str()) {
                Some(idx)
            } else if let (Some(rt), Some(id)) = split_ref(&reference) {
                index.by_type_id.get(&(rt.as_str().into(), id.as_str().into())).copied()
            } else {
                None
            };
            
            if let Some(idx) = entry_idx {
                if let Some(resource) = entries.get(idx).and_then(|e| e.get("resource")) {
                    resolved.push(FhirPathValue::Resource(resource.clone()));
                }
            }
        }
        
        if debug_perf {
            let lookup_time = lookup_start.elapsed().as_micros();
            eprintln!("🔍 Lookup time: {}μs", lookup_time);
        }
    }
    
    metrics.resolution_us = phase_start.elapsed().as_micros() as u64;
    resolved
}

/// Extracts reference string from various FhirPathValue types
#[inline]
fn extract_reference_string(input_value: &FhirPathValue) -> Option<String> {
    match input_value {
        FhirPathValue::String(s) => Some(s.clone()),
        FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
            j.as_object()
                .and_then(|obj| obj.get("reference"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        }
        _ => None,
    }
}


impl FunctionRegistry {
    fn register_hasTemplateIdOf_function(&self) -> Result<()> {
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.arguments.is_empty() || context.arguments.len() > 2 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0053,
                        "hasTemplateIdOf() requires 1 or 2 arguments (root, optional extension)".to_string()
                    ));
                }

                if context.input.len() != 1 {
                    return Ok(vec![FhirPathValue::Boolean(false)]);
                }

                // Get the root parameter
                let target_root = match &context.arguments[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0053,
                            "hasTemplateIdOf() root parameter must be a string".to_string()
                        ));
                    }
                };

                // Get optional extension parameter
                let target_extension = if context.arguments.len() > 1 {
                    match &context.arguments[1] {
                        FhirPathValue::String(s) => Some(s),
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
                match &context.input[0] {
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
                                            return Ok(vec![FhirPathValue::Boolean(true)]);
                                        }
                                        
                                        // Check extension if required
                                        let extension_matches = template_obj
                                            .get("extension")
                                            .and_then(|v| v.as_str())
                                            .map(|e| target_extension.map_or(false, |te| e == te))
                                            .unwrap_or(false);

                                        if extension_matches {
                                            return Ok(vec![FhirPathValue::Boolean(true)]);
                                        }
                                    }
                                }
                            }
                        }
                        Ok(vec![FhirPathValue::Boolean(false)])
                    },
                    _ => Ok(vec![FhirPathValue::Boolean(false)])
                }
            }
        )
    }
}
