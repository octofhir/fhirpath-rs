# CLI Reference

This document provides comprehensive documentation for the octofhir-fhirpath command-line interface.

## Installation

Install the CLI tools using Cargo:

```bash
cargo install octofhir-fhirpath
```

## Basic Usage

The CLI provides several commands for working with FHIRPath expressions:

```bash
octofhir-fhirpath <COMMAND> [OPTIONS] [EXPRESSION]
```

## Commands

### `evaluate` - Evaluate FHIRPath Expressions

Evaluate FHIRPath expressions against FHIR resources.

```bash
octofhir-fhirpath evaluate [OPTIONS] <EXPRESSION>
```

#### Options

- `--input <FILE|JSON>` - Input FHIR resource (file path or JSON string)
- `--variable <KEY=VALUE>` - Set environment variables (can be used multiple times)
- `--output <FORMAT>` - Output format: `json`, `pretty`, `debug` (default: `pretty`)
- `--timeout <SECONDS>` - Evaluation timeout in seconds (default: 30)

#### Examples

**Simple evaluation with JSON string:**
```bash
octofhir-fhirpath evaluate "Patient.name.given" \
  --input '{"resourceType":"Patient","name":[{"given":["Alice","Bob"]}]}'
# Output: ["Alice", "Bob"]
```

**Evaluate expressions with JSON input from stdin:**
```bash
echo '{"resourceType": "Patient", "name": [{"given": ["John"]}]}' | \
  octofhir-fhirpath evaluate "Patient.name.given"
```

**Evaluate expressions with file input:**
```bash
octofhir-fhirpath evaluate "Patient.name.given" --input patient.json
```

**Evaluate expressions without any input (empty context):**
```bash
octofhir-fhirpath evaluate "true"
octofhir-fhirpath evaluate "1 + 2"
octofhir-fhirpath evaluate "today()"
```

**Using environment variables:**
```bash
octofhir-fhirpath evaluate "age > %minAge" \
  --input patient.json \
  --variable "minAge=18"
```

**Multiple variables:**
```bash
octofhir-fhirpath evaluate "age > %minAge and age < %maxAge" \
  --input patient.json \
  --variable "minAge=18" \
  --variable "maxAge=65"
```

**Complex filtering:**
```bash
octofhir-fhirpath evaluate "Bundle.entry.resource.where(resourceType='Patient').name.family" \
  --input bundle.json
```

### `parse` - Parse FHIRPath to AST

Parse FHIRPath expressions and display the Abstract Syntax Tree.

```bash
octofhir-fhirpath parse [OPTIONS] <EXPRESSION>
```

#### Options

- `--format <FORMAT>` - Output format: `pretty`, `debug`, `json` (default: `pretty`)

#### Examples

**Parse simple expression:**
```bash
octofhir-fhirpath parse "Patient.name.given"
```

**Parse complex expression with pretty formatting:**
```bash
octofhir-fhirpath parse "Patient.name.where(use = 'official').given.first()" --format pretty
```

**Parse expression to JSON:**
```bash
octofhir-fhirpath parse "Bundle.entry[0].resource" --format json
```

### `validate` - Validate FHIRPath Syntax

Validate FHIRPath expression syntax without evaluation.

```bash
octofhir-fhirpath validate [OPTIONS] <EXPRESSION>
```

#### Examples

**Validate correct expression:**
```bash
octofhir-fhirpath validate "Patient.name.given.first()" 
# Output: ✅ Valid FHIRPath expression
```

**Validate incorrect expression:**
```bash
octofhir-fhirpath validate "Patient.name.invalid("
# Output: ❌ Syntax error at position 23: Expected closing parenthesis
```

### `help` - Show Help Information

Display help information for the CLI or specific commands.

```bash
octofhir-fhirpath help
octofhir-fhirpath help evaluate
octofhir-fhirpath help parse
```

## Global Options

These options work with all commands:

- `--verbose`, `-v` - Enable verbose output
- `--quiet`, `-q` - Suppress non-essential output  
- `--color <WHEN>` - Control color output: `auto`, `always`, `never` (default: `auto`)
- `--help`, `-h` - Show help information
- `--version`, `-V` - Show version information

## Output Formats

### Pretty Format (Default)

Human-readable output with syntax highlighting:

```bash
octofhir-fhirpath evaluate "Patient.name.given" --input patient.json
# Output: 
# Result: 
#   ["John", "Jane"]
```

### JSON Format

Machine-readable JSON output:

```bash
octofhir-fhirpath evaluate "Patient.name.given" --input patient.json --output json
# Output: ["John","Jane"]
```

### Debug Format

Detailed debug information including types:

```bash
octofhir-fhirpath evaluate "Patient.name.given" --input patient.json --output debug
# Output:
# FhirPathValue::Collection([
#   FhirPathValue::String("John"),
#   FhirPathValue::String("Jane")
# ])
```

## Environment Variables

The CLI supports FHIRPath environment variables:

### Standard Environment Variables

- `%context` - The original node in the input context
- `%resource` - The resource containing the original node  
- `%rootResource` - The container resource (for contained resources)
- `%sct` - SNOMED CT URL (`http://snomed.info/sct`)
- `%loinc` - LOINC URL (`http://loinc.org`)
- `%"vs-[name]"` - HL7 value set URLs

### Custom Variables

Set custom variables using the `--variable` option:

```bash
# String values
octofhir-fhirpath evaluate '%greeting' --variable 'greeting=Hello World'

# Numeric values  
octofhir-fhirpath evaluate 'age > %threshold' \
  --input patient.json \
  --variable 'threshold=18'

# Boolean values
octofhir-fhirpath evaluate '%enabled' --variable 'enabled=true'

# JSON values (for complex data)
octofhir-fhirpath evaluate '%config.maxItems' \
  --variable 'config={"maxItems": 10, "enabled": true}'
```

## Working with Files

### Input Sources

The CLI accepts input from multiple sources:

**File path:**
```bash
octofhir-fhirpath evaluate "Patient.name" --input /path/to/patient.json
```

**JSON string:**
```bash  
octofhir-fhirpath evaluate "Patient.name" --input '{"resourceType":"Patient"}'
```

**Standard input (stdin):**
```bash
cat patient.json | octofhir-fhirpath evaluate "Patient.name"
```

**No input (empty context):**
```bash
octofhir-fhirpath evaluate "today()"
```

### File Formats

The CLI automatically detects and supports:

- JSON files (`.json`)
- YAML files (`.yaml`, `.yml`) - converted to JSON internally
- Text files containing JSON

## Error Handling

The CLI provides detailed error information:

**Syntax errors:**
```bash
octofhir-fhirpath evaluate "Patient.name.invalid("
# Error: Parse error at line 1, column 23
# Expected: closing parenthesis ')'
# Found: end of input
# 
# Patient.name.invalid(
#                      ^
```

**Runtime errors:**
```bash
octofhir-fhirpath evaluate "Patient.nonexistent.field" --input patient.json
# Error: Path navigation failed
# Field 'nonexistent' not found on resource type 'Patient'
# 
# Suggestions:
#   - Did you mean 'name'?
#   - Check the FHIR specification for valid fields
```

**File errors:**
```bash
octofhir-fhirpath evaluate "Patient.name" --input nonexistent.json
# Error: Failed to read input file
# File not found: nonexistent.json
```

## Exit Codes

The CLI uses standard exit codes:

- `0` - Success
- `1` - General error (syntax error, runtime error)
- `2` - Invalid arguments or options
- `3` - File I/O error
- `4` - Timeout error

## Advanced Usage

### Batch Processing

Process multiple files using shell scripts:

```bash
#!/bin/bash
for file in *.json; do
    echo "Processing $file..."
    octofhir-fhirpath evaluate "Patient.name.family" --input "$file"
done
```

### Pipeline Integration

Use in data processing pipelines:

```bash
# Extract patient names from a bundle
curl -s "https://api.example.com/Bundle/123" | \
  octofhir-fhirpath evaluate "Bundle.entry.resource.where(resourceType='Patient').name.family" | \
  jq -r '.[]'
```

### Performance Testing

Test expression performance:

```bash
time octofhir-fhirpath evaluate "Bundle.entry.resource.where(resourceType='Patient').name.family" \
  --input large-bundle.json
```

### Debugging Complex Expressions

Break down complex expressions step by step:

```bash
# Step 1: Get bundle entries
octofhir-fhirpath evaluate "Bundle.entry" --input bundle.json

# Step 2: Filter to patients
octofhir-fhirpath evaluate "Bundle.entry.resource.where(resourceType='Patient')" --input bundle.json

# Step 3: Extract names  
octofhir-fhirpath evaluate "Bundle.entry.resource.where(resourceType='Patient').name" --input bundle.json

# Step 4: Get family names
octofhir-fhirpath evaluate "Bundle.entry.resource.where(resourceType='Patient').name.family" --input bundle.json
```

## Configuration

### Config File

The CLI supports a configuration file at `~/.octofhir-fhirpath.toml`:

```toml
[default]
output_format = "pretty"
timeout = 60
color = "auto"

[aliases]
patients = "Bundle.entry.resource.where(resourceType='Patient')"
observations = "Bundle.entry.resource.where(resourceType='Observation')"
```

### Environment Variables

Set default behavior using environment variables:

```bash
export OCTOFHIR_FHIRPATH_OUTPUT=json
export OCTOFHIR_FHIRPATH_TIMEOUT=120
export OCTOFHIR_FHIRPATH_COLOR=always
```

## Examples by Use Case

### Healthcare Data Analysis

**Find all active patients:**
```bash
octofhir-fhirpath evaluate "Bundle.entry.resource.where(resourceType='Patient' and active=true)" \
  --input patient-bundle.json
```

**Extract vital signs:**
```bash
octofhir-fhirpath evaluate "Bundle.entry.resource.where(resourceType='Observation' and category.coding.code='vital-signs').valueQuantity" \
  --input vitals-bundle.json
```

**Get patient medications:**
```bash
octofhir-fhirpath evaluate "Bundle.entry.resource.where(resourceType='MedicationRequest').medicationCodeableConcept.coding.display" \
  --input medication-bundle.json
```

### Data Validation

**Check required fields:**
```bash
octofhir-fhirpath validate "Patient.name.exists() and Patient.birthDate.exists()"
```

**Validate date formats:**
```bash
octofhir-fhirpath evaluate "Patient.birthDate.matches('[0-9]{4}-[0-9]{2}-[0-9]{2}')" \
  --input patient.json
```

### Data Transformation

**Extract patient summary:**
```bash
octofhir-fhirpath evaluate "{
  name: Patient.name.given.first() + ' ' + Patient.name.family,
  age: today().toString().substring(0,4).toInteger() - Patient.birthDate.substring(0,4).toInteger(),
  gender: Patient.gender
}" --input patient.json
```

This comprehensive CLI reference covers all major use cases for the octofhir-fhirpath command-line interface. For additional help, use `octofhir-fhirpath help` or refer to the [Examples Guide](EXAMPLES.md).