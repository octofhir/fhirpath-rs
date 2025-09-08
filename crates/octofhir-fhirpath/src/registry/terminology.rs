//! Terminology-related FHIRPath functions (async)

use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::core::error_code::{FP0051, FP0053, FP0200};
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::register_function;

use super::terminology_utils::{Coding, TerminologyUtils};

impl FunctionRegistry {
    /// Register terminology functions. These functions depend on a TerminologyService
    /// provided via the evaluation context. If not available, they return an error.
    pub fn register_terminology_functions(&self) -> Result<()> {
        self.register_member_of_function()?;
        self.register_translate_function()?;
        self.register_validate_code_function()?;
        self.register_subsumes_function()?;
        self.register_designation_function()?;
        self.register_property_function()?;

        // Register %terminologies built-in functions
        self.register_expand_function()?;
        self.register_validate_vs_function()?;

        Ok(())
    }

    fn register_member_of_function(&self) -> Result<()> {
        register_function!(
            self,
            async "memberOf",
            category: FunctionCategory::Terminology,
            description: "Returns true if the input coding/code is a member of the specified ValueSet",
            parameters: ["valueset": Some("string".to_string()) => "ValueSet URL or identifier"],
            return_type: "boolean",
            examples: [
                "Observation.code.coding.memberOf('http://hl7.org/fhir/ValueSet/observation-codes')",
                "Patient.maritalStatus.coding.memberOf('http://hl7.org/fhir/ValueSet/marital-status')"
            ],
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathValue>> + Send + '_>> {
                Box::pin(async move {
                    if context.input.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "memberOf() requires a singleton input".to_string(),
                        ));
                    }
                    if context.arguments.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "memberOf() requires exactly one argument (ValueSet URL)".to_string(),
                        ));
                    }
                    let vs = match context.arguments.first() {
                        Some(FhirPathValue::String(s)) => s.as_str(),
                        _ => {
                            return Err(FhirPathError::evaluation_error(
                                FP0051,
                                "memberOf() ValueSet argument must be a string".to_string(),
                            ));
                        }
                    };

                    let coding = TerminologyUtils::extract_coding(context.input.first().ok_or_else(|| {
                        FhirPathError::evaluation_error(
                            FP0053,
                            "memberOf() requires a singleton input".to_string(),
                        )
                    })?)?;
                    let coded_value = TerminologyUtils::coding_to_value(&coding);

                    let svc = match context.terminology {
                        Some(s) => s,
                        None => {
                            return Err(FhirPathError::evaluation_error(
                                FP0200,
                                "Terminology service is not configured".to_string(),
                            ));
                        }
                    };

                    // Use validateVS semantics: returns a collection that should indicate validity.
                    let coll = svc.validate_vs(vs, &coded_value, None).await?;
                    // Interpret first boolean value if present, otherwise false
                    let is_member = coll.iter().find_map(|v| match v {
                        FhirPathValue::Boolean(b) => Some(*b),
                        _ => None,
                    }).unwrap_or(false);
                    Ok(FhirPathValue::boolean(is_member))
                })
            }
        )
    }

    fn register_translate_function(&self) -> Result<()> {
        register_function!(
            self,
            async "translate",
            category: FunctionCategory::Terminology,
            description: "Translates the input coding using the specified ConceptMap",
            parameters: [
                "conceptmap": Some("string".to_string()) => "ConceptMap URL or identifier"
            ],
            return_type: "collection",
            examples: [
                "Observation.code.coding.translate('http://example.org/ConceptMap/lab-to-loinc')"
            ],
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathValue>> + Send + '_>> {
                Box::pin(async move {
                    if context.input.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "translate() requires a singleton input".to_string(),
                        ));
                    }
                    if context.arguments.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "translate() requires exactly one argument (ConceptMap URL)".to_string(),
                        ));
                    }
                    let cm = match context.arguments.first() {
                        Some(FhirPathValue::String(s)) => s.as_str(),
                        _ => {
                            return Err(FhirPathError::evaluation_error(
                                FP0051,
                                "translate() ConceptMap argument must be a string".to_string(),
                            ));
                        }
                    };
                    let coding = TerminologyUtils::extract_coding(context.input.first().ok_or_else(|| {
                        FhirPathError::evaluation_error(
                            FP0053,
                            "translate() requires a singleton input".to_string(),
                        )
                    })?)?;
                    let coded_value = TerminologyUtils::coding_to_value(&coding);

                    let svc = match context.terminology {
                        Some(s) => s,
                        None => {
                            return Err(FhirPathError::evaluation_error(
                                FP0200,
                                "Terminology service is not configured".to_string(),
                            ));
                        }
                    };
                    let out = svc.translate(cm, &coded_value, None).await?;
                    Ok(FhirPathValue::collection(out.into_vec()))
                })
            }
        )
    }

    fn register_validate_code_function(&self) -> Result<()> {
        register_function!(
            self,
            async "validateCode",
            category: FunctionCategory::Terminology,
            description: "Validates that a code is valid within the specified code system",
            parameters: [
                "system": Some("string".to_string()) => "Code system URL",
                "code": Some("string".to_string()) => "Code to validate (optional if input is a coding)"
            ],
            return_type: "boolean",
            examples: [
                "'male'.validateCode('http://hl7.org/fhir/administrative-gender', 'male')",
                "Patient.gender.validateCode('http://hl7.org/fhir/administrative-gender')"
            ],
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathValue>> + Send + '_>> {
                Box::pin(async move {
                    if context.input.len() > 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "validateCode() expects at most one input value".to_string(),
                        ));
                    }
                    if context.arguments.is_empty() || context.arguments.len() > 2 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "validateCode() requires 1 or 2 arguments: system and optional code".to_string(),
                        ));
                    }
                    let system = match context.arguments.first() {
                        Some(FhirPathValue::String(s)) => s.as_str(),
                        _ => {
                            return Err(FhirPathError::evaluation_error(
                                FP0051,
                                "validateCode() system argument must be a string".to_string(),
                            ));
                        }
                    };

                    let coding: Coding = if context.arguments.len() > 1 {
                        let code = match context.arguments.get(1) {
                            Some(FhirPathValue::String(s)) => s.as_str(),
                            _ => {
                                return Err(FhirPathError::evaluation_error(
                                    FP0051,
                                    "validateCode() code argument must be a string".to_string(),
                                ));
                            }
                        };
                        Coding::new(system, code)
                    } else if !context.input.is_empty() {
                        TerminologyUtils::extract_coding(&context.input)?
                    } else {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "validateCode() requires a code argument or coding input".to_string(),
                        ));
                    };

                    let svc = match context.terminology {
                        Some(s) => s,
                        None => {
                            return Err(FhirPathError::evaluation_error(
                                FP0200,
                                "Terminology service is not configured".to_string(),
                            ));
                        }
                    };

                    // Best-effort using lookup; treat non-empty result as valid
                    let coded_value = TerminologyUtils::coding_to_value(&coding);
                    let mut params = std::collections::HashMap::new();
                    params.insert("system".to_string(), system.to_string());
                    let result = svc.lookup(&coded_value, Some(params)).await?;
                    let is_valid = !result.is_empty();
                    Ok(FhirPathValue::boolean(is_valid))
                })
            }
        )
    }

    fn register_subsumes_function(&self) -> Result<()> {
        register_function!(
            self,
            async "subsumes",
            category: FunctionCategory::Terminology,
            description: "Returns true if the first code subsumes (is more general than) the second code",
            parameters: [
                "codeA": Some("coding".to_string()) => "The potentially subsuming code",
                "codeB": Some("coding".to_string()) => "The potentially subsumed code"
            ],
            return_type: "boolean",
            examples: [
                "subsumes(ParentCode.coding, ChildCode.coding)",
                "Condition.code.coding.subsumes(MoreSpecificCondition.code.coding)"
            ],
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathValue>> + Send + '_>> {
                Box::pin(async move {
                    if context.arguments.len() != 2 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "subsumes() requires exactly two arguments".to_string(),
                        ));
                    }

                    let coding_a = TerminologyUtils::extract_coding(context.arguments.first().ok_or_else(|| FhirPathError::evaluation_error(FP0053, "subsumes() requires first argument".to_string()))?)?;
                    let coding_b = TerminologyUtils::extract_coding(context.arguments.get(1).ok_or_else(|| FhirPathError::evaluation_error(FP0053, "subsumes() requires second argument".to_string()))?)?;

                    if coding_a.system != coding_b.system {
                        return Ok(FhirPathValue::boolean(false));
                    }

                    let svc = match context.terminology {
                        Some(s) => s,
                        None => {
                            return Err(FhirPathError::evaluation_error(
                                FP0200,
                                "Terminology service is not configured".to_string(),
                            ));
                        }
                    };

                    let a_val = TerminologyUtils::coding_to_value(&coding_a);
                    let b_val = TerminologyUtils::coding_to_value(&coding_b);
                    let coll = svc.subsumes(&coding_a.system, &a_val, &b_val, None).await?;
                    // Interpret boolean if provided, else false
                    let subsumes = coll.iter().find_map(|v| match v { FhirPathValue::Boolean(b) => Some(*b), _ => None }).unwrap_or(false);
                    Ok(FhirPathValue::boolean(subsumes))
                })
            }
        )
    }

    fn register_designation_function(&self) -> Result<()> {
        register_function!(
            self,
            async "designation",
            category: FunctionCategory::Terminology,
            description: "Returns designations (display names) for the input coding in the specified language",
            parameters: [
                "language": Some("string".to_string()) => "Language code (optional, e.g., 'en', 'es')",
                "use": Some("string".to_string()) => "Designation use code (optional)"
            ],
            return_type: "collection",
            examples: [
                "Observation.code.coding.designation('es')",
                "Patient.maritalStatus.coding.designation('en', 'display')"
            ],
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathValue>> + Send + '_>> {
                Box::pin(async move {
                    if context.input.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "designation() requires a singleton input".to_string(),
                        ));
                    }

                    if context.arguments.len() > 2 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "designation() accepts at most 2 arguments: language and use".to_string(),
                        ));
                    }

                    let args_vec = context.arguments.cloned_collection();
                    let language = if !args_vec.is_empty() {
                        match &args_vec[0] {
                            FhirPathValue::String(s) => Some(s.as_str()),
                            _ => {
                                return Err(FhirPathError::evaluation_error(
                                    FP0051,
                                    "designation() language argument must be a string".to_string(),
                                ));
                            }
                        }
                    } else {
                        None
                    };

                    let use_code = if args_vec.len() > 1 {
                        match &args_vec[1] {
                            FhirPathValue::String(s) => Some(s.as_str()),
                            _ => {
                                return Err(FhirPathError::evaluation_error(
                                    FP0051,
                                    "designation() use argument must be a string".to_string(),
                                ));
                            }
                        }
                    } else {
                        None
                    };

                    let coding = TerminologyUtils::extract_coding(&context.input)?;
                    let coded_value = TerminologyUtils::coding_to_value(&coding);

                    let svc = match context.terminology {
                        Some(s) => s,
                        None => {
                            return Err(FhirPathError::evaluation_error(
                                FP0200,
                                "Terminology service is not configured".to_string(),
                            ));
                        }
                    };

                    // Build parameters for designation lookup
                    let mut params = std::collections::HashMap::new();
                    if let Some(lang) = language {
                        params.insert("language".to_string(), lang.to_string());
                    }
                    if let Some(use_val) = use_code {
                        params.insert("use".to_string(), use_val.to_string());
                    }

                    // Use lookup operation to get designations - terminology service will extract designations from the result
                    let result = svc.lookup(&coded_value, Some(params)).await?;
                    Ok(FhirPathValue::collection(result.into_vec()))
                })
            }
        )
    }

    fn register_property_function(&self) -> Result<()> {
        register_function!(
            self,
            async "property",
            category: FunctionCategory::Terminology,
            description: "Returns properties of the input coding from the terminology server",
            parameters: ["property": Some("string".to_string()) => "Property name to retrieve"],
            return_type: "collection",
            examples: [
                "Observation.code.coding.property('parent')",
                "Medication.code.coding.property('ingredient')"
            ],
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathValue>> + Send + '_>> {
                Box::pin(async move {
                    if context.input.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "property() requires a singleton input".to_string(),
                        ));
                    }

                    if context.arguments.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "property() requires exactly one property name argument".to_string(),
                        ));
                    }

                    let property_name = match &context.arguments {
                        FhirPathValue::String(s) => s.as_str(),
                        _ => {
                            return Err(FhirPathError::evaluation_error(
                                FP0051,
                                "property() property argument must be a string".to_string(),
                            ));
                        }
                    };

                    let coding = TerminologyUtils::extract_coding(&context.input)?;
                    let coded_value = TerminologyUtils::coding_to_value(&coding);

                    let svc = match context.terminology {
                        Some(s) => s,
                        None => {
                            return Err(FhirPathError::evaluation_error(
                                FP0200,
                                "Terminology service is not configured".to_string(),
                            ));
                        }
                    };

                    // Build parameters for property lookup
                    let mut params = std::collections::HashMap::new();
                    params.insert("property".to_string(), property_name.to_string());

                    // Use lookup operation to get properties - terminology service will extract properties from the result
                    let result = svc.lookup(&coded_value, Some(params)).await?;
                    Ok(FhirPathValue::collection(result.into_vec()))
                })
            }
        )
    }

    fn register_expand_function(&self) -> Result<()> {
        register_function!(
            self,
            async "expand",
            category: FunctionCategory::Terminology,
            description: "Expand a ValueSet to get all member codes (called on %terminologies)",
            parameters: ["valueset": Some("string".to_string()) => "ValueSet URL or identifier"],
            return_type: "resource",
            examples: [
                "%terminologies.expand('http://hl7.org/fhir/ValueSet/administrative-gender')"
            ],
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathValue>> + Send + '_>> {
                Box::pin(async move {
                    // This function should only be called on %terminologies
                    if !Self::is_terminologies_input(&context.input) {
                        return Err(FhirPathError::evaluation_error(
                            FP0051,
                            "expand() can only be called on %terminologies".to_string(),
                        ));
                    }

                    if context.arguments.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "expand() requires exactly one argument (ValueSet URL)".to_string(),
                        ));
                    }

                    let valueset_url = match context.arguments.first() {
                        Some(FhirPathValue::String(s)) => s.as_str(),
                        _ => {
                            return Err(FhirPathError::evaluation_error(
                                FP0051,
                                "expand() ValueSet argument must be a string".to_string(),
                            ));
                        }
                    };

                    // Get terminology service from context
                    let svc = match context.terminology {
                        Some(s) => s,
                        None => {
                            return Err(FhirPathError::evaluation_error(
                                FP0200,
                                "Terminology service is not configured".to_string(),
                            ));
                        }
                    };

                    // Call expand and return the result
                    let expansion_result = svc.expand(valueset_url, None).await?;

                    // Return the ValueSet expansion result
                    if expansion_result.is_empty() {
                        Ok(FhirPathValue::Empty)
                    } else {
                        Ok(expansion_result.first().unwrap().clone())
                    }
                })
            }
        )
    }

    fn register_validate_vs_function(&self) -> Result<()> {
        register_function!(
            self,
            async "validateVS",
            category: FunctionCategory::Terminology,
            description: "Validate a code against a ValueSet (called on %terminologies)",
            parameters: [
                "valueset": Some("string".to_string()) => "ValueSet URL or identifier",
                "coded": Some("coding".to_string()) => "Coding or code to validate"
            ],
            return_type: "resource",
            examples: [
                "%terminologies.validateVS('http://hl7.org/fhir/ValueSet/administrative-gender', $this.gender)"
            ],
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathValue>> + Send + '_>> {
                Box::pin(async move {
                    // This function should only be called on %terminologies
                    if !Self::is_terminologies_input(&context.input) {
                        return Err(FhirPathError::evaluation_error(
                            FP0051,
                            "validateVS() can only be called on %terminologies".to_string(),
                        ));
                    }

                    if context.arguments.len() != 2 {
                        return Err(FhirPathError::evaluation_error(
                            FP0053,
                            "validateVS() requires exactly two arguments (ValueSet URL and coded value)".to_string(),
                        ));
                    }

                    let valueset_url = match context.arguments.first() {
                        Some(FhirPathValue::String(s)) => s.as_str(),
                        _ => {
                            return Err(FhirPathError::evaluation_error(
                                FP0051,
                                "validateVS() ValueSet argument must be a string".to_string(),
                            ));
                        }
                    };

                    let coded_value = context.arguments.get(1).ok_or_else(|| {
                        FhirPathError::evaluation_error(
                            FP0053,
                            "validateVS() requires second argument (coded value)".to_string(),
                        )
                    })?;

                    // Get terminology service from context
                    let svc = match context.terminology {
                        Some(s) => s,
                        None => {
                            return Err(FhirPathError::evaluation_error(
                                FP0200,
                                "Terminology service is not configured".to_string(),
                            ));
                        }
                    };

                    // Call validate_vs and return the result
                    let validation_result = svc.validate_vs(valueset_url, coded_value, None).await?;

                    // Return the validation Parameters result
                    if validation_result.is_empty() {
                        Ok(FhirPathValue::Empty)
                    } else {
                        Ok(validation_result.first().unwrap().clone())
                    }
                })
            }
        )
    }

    /// Check if the input is the %terminologies variable
    fn is_terminologies_input(input: &FhirPathValue) -> bool {
        match input {
            FhirPathValue::TypeInfoObject { name, .. } => name == "terminologies",
            _ => false,
        }
    }
}
