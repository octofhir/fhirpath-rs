//! Field existence validation for FHIRPath expressions using FhirSchema
//!
//! This module provides validation that ensures path navigation uses fields that
//! actually exist in FHIR resource types, leveraging FhirSchemaFieldValidator
//! for accurate field existence checking based on the actual FHIR schema.

use octofhir_fhirpath_ast::{ExpressionNode, LiteralValue};
use octofhir_fhirpath_model::provider::{ModelProvider, TypeReflectionInfo};
use octofhir_fhirschema::{FhirSchemaFieldValidator, ModelProvider as SchemaModelProvider};
use std::sync::Arc;

use crate::{
    error::{ValidationError, ValidationErrorType},
    types::AnalysisContext,
};

/// Information about field cardinality for validation
#[derive(Debug, Clone)]
struct FieldCardinalityInfo {
    /// Whether this field can contain multiple values (max > 1 or max = *)
    pub is_collection: bool,
    /// Whether this field is required (min > 0)
    pub is_required: bool,
    /// The actual cardinality string (e.g., "0..1", "1..*", "0..*")
    pub cardinality: String,
    /// The field type
    pub field_type: String,
}

/// Function cardinality requirement for validation
#[derive(Debug, Clone, PartialEq)]
enum FunctionCardinalityRequirement {
    /// Function requires a collection input
    RequiresCollection,
    /// Function requires a scalar input but may work on collections
    RequiresScalar,
    /// Function accepts both collection and scalar inputs
    AcceptsBoth,
}

/// Field validator that checks if fields exist in FHIR resource types using FhirSchema
pub struct FieldValidator {
    schema_field_validator: Arc<FhirSchemaFieldValidator>,
}

/// Context for tracking field validation during AST traversal
#[derive(Debug, Clone)]
struct FieldValidationContext {
    /// Current resource type we're validating against
    current_resource_type: Option<String>,
    /// Full navigation path with detailed context
    navigation_path: Vec<PathSegment>,
    /// Whether we're in a valid traversal context
    is_valid_context: bool,
    /// Stack of contexts for nested expressions (filters, etc.)
    context_stack: Vec<ContextFrame>,
    /// Current field cardinality information for validation
    current_cardinality: Option<FieldCardinalityInfo>,
}

/// Represents a segment in the navigation path
#[derive(Debug, Clone)]
struct PathSegment {
    /// Name of the field or operation
    segment_name: String,
    /// Type of segment (field, index, filter, etc.)
    segment_type: SegmentType,
    /// The resource/element type this segment is applied to
    applied_to_type: Option<String>,
    /// The result type after this segment
    result_type: Option<String>,
    /// Array index if applicable
    array_index: Option<usize>,
    /// Whether this segment creates a collection
    creates_collection: bool,
}

/// Type of path segment
#[derive(Debug, Clone, PartialEq)]
enum SegmentType {
    /// Simple field access (e.g., "name")
    Field,
    /// Array index access (e.g., "[0]")
    Index,
    /// Filter operation (e.g., ".where(...)")
    Filter,
    /// Function call (e.g., ".count()")
    Function,
    /// Method call (e.g., ".exists()")
    Method,
    /// Root identifier (e.g., "Patient")
    Root,
}

/// Context frame for nested expressions
#[derive(Debug, Clone)]
struct ContextFrame {
    /// The context at this frame
    resource_type: Option<String>,
    /// Path at this frame
    path_snapshot: Vec<PathSegment>,
    /// Description of this frame
    frame_description: String,
}

impl FieldValidationContext {
    fn new() -> Self {
        Self {
            current_resource_type: None,
            navigation_path: Vec::new(),
            is_valid_context: true,
            context_stack: Vec::new(),
            current_cardinality: None,
        }
    }

    fn with_resource_type(resource_type: String) -> Self {
        Self {
            current_resource_type: Some(resource_type.clone()),
            navigation_path: vec![PathSegment {
                segment_name: resource_type.clone(),
                segment_type: SegmentType::Root,
                applied_to_type: None,
                result_type: Some(resource_type),
                array_index: None,
                creates_collection: false,
            }],
            is_valid_context: true,
            context_stack: Vec::new(),
            current_cardinality: None,
        }
    }

    fn add_field_segment(
        &mut self,
        field_name: &str,
        applied_to_type: &str,
        result_type: Option<String>,
    ) {
        self.navigation_path.push(PathSegment {
            segment_name: field_name.to_string(),
            segment_type: SegmentType::Field,
            applied_to_type: Some(applied_to_type.to_string()),
            result_type: result_type.clone(),
            array_index: None,
            creates_collection: false, // Will be determined by schema lookup
        });
        self.current_resource_type = result_type;
    }

    fn add_index_segment(&mut self, index: usize, applied_to_type: &str) {
        self.navigation_path.push(PathSegment {
            segment_name: format!("[{}]", index),
            segment_type: SegmentType::Index,
            applied_to_type: Some(applied_to_type.to_string()),
            result_type: Some(applied_to_type.to_string()),
            array_index: Some(index),
            creates_collection: false,
        });
    }

    fn add_filter_segment(&mut self, description: &str, applied_to_type: &str) {
        self.navigation_path.push(PathSegment {
            segment_name: format!(".where({})", description),
            segment_type: SegmentType::Filter,
            applied_to_type: Some(applied_to_type.to_string()),
            result_type: Some(applied_to_type.to_string()), // Filter preserves type but may reduce collection
            array_index: None,
            creates_collection: true, // Filters typically work on collections
        });
    }

    fn add_function_segment(
        &mut self,
        function_name: &str,
        applied_to_type: &str,
        result_type: Option<String>,
    ) {
        self.navigation_path.push(PathSegment {
            segment_name: format!(".{}()", function_name),
            segment_type: SegmentType::Function,
            applied_to_type: Some(applied_to_type.to_string()),
            result_type: result_type.clone(),
            array_index: None,
            creates_collection: false,
        });
        self.current_resource_type = result_type;
    }

    fn add_method_segment(
        &mut self,
        method_name: &str,
        applied_to_type: &str,
        result_type: Option<String>,
    ) {
        self.navigation_path.push(PathSegment {
            segment_name: format!(".{}(...)", method_name),
            segment_type: SegmentType::Method,
            applied_to_type: Some(applied_to_type.to_string()),
            result_type: result_type.clone(),
            array_index: None,
            creates_collection: method_name == "select", // Select creates collections
        });
        self.current_resource_type = result_type;
    }

    fn push_context_frame(&mut self, description: String) {
        self.context_stack.push(ContextFrame {
            resource_type: self.current_resource_type.clone(),
            path_snapshot: self.navigation_path.clone(),
            frame_description: description,
        });
    }

    fn pop_context_frame(&mut self) -> Option<ContextFrame> {
        self.context_stack.pop()
    }

    fn invalidate(&mut self) {
        self.is_valid_context = false;
    }

    fn get_full_path(&self) -> String {
        self.navigation_path
            .iter()
            .map(|seg| match seg.segment_type {
                SegmentType::Root => seg.segment_name.clone(),
                SegmentType::Field => format!(".{}", seg.segment_name),
                _ => seg.segment_name.clone(),
            })
            .collect::<String>()
    }

    fn get_detailed_path_description(&self) -> String {
        let mut description = String::new();

        for (i, segment) in self.navigation_path.iter().enumerate() {
            if i > 0 {
                description.push_str(" → ");
            }

            match &segment.segment_type {
                SegmentType::Root => {
                    description.push_str(&format!("{} (resource)", segment.segment_name));
                }
                SegmentType::Field => {
                    description.push_str(&format!(
                        "{} (field on {}{})",
                        segment.segment_name,
                        segment.applied_to_type.as_deref().unwrap_or("unknown"),
                        if let Some(ref result_type) = segment.result_type {
                            format!(" → {}", result_type)
                        } else {
                            String::new()
                        }
                    ));
                }
                SegmentType::Index => {
                    description.push_str(&format!("{} (array index)", segment.segment_name));
                }
                SegmentType::Filter => {
                    description.push_str(&format!("{} (filter)", segment.segment_name));
                }
                SegmentType::Function => {
                    description.push_str(&format!(
                        "{} (function{})",
                        segment.segment_name,
                        if let Some(ref result_type) = segment.result_type {
                            format!(" → {}", result_type)
                        } else {
                            String::new()
                        }
                    ));
                }
                SegmentType::Method => {
                    description.push_str(&format!(
                        "{} (method{})",
                        segment.segment_name,
                        if let Some(ref result_type) = segment.result_type {
                            format!(" → {}", result_type)
                        } else {
                            String::new()
                        }
                    ));
                }
            }
        }

        description
    }

    fn get_current_context_info(&self) -> String {
        if let Some(frame) = self.context_stack.last() {
            format!(
                "Within {}: {}",
                frame.frame_description,
                self.get_detailed_path_description()
            )
        } else {
            self.get_detailed_path_description()
        }
    }

    /// Get the current field name for error reporting
    fn get_current_field_name(&self) -> Option<String> {
        // Find the last field segment in the navigation path
        self.navigation_path
            .iter()
            .rev()
            .find(|segment| matches!(segment.segment_type, SegmentType::Field))
            .map(|segment| segment.segment_name.clone())
    }
}

impl FieldValidator {
    /// Create a new field validator with the given model provider
    pub fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        // Create a FhirSchemaFieldValidator using the ModelProvider
        // We need to convert from ModelProvider to SchemaModelProvider
        let schema_model_provider = SchemaModelProviderAdapter::new(model_provider);
        let schema_field_validator = Arc::new(FhirSchemaFieldValidator::new(Arc::new(
            schema_model_provider,
        )));

        Self {
            schema_field_validator,
        }
    }

    /// Get access to the schema field validator for additional operations
    pub fn get_schema_field_validator(&self) -> &Arc<FhirSchemaFieldValidator> {
        &self.schema_field_validator
    }

    /// Validate that all field navigation in the expression uses existing fields
    pub async fn validate_field_navigation(
        &self,
        expression: &ExpressionNode,
        context: &AnalysisContext,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let mut field_context = FieldValidationContext::new();

        self.validate_node_recursive(expression, context, &mut field_context, &mut errors)
            .await;

        errors
    }

    /// Recursively validate field navigation in AST nodes
    #[async_recursion::async_recursion]
    async fn validate_node_recursive(
        &self,
        node: &ExpressionNode,
        context: &AnalysisContext,
        field_context: &mut FieldValidationContext,
        errors: &mut Vec<ValidationError>,
    ) -> Option<String> {
        if !field_context.is_valid_context {
            return None;
        }

        match node {
            ExpressionNode::Identifier(name) => {
                // For root identifiers, check if they're valid FHIR resource types
                if field_context.current_resource_type.is_none() {
                    // This is a root resource type identifier
                    match self
                        .schema_field_validator
                        .validate_resource_type(name)
                        .await
                    {
                        Ok(is_valid) if is_valid => {
                            field_context.current_resource_type = Some(name.clone());
                            Some(name.clone())
                        }
                        Ok(_) => {
                            // Invalid resource type
                            if let Ok(suggestions) = self
                                .schema_field_validator
                                .generate_resource_type_suggestions(name)
                                .await
                            {
                                errors.push(ValidationError {
                                    message: format!(
                                        "'{}' is not a valid FHIR resource type",
                                        name
                                    ),
                                    error_type: ValidationErrorType::InvalidResourceType,
                                    location: None,
                                    suggestions,
                                });
                            } else {
                                errors.push(ValidationError {
                                    message: format!(
                                        "'{}' is not a valid FHIR resource type",
                                        name
                                    ),
                                    error_type: ValidationErrorType::InvalidResourceType,
                                    location: None,
                                    suggestions: vec![],
                                });
                            }
                            field_context.invalidate();
                            None
                        }
                        Err(_) => {
                            // Error occurred during validation
                            field_context.invalidate();
                            None
                        }
                    }
                } else {
                    // This is a field name in a path
                    if let Some(resource_type) = field_context.current_resource_type.clone() {
                        match self
                            .validate_field_in_resource(&resource_type, name, field_context, errors)
                            .await
                        {
                            Some(field_type) => {
                                field_context.add_field_segment(
                                    name,
                                    &resource_type,
                                    Some(field_type.clone()),
                                );
                                Some(field_type)
                            }
                            None => {
                                field_context.invalidate();
                                None
                            }
                        }
                    } else {
                        field_context.invalidate();
                        None
                    }
                }
            }
            ExpressionNode::Path { base, path } => {
                // First validate the base expression
                let base_type = self
                    .validate_node_recursive(base, context, field_context, errors)
                    .await;

                if let Some(current_type) = &base_type {
                    // Now validate the path navigation
                    match self
                        .validate_field_in_resource(current_type, path, field_context, errors)
                        .await
                    {
                        Some(result_type) => {
                            field_context.add_field_segment(
                                path,
                                current_type,
                                Some(result_type.clone()),
                            );
                            Some(result_type)
                        }
                        None => {
                            field_context.invalidate();
                            None
                        }
                    }
                } else {
                    field_context.invalidate();
                    None
                }
            }
            ExpressionNode::FunctionCall(data) => {
                // Push context frame for function call
                field_context.push_context_frame(format!("function call {}()", data.name));

                // Validate arguments recursively
                let mut arg_context = field_context.clone();
                for arg in &data.args {
                    self.validate_node_recursive(arg, context, &mut arg_context, errors)
                        .await;
                }

                // Check for cardinality validation issues before function execution
                self.validate_function_cardinality(&data.name, field_context, errors)
                    .await;

                // Function calls typically return specific types
                let result_type = match data.name.as_str() {
                    "count" | "length" => Some("integer".to_string()),
                    "exists" | "empty" => Some("boolean".to_string()),
                    "first" | "last" | "single" => {
                        // These preserve the type of their input
                        field_context.current_resource_type.clone()
                    }
                    _ => None, // Unknown function result type
                };

                // Add function segment to path if we have a current type
                if let Some(applied_to_type) = field_context.current_resource_type.clone() {
                    field_context.add_function_segment(
                        &data.name,
                        &applied_to_type,
                        result_type.clone(),
                    );
                }

                field_context.pop_context_frame();
                result_type
            }
            ExpressionNode::MethodCall(data) => {
                // Validate base expression first
                let base_type = self
                    .validate_node_recursive(&data.base, context, field_context, errors)
                    .await;

                // Push context frame for method call
                field_context.push_context_frame(format!("method call .{}(...)", data.method));

                // Validate arguments
                let mut arg_context = field_context.clone();
                for arg in &data.args {
                    self.validate_node_recursive(arg, context, &mut arg_context, errors)
                        .await;
                }

                // Method calls typically preserve or transform the base type
                let result_type = match data.method.as_str() {
                    "where" | "select" => base_type.clone(), // These preserve base type
                    _ => None,                               // Unknown method result type
                };

                // Add method segment to path if we have a base type
                if let Some(applied_to_type) = &base_type {
                    field_context.add_method_segment(
                        &data.method,
                        applied_to_type,
                        result_type.clone(),
                    );
                }

                field_context.pop_context_frame();
                result_type
            }
            ExpressionNode::BinaryOp(data) => {
                // Validate both operands
                self.validate_node_recursive(&data.left, context, field_context, errors)
                    .await;
                self.validate_node_recursive(&data.right, context, field_context, errors)
                    .await;
                // Binary operations typically return Boolean
                Some("boolean".to_string())
            }
            ExpressionNode::UnaryOp { operand, .. } => {
                self.validate_node_recursive(operand, context, field_context, errors)
                    .await;
                Some("boolean".to_string())
            }
            ExpressionNode::Index { base, index } => {
                let base_type = self
                    .validate_node_recursive(base, context, field_context, errors)
                    .await;

                // Try to extract numeric index if it's a literal
                let index_value = self.extract_index_value(index);

                // Validate index expression
                self.validate_node_recursive(index, context, field_context, errors)
                    .await;

                // Add index segment to path if we have a base type
                if let (Some(applied_to_type), Some(idx_val)) = (&base_type, index_value) {
                    field_context.add_index_segment(idx_val, applied_to_type);
                }

                base_type // Index preserves base type
            }
            ExpressionNode::Filter { base, condition } => {
                let base_type = self
                    .validate_node_recursive(base, context, field_context, errors)
                    .await;

                // Push context frame for filter
                field_context.push_context_frame("filter condition".to_string());

                // For filter conditions, create a new context since we're filtering items
                let mut filter_context = field_context.clone();
                self.validate_node_recursive(condition, context, &mut filter_context, errors)
                    .await;

                // Add filter segment to path if we have a base type
                if let Some(applied_to_type) = &base_type {
                    let condition_desc = self.get_condition_description(condition);
                    field_context.add_filter_segment(&condition_desc, applied_to_type);
                }

                field_context.pop_context_frame();
                base_type // Filter preserves base type
            }
            ExpressionNode::Union { left, right } => {
                self.validate_node_recursive(left, context, field_context, errors)
                    .await;
                self.validate_node_recursive(right, context, field_context, errors)
                    .await;
                None // Union creates mixed collection
            }
            ExpressionNode::TypeCheck { expression, .. } => {
                self.validate_node_recursive(expression, context, field_context, errors)
                    .await;
                Some("boolean".to_string()) // Type checks return Boolean
            }
            ExpressionNode::TypeCast {
                expression,
                type_name,
            } => {
                self.validate_node_recursive(expression, context, field_context, errors)
                    .await;
                Some(type_name.clone()) // Type cast returns target type
            }
            ExpressionNode::Lambda(data) => {
                // For lambda expressions, validate the body with a new context
                let mut lambda_context = field_context.clone();
                self.validate_node_recursive(&data.body, context, &mut lambda_context, errors)
                    .await;
                None
            }
            ExpressionNode::Conditional(data) => {
                self.validate_node_recursive(&data.condition, context, field_context, errors)
                    .await;
                let then_type = self
                    .validate_node_recursive(&data.then_expr, context, field_context, errors)
                    .await;
                if let Some(else_expr) = &data.else_expr {
                    self.validate_node_recursive(else_expr, context, field_context, errors)
                        .await;
                }
                then_type // Return type of then branch (simplified)
            }
            // Literals and other expressions don't need field validation
            _ => None,
        }
    }

    /// Validate a specific field in a resource type using FhirSchema
    async fn validate_field_in_resource(
        &self,
        resource_type: &str,
        field_name: &str,
        field_context: &mut FieldValidationContext,
        errors: &mut Vec<ValidationError>,
    ) -> Option<String> {
        match self
            .schema_field_validator
            .validate_field(resource_type, field_name)
            .await
        {
            Ok(result) => {
                if result.exists {
                    // Field exists, check if we can determine the next type
                    if let Some(field_info) = &result.element_info {
                        // Parse cardinality information from the field
                        let cardinality_info = self.parse_cardinality_info(field_info, field_name);

                        // Update field context with cardinality information
                        field_context.current_cardinality = Some(cardinality_info);

                        // Try to get the first element type for navigation
                        field_info.element_types.first().cloned()
                    } else {
                        // Field exists but we don't have type information
                        field_context.current_cardinality = None;
                        None
                    }
                } else {
                    // Field doesn't exist
                    let detailed_message = if field_context.navigation_path.len() > 1 {
                        format!(
                            "Field '{}' does not exist in FHIR resource type '{}' at path: {}",
                            field_name,
                            resource_type,
                            field_context.get_current_context_info()
                        )
                    } else {
                        format!(
                            "Field '{}' does not exist in FHIR resource type '{}'",
                            field_name, resource_type
                        )
                    };

                    errors.push(ValidationError {
                        message: detailed_message,
                        error_type: ValidationErrorType::InvalidField,
                        location: None,
                        suggestions: result.suggestions,
                    });
                    None
                }
            }
            Err(_) => {
                // Error occurred during validation
                errors.push(ValidationError {
                    message: format!(
                        "Failed to validate field '{}' in resource type '{}'",
                        field_name, resource_type
                    ),
                    error_type: ValidationErrorType::InvalidField,
                    location: None,
                    suggestions: vec![],
                });
                None
            }
        }
    }

    /// Extract numeric index value from an expression node
    fn extract_index_value(&self, node: &ExpressionNode) -> Option<usize> {
        match node {
            ExpressionNode::Literal(LiteralValue::Integer(i)) => (*i as usize).into(),
            _ => None, // Complex expressions for index not supported
        }
    }

    /// Parse cardinality information from field info
    fn parse_cardinality_info(
        &self,
        field_info: &octofhir_fhirschema::FieldInfo,
        field_name: &str,
    ) -> FieldCardinalityInfo {
        let cardinality = &field_info.cardinality;

        // Parse cardinality string (e.g., "0..1", "1..*", "0..*")
        let is_collection = cardinality.contains("*") || cardinality.ends_with("..n") || {
            // Check if max is > 1
            if let Some(max_part) = cardinality.split("..").last() {
                max_part.parse::<u32>().map_or(false, |max| max > 1)
            } else {
                false
            }
        };

        let is_required = cardinality.starts_with('1') || {
            // Check if min is > 0
            if let Some(min_part) = cardinality.split("..").next() {
                min_part.parse::<u32>().map_or(false, |min| min > 0)
            } else {
                false
            }
        };

        let field_type = field_info
            .element_types
            .first()
            .cloned()
            .unwrap_or_default();

        FieldCardinalityInfo {
            is_collection,
            is_required,
            cardinality: cardinality.clone(),
            field_type,
        }
    }

    /// Validate function cardinality requirements against current field context using registry
    async fn validate_function_cardinality(
        &self,
        function_name: &str,
        field_context: &FieldValidationContext,
        errors: &mut Vec<ValidationError>,
    ) {
        let Some(cardinality_info) = &field_context.current_cardinality else {
            // No cardinality info available for validation
            return;
        };

        // Handle special lambda functions and method calls that have Rust implementations
        // but may not be in the registry as regular functions
        let cardinality_requirement = self.get_function_cardinality_requirement(function_name);

        match cardinality_requirement {
            FunctionCardinalityRequirement::RequiresCollection
                if !cardinality_info.is_collection =>
            {
                let suggestion = self.get_collection_function_suggestion(function_name);

                errors.push(ValidationError {
                    message: format!(
                        "Function '{}()' expects a collection but field '{}' has cardinality '{}' (single value). {}",
                        function_name,
                        field_context.get_current_field_name().unwrap_or("unknown".to_string()),
                        cardinality_info.cardinality,
                        suggestion
                    ),
                    error_type: ValidationErrorType::InvalidField,
                    location: None,
                    suggestions: vec![
                        "Check if you meant to use a different field that can have multiple values".to_string(),
                        format!("Consider if you need this function on a single '{}' value", cardinality_info.field_type)
                    ],
                });
            }

            FunctionCardinalityRequirement::RequiresScalar if cardinality_info.is_collection => {
                errors.push(ValidationError {
                    message: format!(
                        "Function '{}()' typically works on single values but field '{}' has cardinality '{}' (collection). This may return unexpected results",
                        function_name,
                        field_context.get_current_field_name().unwrap_or("unknown".to_string()),
                        cardinality_info.cardinality
                    ),
                    error_type: ValidationErrorType::InvalidField,
                    location: None,
                    suggestions: vec![
                        "Consider using .first() or .single() to get a single value first".to_string(),
                        "Or use collection functions like .all() or .any() to check conditions".to_string()
                    ],
                });
            }

            _ => {
                // No cardinality issues detected
            }
        }
    }

    /// Get function cardinality requirement including special lambda functions
    fn get_function_cardinality_requirement(
        &self,
        function_name: &str,
    ) -> FunctionCardinalityRequirement {
        match function_name {
            // Collection functions (including lambda/method functions with Rust implementations)
            "count" | "length" | "size" | "where" | "select" | "all" | "any" | "exists" | "empty" |
            "first" | "last" | "single" | "tail" | "take" | "skip" | "distinct" | "union" | "intersect" |
            // Lambda method calls that are implemented in Rust but operate on collections
            "filter" | "map" | "reduce" | "aggregate" => FunctionCardinalityRequirement::RequiresCollection,

            // Scalar functions  
            "toString" | "toInteger" | "toDecimal" | "toBoolean" | "toDateTime" | "toTime" |
            "matches" | "replaceMatches" | "contains" | "startsWith" | "endsWith" |
            "indexOf" | "substring" | "upper" | "lower" | "trim" => FunctionCardinalityRequirement::RequiresScalar,

            // Functions that work on both
            _ => FunctionCardinalityRequirement::AcceptsBoth,
        }
    }

    /// Get suggestion message for collection functions
    fn get_collection_function_suggestion(&self, function_name: &str) -> &'static str {
        match function_name {
            "count" | "length" | "size" => "This function counts elements in a collection",
            "where" | "select" | "filter" | "map" => {
                "These functions filter or transform collections"
            }
            "first" | "last" | "single" => "These functions get elements from collections",
            "exists" | "empty" => "These functions check collection state",
            "all" | "any" => "These functions test conditions across collection elements",
            "distinct" | "union" | "intersect" => {
                "These functions operate on collections of elements"
            }
            _ => "This function operates on collections",
        }
    }

    /// Get a human-readable description of a filter condition
    fn get_condition_description(&self, node: &ExpressionNode) -> String {
        match node {
            ExpressionNode::BinaryOp(_data) => {
                format!("binary condition")
            }
            ExpressionNode::Identifier(name) => {
                format!("field check: {}", name)
            }
            ExpressionNode::FunctionCall(data) => {
                format!("{}()", data.name)
            }
            _ => "complex condition".to_string(),
        }
    }
}

/// Adapter to convert ModelProvider to SchemaModelProvider
struct SchemaModelProviderAdapter {
    model_provider: Arc<dyn ModelProvider>,
}

impl SchemaModelProviderAdapter {
    fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        Self { model_provider }
    }

    /// Extract the proper type code from TypeReflectionInfo
    fn extract_element_type_code(&self, type_info: &TypeReflectionInfo) -> String {
        match type_info {
            TypeReflectionInfo::SimpleType { name, .. } => name.clone(),
            TypeReflectionInfo::ClassInfo { name, .. } => name.clone(),
            TypeReflectionInfo::ListType { element_type } => {
                // For list types, we want the element type, not "List"
                self.extract_element_type_code(element_type)
            }
            TypeReflectionInfo::TupleType { .. } => "Tuple".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl SchemaModelProvider for SchemaModelProviderAdapter {
    async fn get_schema(
        &self,
        canonical_url: &str,
    ) -> Option<Arc<octofhir_fhirschema::FhirSchema>> {
        // Try to get schema from the model provider
        if let Some(type_reflection) = self.model_provider.get_type_reflection(canonical_url).await
        {
            // Convert TypeReflectionInfo to FhirSchema (simplified)
            match &type_reflection {
                TypeReflectionInfo::ClassInfo { name, elements, .. } => {
                    let mut schema = octofhir_fhirschema::FhirSchema::new(name.clone());

                    // Add elements to the schema
                    for element in elements {
                        let element_path = if name == &element.name {
                            element.name.clone()
                        } else {
                            format!("{}.{}", name, element.name)
                        };

                        let schema_element = octofhir_fhirschema::Element {
                            path: element_path.clone(),
                            definition: element.documentation.clone(),
                            short: None,
                            comment: None,
                            min: Some(if element.is_required() { 1 } else { 0 }),
                            max: if element.max_cardinality.is_none()
                                || element.max_cardinality.unwrap_or(1) > 1
                            {
                                Some("*".to_string())
                            } else {
                                Some(element.max_cardinality.unwrap_or(1).to_string())
                            },
                            element_type: Some(vec![octofhir_fhirschema::ElementType {
                                code: self.extract_element_type_code(&element.type_info),
                                profile: None,
                                target_profile: None,
                                aggregation: None,
                                versioning: None,
                            }]),
                            fixed: None,
                            pattern: None,
                            constraints: Vec::new(),
                            binding: None,
                            mapping: Vec::new(),
                            is_modifier: false,
                            is_summary: false,
                            extensions: std::collections::HashMap::new(),
                        };

                        schema.elements.insert(element_path, schema_element);
                    }

                    Some(Arc::new(schema))
                }
                _ => None,
            }
        } else {
            None
        }
    }

    async fn get_schemas_by_type(
        &self,
        resource_type: &str,
    ) -> Vec<Arc<octofhir_fhirschema::FhirSchema>> {
        if let Some(schema) = self.get_schema(resource_type).await {
            vec![schema]
        } else {
            Vec::new()
        }
    }

    async fn resolve_profile(
        &self,
        _base_type: &str,
        profile_url: &str,
    ) -> Option<Arc<octofhir_fhirschema::FhirSchema>> {
        self.get_schema(profile_url).await
    }

    async fn has_resource_type(&self, resource_type: &str) -> bool {
        self.model_provider
            .get_type_reflection(resource_type)
            .await
            .is_some()
    }

    async fn get_resource_types(&self) -> Vec<String> {
        // Since we don't have direct access to schema field validator here,
        // and this method is only used for getting schemas by type,
        // we'll return an empty list and let the caller handle resource type resolution
        // through other means (like the type reflection system)
        Vec::new()
    }

    async fn search_schemas(&self, query: &str) -> Vec<Arc<octofhir_fhirschema::FhirSchema>> {
        let resource_types = self.get_resource_types().await;
        let mut results = Vec::new();

        for resource_type in resource_types {
            if resource_type.to_lowercase().contains(&query.to_lowercase()) {
                if let Some(schema) = self.get_schema(&resource_type).await {
                    results.push(schema);
                }
            }
        }

        results
    }
}
