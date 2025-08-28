# FHIRPath CLI Reference

The `octofhir-fhirpath` command-line tool provides a comprehensive interface for evaluating FHIRPath expressions against FHIR data, featuring an interactive REPL, multiple output formats, web interface, and advanced analysis capabilities.

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

## Global Options

All commands support these global options:

- `--fhir-version <VERSION>` - FHIR version to use (r4, r4b, r5) [default: r4]
- `--package <PACKAGE>` - Additional FHIR packages to load (format: package@version)
- `-o, --output-format <FORMAT>` - Output format: `raw`, `pretty`, `json`, `table` [default: raw]
- `--no-color` - Disable colored output (also via `FHIRPATH_NO_COLOR` env var)
- `-q, --quiet` - Suppress informational messages
- `-v, --verbose` - Verbose output with additional details

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
- `-p, --pretty` - Pretty-print JSON output (only applies to raw format)

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

Analyzes FHIRPath expressions with comprehensive FHIR field validation, type checking, and optimization suggestions.

```bash
octofhir-fhirpath analyze [OPTIONS] <EXPRESSION>
```

#### Arguments
- `<EXPRESSION>` - The FHIRPath expression to analyze

#### Options
- `-v, --variable <NAME=VALUE>` - Set environment variable (can be used multiple times)
- `--validate-only` - Only validate, don't analyze types
- `--no-inference` - Disable type inference

### `repl` - Interactive FHIRPath REPL

Starts an interactive Read-Eval-Print Loop for rapid FHIRPath prototyping and debugging.

```bash
octofhir-fhirpath repl [OPTIONS]
```

#### Options
- `-i, --input <FILE>` - JSON file containing FHIR resource to load initially
- `-v, --variable <NAME=VALUE>` - Set environment variable (can be used multiple times)
- `--history-file <FILE>` - History file to use (default: ~/.fhirpath_history)
- `--history-size <SIZE>` - Maximum number of history entries [default: 1000]

### `server` - HTTP Server with Web Interface

Starts an HTTP server with a web-based FHIRPath evaluation interface.

```bash
octofhir-fhirpath server [OPTIONS]
```

#### Options
- `-p, --port <PORT>` - Port to bind the server to [default: 8080]
- `-s, --storage <DIR>` - Directory for JSON file storage [default: ./storage]
- `--host <HOST>` - Host to bind to [default: 127.0.0.1]
- `--cors-all` - Enable CORS for all origins (development mode)

#### Examples

```bash
# Basic analysis with type checking
octofhir-fhirpath analyze "Patient.name.where(use='official').family"

# Validation only
octofhir-fhirpath analyze "Bundle.entry.resource.count()" --validate-only

# With variables
octofhir-fhirpath analyze "age > %minAge" --variable "minAge=18"
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

## Interactive REPL

The FHIRPath REPL provides an interactive environment for rapid prototyping and debugging of FHIRPath expressions.

### Starting the REPL

```bash
# Start REPL
octofhir-fhirpath repl

# Start REPL with initial resource
octofhir-fhirpath repl --input patient.json

# Start REPL with specific FHIR version
octofhir-fhirpath repl --fhir-version r5

# Start REPL with initial variables
octofhir-fhirpath repl --variable "minAge=18" --variable "organization=Acme"
```

### REPL Commands

All REPL commands start with `:` (colon):

- `<expression>` - Evaluate any FHIRPath expression
- `:load <file>` - Load FHIR resource from file
- `:set <name> <value>` - Set variable value
- `:unset <name>` - Remove variable
- `:vars` - List all variables and context
- `:resource` - Show current resource information
- `:type <expression>` - Show type information for expression
- `:explain <expression>` - Show evaluation steps and analysis
- `:help [function]` - Show help for commands or functions
- `:history` - Show command history
- `:quit` - Exit REPL (aliases: `:q`, `:exit`)

### REPL Features

- **Interactive line editing** with history and arrow key navigation
- **Auto-completion** for function names and FHIR properties
- **Colored output** for better readability
- **Variable management** for complex expression building
- **Resource loading** from JSON files
- **Command history** with persistent storage
- **Help system** with comprehensive function documentation
- **Error handling** with clear, actionable messages

### Example REPL Session

```
$ octofhir-fhirpath repl
FHIRPath REPL v0.4.x - Type :help for commands

fhirpath> :load examples/patient.json
Loaded Patient resource (id: example-1)

fhirpath> Patient.name.given.first()
"John"

fhirpath> :set myVar "test"
Variable 'myVar' set to "test"

fhirpath> :vars
%context = Patient resource (id: example-1)
%resource = Patient resource (id: example-1)
myVar = "test"

fhirpath> Patient.name.where(use = 'official').family
["Doe"]

fhirpath> :help first
first() - Returns the first item in a collection
Usage: collection.first()
Returns: single item or empty if collection is empty

Examples:
  Patient.name.first()
  telecom.first().value

fhirpath> :quit
Goodbye!
```

## Web Server Interface

The server command starts an HTTP server with a web-based FHIRPath evaluation interface.

```bash
# Start server on default port 8080
octofhir-fhirpath server

# Start server on custom port with CORS enabled
octofhir-fhirpath server --port 3000 --cors-all

# Start server with custom storage directory
octofhir-fhirpath server --storage ./my-fhir-data
```

### Server Features

- **Web-based interface** for FHIRPath evaluation
- **File management** for FHIR resources
- **Real-time evaluation** with syntax highlighting
- **Multiple output formats** (JSON, table, pretty)
- **CORS support** for development integration
- **REST API** for programmatic access

### Server Endpoints

- `GET /` - Web interface
- `POST /evaluate` - Evaluate FHIRPath expression
- `GET /files` - List stored files
- `POST /files` - Upload FHIR resource
- `GET /files/{id}` - Get specific file
- `DELETE /files/{id}` - Delete file

## Output Formats

### Raw (default)
```bash
octofhir-fhirpath evaluate "Patient.name.given" --input patient.json
# Output: ["Alice","Bob"]
```

### Pretty Format
Colorized, emoji-rich output with execution metrics:
```bash
octofhir-fhirpath evaluate "Patient.name.given" --output-format pretty --input patient.json
# Output with colors and emojis:
# 🎯 Expression: Patient.name.given
# ⚡ Result: ["Alice", "Bob"]
# ⏱️  Execution time: 1.2ms
# 💾 Memory used: 1.5KB
```

### JSON Format
Structured JSON output for machine parsing:
```bash
octofhir-fhirpath evaluate "Patient.name.given" --output-format json --input patient.json
# Output:
# {
#   "success": true,
#   "result": ["Alice", "Bob"],
#   "expression": "Patient.name.given",
#   "execution_time_ms": 1.2,
#   "metadata": {
#     "cache_hits": 0,
#     "ast_nodes": 3,
#     "memory_used": 1536
#   }
# }
```

### Table Format
Formatted table output for collections:
```bash
octofhir-fhirpath evaluate "Patient.name.given" --output-format table --input patient.json
# Output:
# ┌─────────┐
# │ Value   │
# ├─────────┤
# │ Alice   │
# │ Bob     │
# └─────────┘
```

## FHIR Version Support

### FHIR R4 (default)
Full FHIR R4 schema support with type checking:
```bash
octofhir-fhirpath evaluate "Patient.active is Boolean" --fhir-version r4 --input patient.json
```

### FHIR R4B
FHIR R4B schema support:
```bash
octofhir-fhirpath evaluate "Patient.active is Boolean" --fhir-version r4b --input patient.json
```

### FHIR R5
Latest FHIR R5 schema support:
```bash
octofhir-fhirpath evaluate "Patient.active is Boolean" --fhir-version r5 --input patient.json
```

### Additional FHIR Packages
Load additional FHIR packages for extended functionality:
```bash
octofhir-fhirpath evaluate "expression" --package "us.core@6.1.0" --input patient.json
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
- `FHIRPATH_OUTPUT_FORMAT` - Default output format (`raw`, `pretty`, `json`, `table`)
- `FHIRPATH_NO_COLOR` - Disable colored output (also `NO_COLOR`)
- `RUST_LOG` - Enable debug logging (`debug`, `info`, `warn`, `error`)


## Tips and Best Practices

### Performance
- Use the `analyze` command to profile expressions and get optimization suggestions
- Enable JSON streaming for large datasets
- Use the REPL for iterative development to avoid startup overhead

### Debugging
- Use `parse` to understand expression structure
- Use `validate` to check syntax before evaluation
- Use `--verbose` flag for detailed error information
- Use REPL `:explain` command for step-by-step evaluation
- Use REPL `:type` command for type information

### Security
- Be careful with `--variable` when processing untrusted input
- Use appropriate timeouts for user-provided expressions
- Validate input JSON before processing

### Scripting
- Check exit codes in scripts
- Use `--format table` for human-readable output
- Use `--format json` for programmatic processing

## Function Reference

The CLI provides comprehensive help for FHIRPath functions. Use the REPL's `:help` command:

```bash
# In REPL
fhirpath> :help first
first() - Returns the first item in a collection
Usage: collection.first()
Returns: single item or empty if collection is empty

Examples:
  Patient.name.first()
  telecom.first().value
```

### Common Functions

**Collection Functions:**
- `first()`, `last()`, `count()`, `length()`
- `where(condition)`, `select(expression)` 
- `exists()`, `empty()`, `single()`
- `skip(n)`, `take(n)`, `distinct()`

**String Functions:**
- `contains(substring)`, `startsWith(prefix)`, `endsWith(suffix)`
- `substring(start)`, `substring(start, length)`
- `upper()`, `lower()`, `replace(old, new)`

**Type Functions:**
- `is(Type)`, `as(Type)`, `ofType(Type)`
- `toString()`, `toInteger()`, `toDecimal()`

**FHIR Functions:**
- `resolve()`, `extension(url)`, `children()`
- `conformsTo(url)`, `memberOf(valueset)`

**Date/Time Functions:**
- `today()`, `now()`, `timeOfDay()`

**Logical Functions:**
- `iif(condition, true_val, false_val)`
- `all(condition)`, `any(condition)`

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

## Quick Reference Commands

Here are the most commonly used commands for quick reference:

```bash
# Basic evaluation
octofhir-fhirpath evaluate "Patient.name.given" --input patient.json

# Interactive REPL
octofhir-fhirpath repl --input patient.json

# Pretty output with colors
octofhir-fhirpath evaluate "Patient.name" --output-format pretty --input patient.json

# Table format for collections
octofhir-fhirpath evaluate "Patient.name.given" --output-format table --input patient.json

# JSON format with metadata
octofhir-fhirpath evaluate "Patient.active" --output-format json --input patient.json

# Parse expression to AST
octofhir-fhirpath parse "Patient.name.where(use='official').family"

# Validate syntax
octofhir-fhirpath validate "Patient.birthDate > @2000-01-01"

# Analyze with type checking
octofhir-fhirpath analyze "Patient.name.where(use='official').family"

# Start web server
octofhir-fhirpath server --port 8080

# FHIR R5 with additional packages
octofhir-fhirpath evaluate "Patient.active" --fhir-version r5 --package "us.core@6.1.0"
```

## Justfile Integration

If you're working with the source code, use the provided `justfile` commands:

```bash
# CLI evaluation (reads from stdin)
just cli-evaluate "Patient.name.given"
just cli-evaluate "Patient.name.given" patient.json

# Enhanced output formats
just cli-pretty "Patient.name" patient.json
just cli-json "Patient.name" patient.json  
just cli-table "Patient.name.given" patient.json

# Interactive REPL
just repl
just repl patient.json
just repl --fhir-version r5

# Analysis and parsing
just cli-parse "Patient.name.where(use='official')"
just cli-validate "Patient.birthDate > @2000-01-01"
just cli-analyze "Patient.name.given"
```

For more examples and advanced usage, see the [examples directory](examples/) and [CLAUDE.md](CLAUDE.md) development guide in the repository.