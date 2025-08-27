# FHIRPath CLI Reference

The `octofhir-fhirpath` command-line tool provides a powerful interface for evaluating FHIRPath expressions against FHIR data.

## Installation

```bash
cargo install octofhir-fhirpath
```

## Quick Start

```bash
# Basic evaluation with piped data
echo '{"resourceType":"Patient","active":true}' | \
  octofhir-fhirpath evaluate "Patient.active"

# Evaluation with input file
octofhir-fhirpath evaluate "Patient.name.given" --input patient.json

# Parse expression to see AST
octofhir-fhirpath parse "Patient.name.where(use='official').family"

# Validate expression syntax
octofhir-fhirpath validate "Patient.birthDate > @2000-01-01"
```

## Commands

### `evaluate` - Evaluate FHIRPath Expression

Evaluates a FHIRPath expression against FHIR data and returns the result.

```bash
octofhir-fhirpath evaluate [OPTIONS] <EXPRESSION>
```

#### Arguments
- `<EXPRESSION>` - The FHIRPath expression to evaluate

#### Options
- `-i, --input <FILE>` - Input JSON file (if not provided, reads from stdin)
- `-v, --variable <NAME=VALUE>` - Set environment variable (can be used multiple times)
- `-p, --pretty` - Pretty-print JSON output  
- `-f, --format <FORMAT>` - Output format: `json` (default), `yaml`, `table`
- `--model <MODEL>` - FHIR model version: `r4` (default), `r5`, `mock`
- `--timeout <SECONDS>` - Evaluation timeout (default: 30)

#### Examples

**Basic evaluation:**
```bash
octofhir-fhirpath evaluate "Patient.active" --input patient.json
```

**With environment variables:**
```bash
octofhir-fhirpath evaluate "age > %minAge and age < %maxAge" \
  --input patient.json \
  --variable "minAge=18" \
  --variable "maxAge=65"
```

**Complex filtering:**
```bash
octofhir-fhirpath evaluate \
  "Bundle.entry.resource.where(resourceType='Patient' and active=true).name.given" \
  --input bundle.json \
  --pretty
```

**Piped input:**
```bash
curl -s "https://api.example.com/Patient/123" | \
  octofhir-fhirpath evaluate "Patient.name.where(use='official').family.first()"
```

### `parse` - Parse Expression to AST

Parses a FHIRPath expression and displays the Abstract Syntax Tree.

```bash
octofhir-fhirpath parse [OPTIONS] <EXPRESSION>
```

#### Arguments
- `<EXPRESSION>` - The FHIRPath expression to parse

#### Options
- `-f, --format <FORMAT>` - Output format: `tree` (default), `json`, `debug`
- `--show-positions` - Show token positions in output

#### Examples

```bash
# Parse simple expression
octofhir-fhirpath parse "Patient.name.given"

# Parse complex expression with JSON output
octofhir-fhirpath parse "Patient.name.where(use='official')" --format json

# Show debug information
octofhir-fhirpath parse "Patient.birthDate > @2000-01-01" --format debug
```

### `validate` - Validate Expression Syntax

Validates FHIRPath expression syntax without evaluation.

```bash
octofhir-fhirpath validate [OPTIONS] <EXPRESSION>
```

#### Arguments
- `<EXPRESSION>` - The FHIRPath expression to validate

#### Options
- `-v, --verbose` - Show detailed validation information
- `--strict` - Use strict validation mode

#### Examples

```bash
# Basic validation
octofhir-fhirpath validate "Patient.name.given"

# Verbose validation
octofhir-fhirpath validate "Patient.name.where(use='official')" --verbose

# Validate multiple expressions from file
cat expressions.txt | xargs -I {} octofhir-fhirpath validate "{}"
```

### `analyze` - Analyze Expression

Analyzes FHIRPath expressions for potential issues, performance characteristics, and optimization suggestions.

```bash
octofhir-fhirpath analyze [OPTIONS] <EXPRESSION>
```

#### Arguments
- `<EXPRESSION>` - The FHIRPath expression to analyze

#### Options
- `--validate-only` - Only validate, don't analyze
- `--show-optimizations` - Show optimization suggestions
- `--performance` - Include performance analysis
- `--resource-type <TYPE>` - Specify target resource type for better analysis

#### Examples

```bash
# Basic analysis
octofhir-fhirpath analyze "Patient.name.where(use='official').family"

# Performance analysis
octofhir-fhirpath analyze "Bundle.entry.resource.count()" --performance

# With resource type context
octofhir-fhirpath analyze "name.given" --resource-type Patient
```

## Environment Variables

FHIRPath supports environment variables in expressions using the `%variable` syntax.

### Standard Environment Variables

These are automatically available:
- `%context` - The original input node
- `%resource` - The containing resource  
- `%rootResource` - The root container resource
- `%sct` - SNOMED CT URL (`http://snomed.info/sct`)
- `%loinc` - LOINC URL (`http://loinc.org`)
- `%ucum` - UCUM URL (`http://unitsofmeasure.org`)

### Custom Environment Variables

Set using the `--variable` option:

```bash
# String values
octofhir-fhirpath evaluate "Patient.birthDate > %cutoffDate" \
  --input patient.json \
  --variable "cutoffDate=2000-01-01"

# Numeric values  
octofhir-fhirpath evaluate "age > %minAge" \
  --input patient.json \
  --variable "minAge=18"

# JSON values
octofhir-fhirpath evaluate "%config.enabled" \
  --input patient.json \
  --variable 'config={"enabled": true, "threshold": 100}'

# Multiple variables
octofhir-fhirpath evaluate "age between %min and %max" \
  --input patient.json \
  --variable "min=18" \
  --variable "max=65"
```

## Input Formats

### JSON Files
```bash
octofhir-fhirpath evaluate "Patient.active" --input patient.json
```

### YAML Files (auto-detected by extension)
```bash
octofhir-fhirpath evaluate "Patient.active" --input patient.yaml
```

### Standard Input (JSON)
```bash
echo '{"resourceType":"Patient","active":true}' | \
  octofhir-fhirpath evaluate "Patient.active"

cat patient.json | octofhir-fhirpath evaluate "Patient.name.given"
```

### From URLs (using curl)
```bash
curl -s "https://api.example.com/Patient/123" | \
  octofhir-fhirpath evaluate "Patient.name.family"
```

## Output Formats

### JSON (default)
```bash
octofhir-fhirpath evaluate "Patient.name.given" --input patient.json
# Output: ["Alice","Bob"]
```

### Pretty JSON
```bash
octofhir-fhirpath evaluate "Patient" --input patient.json --pretty
# Output: 
# [
#   {
#     "resourceType": "Patient",
#     "name": [...]
#   }
# ]
```

### YAML
```bash
octofhir-fhirpath evaluate "Patient.name" --input patient.json --format yaml
# Output:
# - given:
#   - Alice
#   family: Smith
```

### Table (for simple values)
```bash
octofhir-fhirpath evaluate "Patient.name.given" --input patient.json --format table
# Output:
# ┌─────────┐
# │ Value   │
# ├─────────┤
# │ Alice   │
# │ Bob     │
# └─────────┘
```

## Model Providers

### Mock Provider (default)
Fast, basic provider for simple use cases:
```bash
octofhir-fhirpath evaluate "Patient.active" --model mock --input patient.json
```

### FHIR R4 Provider
Full FHIR R4 schema support with type checking:
```bash
octofhir-fhirpath evaluate "Patient.active is Boolean" --model r4 --input patient.json
```

### FHIR R5 Provider  
Latest FHIR R5 schema support:
```bash
octofhir-fhirpath evaluate "Patient.active is Boolean" --model r5 --input patient.json
```

## Advanced Examples

### Healthcare Data Analysis
```bash
# Find active patients over 18
octofhir-fhirpath evaluate \
  "Bundle.entry.resource.where(resourceType='Patient' and active=true and birthDate < today()-18 'years').count()" \
  --input bundle.json

# Get all medication names
octofhir-fhirpath evaluate \
  "Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().code.coding.display" \
  --input bundle.json \
  --model r5
```

### Data Validation
```bash
# Check required fields
octofhir-fhirpath evaluate \
  "Patient.name.exists() and Patient.birthDate.exists()" \
  --input patient.json

# Validate email format
octofhir-fhirpath evaluate \
  "Patient.telecom.where(system='email').value.matches('[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}')" \
  --input patient.json
```

### Batch Processing
```bash
# Process multiple files
find ./patients -name "*.json" -exec \
  octofhir-fhirpath evaluate "Patient.name.family" --input {} \;

# With variable substitution
for age in 18 21 65; do
  echo "Age $age:"
  octofhir-fhirpath evaluate "Patient.birthDate < today()-$age 'years'" \
    --input patients.json \
    --variable "age=$age"
done
```

### Performance Testing
```bash
# Time evaluation
time octofhir-fhirpath evaluate \
  "Bundle.entry.resource.count()" \
  --input large-bundle.json

# With timeout
octofhir-fhirpath evaluate \
  "complex.nested.expression" \
  --input data.json \
  --timeout 60
```

## Error Handling

The CLI provides detailed error messages:

### Syntax Errors
```bash
$ octofhir-fhirpath evaluate "Patient.name."
Error: Parse error at position 13
  Patient.name.
               ^
Expected: identifier
```

### Evaluation Errors
```bash
$ octofhir-fhirpath evaluate "Patient.invalid" --input patient.json
Error: Path 'invalid' not found on Patient resource
Available paths: active, birthDate, gender, name, ...
```

### Type Errors (with schema providers)
```bash
$ octofhir-fhirpath evaluate "Patient.active + 1" --input patient.json --model r4
Error: Cannot add Boolean and Integer
  Patient.active + 1
                 ^
```

## Exit Codes

- `0` - Success
- `1` - Expression evaluation error  
- `2` - Parse error
- `3` - Input/output error
- `4` - Invalid arguments
- `5` - Timeout

## Configuration

### Environment Variables
- `FHIRPATH_MODEL` - Default model provider (`mock`, `r4`, `r5`)
- `FHIRPATH_TIMEOUT` - Default timeout in seconds
- `FHIRPATH_FORMAT` - Default output format
- `NO_COLOR` - Disable colored output

### Configuration File
Create `~/.fhirpath.yaml`:
```yaml
model: r5
timeout: 60
format: json
pretty: true
variables:
  defaultMinAge: 18
  organization: "Acme Healthcare"
```

## Tips and Best Practices

### Performance
- Use `--model mock` for simple expressions on large datasets
- Use `--timeout` for complex expressions on large bundles  
- Profile expressions with the `analyze` command

### Debugging
- Use `parse` to understand expression structure
- Use `validate` to check syntax before evaluation
- Use `--verbose` flags for detailed error information

### Security
- Be careful with `--variable` when processing untrusted input
- Use appropriate timeouts for user-provided expressions
- Validate input JSON before processing

### Scripting
- Check exit codes in scripts
- Use `--format table` for human-readable output
- Use `--format json` for programmatic processing

## Integration Examples

### Bash Scripts
```bash
#!/bin/bash
# Check if patient is active
if octofhir-fhirpath evaluate "Patient.active" --input "$1" | grep -q true; then
  echo "Patient is active"
else
  echo "Patient is inactive"
fi
```

### Python Integration
```python
import subprocess
import json

def evaluate_fhirpath(expression, data):
    process = subprocess.run([
        'octofhir-fhirpath', 'evaluate', expression
    ], input=json.dumps(data), text=True, capture_output=True)
    
    if process.returncode == 0:
        return json.loads(process.stdout)
    else:
        raise Exception(f"FHIRPath error: {process.stderr}")

# Usage
patient = {"resourceType": "Patient", "active": True}
result = evaluate_fhirpath("Patient.active", patient)
print(result)  # [True]
```

### CI/CD Validation
```yaml
# GitHub Actions example
- name: Validate FHIR data
  run: |
    for file in data/*.json; do
      octofhir-fhirpath validate "Patient.name.exists() and Patient.birthDate.exists()" --input "$file"
    done
```

For more examples and advanced usage, see the [examples directory](examples/) in the repository.