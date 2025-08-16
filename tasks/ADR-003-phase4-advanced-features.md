# ADR-003 Phase 4: Advanced Features and Validation

**Related ADR:** [ADR-003: Enhanced FHIRPath Type Reflection System](../docs/adr/ADR-003-fhirpath-type-reflection-system.md)  
**Phase:** 4 of 5  
**Status:** Planned  
**Priority:** Medium  
**Estimated Effort:** 4-5 weeks  
**Prerequisites:** Phases 1, 2, and 3 completed

## Objective

Implement advanced type reflection features including constraint validation, type inference, IDE integration capabilities, and developer tooling support that establish industry-leading developer experience and runtime validation capabilities.

## Scope

### In Scope
1. **Runtime Constraint Validation**
   - FHIRPath constraint evaluation during type reflection
   - Profile-specific validation rules
   - Cardinality and value constraint checking
   - Terminology binding validation

2. **Advanced Type Inference**
   - Incomplete expression type inference
   - Context-aware type resolution
   - Polymorphic type inference for unions
   - Cross-resource type resolution in Bundle contexts

3. **IDE Integration Foundation**
   - Language Server Protocol (LSP) compatibility
   - Code completion candidate generation
   - Real-time type checking and validation
   - Error recovery and suggestion system

4. **Developer Tooling Support**
   - Type hierarchy visualization
   - Expression analysis and optimization hints
   - Performance profiling integration
   - Debug information generation

### Out of Scope
- Full LSP server implementation (separate project)
- Visual IDE plugins (separate project)
- Web-based tooling interfaces

## Technical Implementation

### 1. Runtime Constraint Validation System

**File:** `crates/fhirpath-model/src/validation/constraint_validator.rs`

```rust
use crate::reflection::{FhirPathTypeInfo, ConstraintValidationResult};
use octofhir_fhirpath_core::FhirPathError;
use async_trait::async_trait;

/// Runtime constraint validation for type reflection
pub struct ConstraintValidator {
    expression_evaluator: Arc<dyn FhirPathExpressionEvaluator>,
    terminology_resolver: Arc<dyn TerminologyResolver>,
    validation_cache: Arc<ValidationCache>,
}

impl ConstraintValidator {
    pub async fn validate_type_constraints(
        &self,
        value: &FhirPathValue,
        type_info: &FhirPathTypeInfo,
        context: &ValidationContext,
    ) -> Result<ConstraintValidationResult, FhirPathError> {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        
        match type_info {
            FhirPathTypeInfo::ClassInfo { elements, profile_constraints, .. } => {
                // Validate element cardinalities
                violations.extend(self.validate_cardinalities(value, elements).await?);
                
                // Validate profile constraints if present
                if let Some(constraints) = profile_constraints {
                    violations.extend(self.validate_profile_constraints(
                        value, constraints, context
                    ).await?);
                }
                
                // Validate element-level constraints
                for element in elements.iter() {
                    if let Some(element_constraints) = &element.constraints {
                        violations.extend(self.validate_element_constraints(
                            value, element, element_constraints, context
                        ).await?);
                    }
                }
            },
            FhirPathTypeInfo::SimpleTypeInfo { constraints, .. } => {
                if let Some(type_constraints) = constraints {
                    violations.extend(self.validate_simple_type_constraints(
                        value, type_constraints, context
                    ).await?);
                }
            },
            FhirPathTypeInfo::ListTypeInfo { element_type, cardinality } => {
                violations.extend(self.validate_list_constraints(
                    value, element_type, cardinality, context
                ).await?);
            },
            _ => {} // Other types have no constraints to validate
        }
        
        Ok(ConstraintValidationResult {
            is_valid: violations.is_empty(),
            violations,
            warnings,
            validation_metadata: self.create_validation_metadata(type_info),
        })
    }
    
    async fn validate_profile_constraints(
        &self,
        value: &FhirPathValue,
        constraints: &ProfileConstraints,
        context: &ValidationContext,
    ) -> Result<Vec<ConstraintViolation>, FhirPathError> {
        let mut violations = Vec::new();
        
        // Validate FHIRPath constraints
        for fhirpath_constraint in &constraints.fhirpath_constraints {
            let constraint_result = self.expression_evaluator
                .evaluate_constraint(
                    &fhirpath_constraint.expression,
                    value,
                    context,
                ).await?;
            
            if !constraint_result.is_satisfied {
                violations.push(ConstraintViolation {
                    severity: fhirpath_constraint.severity,
                    message: fhirpath_constraint.human.clone(),
                    location: constraint_result.violation_path,
                    constraint_key: fhirpath_constraint.key.clone(),
                });
            }
        }
        
        // Validate terminology bindings
        for binding in &constraints.terminology_bindings {
            if let Some(violation) = self.validate_terminology_binding(
                value, binding, context
            ).await? {
                violations.push(violation);
            }
        }
        
        Ok(violations)
    }
    
    async fn validate_terminology_binding(
        &self,
        value: &FhirPathValue,
        binding: &TerminologyBinding,
        context: &ValidationContext,
    ) -> Result<Option<ConstraintViolation>, FhirPathError> {
        // Extract coded values from the resource
        let coded_values = self.extract_coded_values(value, &binding.path).await?;
        
        for coded_value in coded_values {
            let is_valid = self.terminology_resolver
                .validate_code(
                    &coded_value.system,
                    &coded_value.code,
                    &binding.value_set_url,
                    binding.strength,
                ).await?;
            
            if !is_valid && binding.strength == BindingStrength::Required {
                return Ok(Some(ConstraintViolation {
                    severity: ConstraintSeverity::Error,
                    message: format!(
                        "Code '{}' from system '{}' is not valid for required binding to '{}'",
                        coded_value.code, coded_value.system, binding.value_set_url
                    ),
                    location: binding.path.clone(),
                    constraint_key: format!("terminology-{}", binding.value_set_url),
                }));
            }
        }
        
        Ok(None)
    }
}

/// Constraint violation information
#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    pub severity: ConstraintSeverity,
    pub message: String,
    pub location: String,
    pub constraint_key: String,
}

/// Constraint validation result
#[derive(Debug, Clone)]
pub struct ConstraintValidationResult {
    pub is_valid: bool,
    pub violations: Vec<ConstraintViolation>,
    pub warnings: Vec<ConstraintWarning>,
    pub validation_metadata: ValidationMetadata,
}

#[derive(Debug, Clone)]
pub enum ConstraintSeverity {
    Error,
    Warning,
    Information,
}

/// Terminology binding validation
#[derive(Debug, Clone)]
pub struct TerminologyBinding {
    pub path: String,
    pub value_set_url: String,
    pub strength: BindingStrength,
}

#[derive(Debug, Clone)]
pub enum BindingStrength {
    Required,
    Extensible,
    Preferred,
    Example,
}
```

### 2. Advanced Type Inference Engine

**File:** `crates/fhirpath-model/src/inference/type_inference.rs`

```rust
use crate::reflection::{FhirPathTypeInfo, TypeInferenceResult};
use octofhir_fhirpath_ast::{Expression, Visitor};
use async_trait::async_trait;

/// Advanced type inference for FHIRPath expressions
pub struct TypeInferenceEngine {
    model_provider: Arc<dyn AdvancedModelProvider>,
    inference_cache: Arc<TypeInferenceCache>,
    confidence_calculator: ConfidenceCalculator,
}

impl TypeInferenceEngine {
    pub async fn infer_expression_type(
        &self,
        expression: &str,
        context_type: &str,
    ) -> Result<TypeInferenceResult, TypeInferenceError> {
        // Parse the expression to AST
        let ast = self.parse_expression(expression)?;
        
        // Perform type inference traversal
        let mut inference_visitor = TypeInferenceVisitor::new(
            self.model_provider.clone(),
            context_type,
        );
        
        ast.accept(&mut inference_visitor).await?;
        
        let inferred_type = inference_visitor.get_result_type()?;
        let confidence = self.confidence_calculator.calculate_confidence(
            &ast,
            &inferred_type,
            &inference_visitor.get_inference_steps(),
        );
        
        Ok(TypeInferenceResult {
            inferred_type,
            confidence,
            inference_steps: inference_visitor.get_inference_steps(),
            warnings: inference_visitor.get_warnings(),
            optimization_hints: self.generate_optimization_hints(&ast, &inferred_type),
        })
    }
    
    pub async fn infer_incomplete_expression(
        &self,
        partial_expression: &str,
        context_type: &str,
        cursor_position: usize,
    ) -> Result<IncompleteBehaviorInferenceResult, TypeInferenceError> {
        // Parse what we can of the expression
        let partial_ast = self.parse_partial_expression(partial_expression, cursor_position)?;
        
        // Infer type at cursor position
        let context_at_cursor = self.determine_context_at_position(
            &partial_ast, cursor_position, context_type
        ).await?;
        
        // Generate completion candidates
        let completion_candidates = self.model_provider
            .get_completion_candidates(
                &context_at_cursor.path,
                &context_at_cursor.type_name,
            ).await?;
        
        Ok(IncompleteBehaviorInferenceResult {
            context_type: context_at_cursor.inferred_type,
            completion_candidates,
            valid_next_tokens: self.determine_valid_tokens(&partial_ast),
            type_suggestions: self.generate_type_suggestions(&context_at_cursor),
        })
    }
    
    async fn determine_context_at_position(
        &self,
        ast: &PartialExpression,
        position: usize,
        initial_context: &str,
    ) -> Result<InferenceContext, TypeInferenceError> {
        let mut context_tracker = ContextTracker::new(initial_context);
        
        // Traverse AST up to the cursor position
        for node in ast.nodes_before_position(position) {
            match node {
                AstNode::Identifier(name) => {
                    let current_type = context_tracker.get_current_type();
                    let property_type = self.model_provider
                        .get_element_type_info(&current_type, name)
                        .await?;
                    
                    context_tracker.push_context(property_type);
                },
                AstNode::FunctionCall(func_name, args) => {
                    let result_type = self.infer_function_result_type(
                        func_name, args, &context_tracker
                    ).await?;
                    
                    context_tracker.push_context(result_type);
                },
                _ => {} // Handle other node types
            }
        }
        
        Ok(InferenceContext {
            inferred_type: context_tracker.get_current_type(),
            path: context_tracker.get_current_path(),
            confidence: context_tracker.get_confidence(),
        })
    }
}

/// Type inference visitor for AST traversal
struct TypeInferenceVisitor {
    model_provider: Arc<dyn AdvancedModelProvider>,
    type_stack: Vec<FhirPathTypeInfo>,
    inference_steps: Vec<InferenceStep>,
    warnings: Vec<TypeWarning>,
    current_context: String,
}

#[async_trait]
impl Visitor for TypeInferenceVisitor {
    async fn visit_identifier(&mut self, name: &str) -> Result<(), FhirPathError> {
        let current_type = self.get_current_type();
        
        match self.model_provider.get_element_type_info(&current_type, name).await {
            Ok(element_type) => {
                self.type_stack.push(element_type.clone());
                self.inference_steps.push(InferenceStep {
                    operation: InferenceOperation::PropertyAccess,
                    input_type: current_type,
                    output_type: element_type,
                    confidence: 0.95,
                });
            },
            Err(_) => {
                self.warnings.push(TypeWarning {
                    message: format!("Property '{}' not found on type '{}'", name, current_type),
                    suggestion: self.suggest_similar_properties(&current_type, name).await,
                });
                
                // Push Any type as fallback
                self.type_stack.push(FhirPathTypeInfo::SimpleTypeInfo {
                    namespace: Arc::from("System"),
                    name: Arc::from("Any"),
                    base_type: None,
                    constraints: None,
                });
            }
        }
        
        Ok(())
    }
    
    async fn visit_function_call(
        &mut self,
        function_name: &str,
        args: &[Expression],
    ) -> Result<(), FhirPathError> {
        // Visit arguments first
        for arg in args {
            arg.accept(self).await?;
        }
        
        // Infer function result type
        let arg_types: Vec<_> = args.iter()
            .map(|_| self.type_stack.pop().unwrap_or_else(|| {
                FhirPathTypeInfo::SimpleTypeInfo {
                    namespace: Arc::from("System"),
                    name: Arc::from("Any"),
                    base_type: None,
                    constraints: None,
                }
            }))
            .collect();
        
        let result_type = self.infer_function_result_type(function_name, &arg_types).await?;
        self.type_stack.push(result_type);
        
        Ok(())
    }
}
```

### 3. IDE Integration Foundation

**File:** `crates/fhirpath-model/src/ide/completion.rs`

```rust
use crate::reflection::{FhirPathTypeInfo, CompletionCandidate, CompletionKind};

/// IDE completion support for FHIRPath expressions
pub struct CompletionProvider {
    model_provider: Arc<dyn AdvancedModelProvider>,
    function_registry: Arc<dyn FunctionRegistry>,
    completion_cache: Arc<CompletionCache>,
}

impl CompletionProvider {
    pub async fn get_completions(
        &self,
        partial_path: &str,
        context_type: &str,
        cursor_position: usize,
    ) -> Result<Vec<CompletionCandidate>, CompletionError> {
        let completion_context = self.analyze_completion_context(
            partial_path, context_type, cursor_position
        ).await?;
        
        let mut candidates = Vec::new();
        
        match completion_context.completion_type {
            CompletionType::Property => {
                candidates.extend(self.get_property_completions(
                    &completion_context.current_type,
                    &completion_context.filter_text,
                ).await?);
            },
            CompletionType::Function => {
                candidates.extend(self.get_function_completions(
                    &completion_context.current_type,
                    &completion_context.filter_text,
                ).await?);
            },
            CompletionType::Operator => {
                candidates.extend(self.get_operator_completions(
                    &completion_context.current_type,
                    &completion_context.filter_text,
                ).await?);
            },
            CompletionType::Constant => {
                candidates.extend(self.get_constant_completions(
                    &completion_context.current_type,
                    &completion_context.filter_text,
                ).await?);
            },
        }
        
        // Sort by relevance and confidence
        candidates.sort_by(|a, b| {
            b.relevance_score.partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(candidates)
    }
    
    async fn get_property_completions(
        &self,
        type_name: &str,
        filter: &str,
    ) -> Result<Vec<CompletionCandidate>, CompletionError> {
        let type_info = self.model_provider
            .get_fhirpath_type_info(type_name)
            .await?;
        
        let mut candidates = Vec::new();
        
        if let FhirPathTypeInfo::ClassInfo { elements, .. } = type_info {
            for element in elements.iter() {
                if element.name.starts_with(filter) {
                    let relevance = self.calculate_property_relevance(element, filter);
                    
                    candidates.push(CompletionCandidate {
                        name: element.name.clone(),
                        type_info: self.resolve_element_type(element).await?,
                        documentation: element.constraints
                            .as_ref()
                            .and_then(|c| c.documentation.clone()),
                        completion_kind: CompletionKind::Property,
                        relevance_score: relevance,
                    });
                }
            }
        }
        
        Ok(candidates)
    }
    
    fn calculate_property_relevance(
        &self,
        element: &ClassInfoElement,
        filter: &str,
    ) -> f32 {
        let mut score = 0.0;
        
        // Exact prefix match gets highest score
        if element.name.starts_with(filter) {
            score += 0.8;
        }
        
        // Must-support elements get priority
        if let Some(constraints) = &element.constraints {
            if constraints.is_must_support {
                score += 0.2;
            }
        }
        
        // Common properties get slight boost
        if matches!(element.name.as_ref(), "id" | "status" | "code" | "value") {
            score += 0.1;
        }
        
        // Required elements get priority
        if element.cardinality.min > 0 {
            score += 0.15;
        }
        
        score
    }
}

/// Completion context analysis
#[derive(Debug)]
struct CompletionContext {
    pub current_type: String,
    pub completion_type: CompletionType,
    pub filter_text: String,
    pub path_so_far: String,
}

#[derive(Debug)]
enum CompletionType {
    Property,
    Function,
    Operator,
    Constant,
}
```

## Implementation Tasks

### Week 1: Runtime Constraint Validation
- [ ] Implement ConstraintValidator with FHIRPath expression evaluation
- [ ] Add terminology binding validation
- [ ] Create cardinality and value constraint checking
- [ ] Integration with existing type reflection system

### Week 2: Advanced Type Inference
- [ ] Implement TypeInferenceEngine with AST traversal
- [ ] Add incomplete expression inference
- [ ] Create context-aware type resolution
- [ ] Add confidence scoring and optimization hints

### Week 3: IDE Integration Foundation
- [ ] Implement CompletionProvider with property/function completion
- [ ] Add real-time type checking capabilities
- [ ] Create error recovery and suggestion system
- [ ] Add type hierarchy visualization support

### Week 4: Developer Tooling
- [ ] Add expression analysis and optimization hints
- [ ] Create performance profiling integration
- [ ] Implement debug information generation
- [ ] Comprehensive testing and documentation

### Week 5: Integration and Optimization
- [ ] Integration testing with all previous phases
- [ ] Performance optimization and caching
- [ ] Error handling and edge case coverage
- [ ] Documentation and usage examples

## Success Criteria

### Functional Requirements
- [ ] Runtime constraint validation operational
- [ ] Type inference accuracy >95% for common expressions
- [ ] IDE completion provides relevant suggestions
- [ ] Real-time error detection and recovery

### Performance Requirements
- [ ] Constraint validation <10ms for typical resources
- [ ] Type inference <5ms for expressions up to 50 tokens
- [ ] Completion suggestions generated in <100ms
- [ ] Memory usage optimized for long-running IDE scenarios

### Quality Requirements
- [ ] 100% test coverage for validation and inference
- [ ] Comprehensive error handling
- [ ] Performance profiling and optimization
- [ ] Developer-friendly APIs and documentation

## Dependencies

### Phase Dependencies
- Phase 1: Core type reflection infrastructure
- Phase 2: Enhanced type function implementation
- Phase 3: FHIRSchema integration enhancements

### External Dependencies
- FHIRPath expression evaluator
- Terminology service integration
- LSP protocol definitions
- Performance profiling tools

## Testing Strategy

### Validation Testing
- Constraint evaluation accuracy
- Profile compliance checking
- Terminology binding validation
- Error handling and recovery

### Inference Testing
- Type inference accuracy across expression types
- Incomplete expression handling
- Context sensitivity validation
- Performance under load

### IDE Integration Testing
- Completion relevance and accuracy
- Real-time validation responsiveness
- Error recovery scenarios
- Memory usage in long-running scenarios

## Deliverables

1. **Runtime Constraint Validation** - Complete validation system
2. **Advanced Type Inference** - Context-aware inference engine
3. **IDE Integration Foundation** - Completion and validation APIs
4. **Developer Tooling Support** - Analysis and debugging capabilities
5. **Test Suite** - Comprehensive validation and inference testing
6. **Performance Benchmarks** - Optimized operation metrics
7. **Documentation** - Integration guides and API documentation

## Next Phase Integration

This phase enables:
- **Phase 5:** Complete specification compliance testing
- LSP server implementation (separate project)
- Visual IDE plugin development
- Advanced debugging and profiling tools