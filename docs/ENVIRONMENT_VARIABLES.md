# FHIRPath Environment Variables

This document describes the environment variables support in the FHIRPath implementation, following the [FHIRPath specification](https://hl7.org/fhirpath/#environment-variables).

## Overview

Environment variables in FHIRPath provide access to contextual information and allow for parameterized expressions. They are accessed using the `%` prefix (e.g., `%context`, `%resource`).

## Standard Environment Variables

### Core Resource Variables

#### `%context`
The original node that is in the input context when the FHIRPath expression is evaluated.

```fhirpath
%context.resourceType  // Returns "Patient" for a Patient resource
```

#### `%resource` 
The resource that contains the original node that is in `%context`. In most cases, this is the same as `%context`.

```fhirpath
%resource.id  // Returns the resource ID
```

#### `%rootResource`
The container resource for the resource identified by `%resource`. For contained resources, this refers to the containing resource. In most cases, this is the same as `%resource`.

```fhirpath
%rootResource.resourceType  // Returns the root resource type
```

### Terminology Variables

#### `%sct`
The URL for SNOMED CT terminology system.

```fhirpath
%sct  // Returns "http://snomed.info/sct"
```

#### `%loinc`
The URL for LOINC terminology system.

```fhirpath
%loinc  // Returns "http://loinc.org"
```

#### `%"vs-[name]"`
Full URL for HL7 value sets with the specified name.

```fhirpath
%"vs-administrative-gender"  // Returns "http://hl7.org/fhir/ValueSet/administrative-gender"
```

## Custom Environment Variables

You can define custom environment variables when evaluating expressions:

### Programmatic API

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue};
use std::collections::HashMap;

// Create engine
let mut engine = FhirPathEngine::with_mock_provider();

// Define custom variables
let mut variables = HashMap::new();
variables.insert("myCustomVar".to_string(), FhirPathValue::String("custom value".into()));
variables.insert("threshold".to_string(), FhirPathValue::Integer(100));

// Evaluate with variables
let result = engine.evaluate_with_variables(
    "%myCustomVar",
    input_data,
    variables
).await?;
```

### CLI Usage

```bash
# Using custom variables in CLI
echo '{"resourceType": "Patient"}' | octofhir-fhirpath evaluate '%customVar' --variable 'customVar="Hello World"'

# Multiple variables
echo '{"resourceType": "Patient"}' | octofhir-fhirpath evaluate 'age > %minAge' \
  --variable 'minAge=18' \
  --variable 'maxAge=65'

# JSON values
echo '{"resourceType": "Patient"}' | octofhir-fhirpath evaluate '%config.enabled' \
  --variable 'config={"enabled": true, "timeout": 30}'
```

## Variable Resolution

Environment variables are resolved in the following order:

1. **Custom variables**: Variables explicitly set via `evaluate_with_variables()` or `--variable` CLI option
2. **Standard variables**: Built-in FHIRPath environment variables (`%sct`, `%loinc`, etc.)
3. **Empty**: If no variable is found, the expression returns empty (`{}`)

## Examples

### Basic Usage

```fhirpath
// Access root resource type
%context.resourceType

// Navigate from environment variable
%resource.name.given

// Use terminology URLs
code.system = %sct
```

### Custom Variables in Complex Expressions

```fhirpath
// Using custom threshold
Patient.age > %ageThreshold

// Combining custom and standard variables
extension.where(url = %customExtensionUrl).value
```

### CLI Examples

```bash
# Environment variable navigation
echo '{"resourceType": "Patient", "name": [{"given": ["John"]}]}' | \
  octofhir-fhirpath evaluate '%resource.name.given'

# Custom variable
echo '{"age": 25}' | \
  octofhir-fhirpath evaluate 'age > %threshold' --variable 'threshold=18'

# Multiple variables
echo '{"status": "active"}' | \
  octofhir-fhirpath evaluate 'status = %activeStatus and version > %minVersion' \
  --variable 'activeStatus="active"' \
  --variable 'minVersion=1'
```

## Error Handling

- **Undefined variables**: Return empty collection (`{}`) per FHIRPath specification
- **Invalid variable names**: Follow FHIRPath identifier rules
- **Type mismatches**: Variables can hold any FHIRPath value type

## Implementation Notes

- Environment variables are implemented through the `EvaluationContext` variable scope system
- The `%` prefix is handled by the parser and stripped from variable names
- Variable lookup is case-sensitive
- Variables are inherited in nested evaluation contexts (e.g., within `where()` clauses)

## Migration from Previous Versions

If you're upgrading from a version without environment variable support:

1. **API Changes**: Use `evaluate_with_variables()` instead of `evaluate()` for custom variables
2. **CLI Changes**: Use `--variable name=value` syntax for custom variables
3. **Behavior**: Standard environment variables (`%context`, `%resource`, etc.) now work automatically

## See Also

- [FHIRPath Specification - Environment Variables](https://hl7.org/fhirpath/#environment-variables)
- [API Documentation](../README.md)
- [CLI Reference](../CLAUDE.md#cli-commands)