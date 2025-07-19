# FhirPath Library Comparison

This project compares popular FhirPath libraries across different programming languages to evaluate their performance, features, and compatibility.

## Libraries Under Comparison

### JavaScript/TypeScript
- **fhirpath.js** - The reference implementation from HL7
  - Repository: https://github.com/HL7/fhirpath.js
  - Language: JavaScript/TypeScript
  - Maintained by: HL7 FHIR community

### Java
- **HAPI FHIR FhirPath** - Part of the HAPI FHIR library
  - Repository: https://github.com/hapifhir/hapi-fhir
  - Language: Java
  - Maintained by: HAPI FHIR team

### .NET/C#
- **Firely .NET SDK** - FhirPath implementation in .NET
  - Repository: https://github.com/FirelyTeam/firely-net-sdk
  - Language: C#
  - Maintained by: Firely team

### Python
- **fhirpath-py** - Python implementation of FhirPath
  - Repository: https://github.com/beda-software/fhir-py
  - Language: Python
  - Maintained by: Beda Software

### Rust
- **fhirpath-core** - This implementation
  - Repository: Current project
  - Language: Rust
  - Maintained by: Project maintainers

### Go
- **go-fhirpath** - Go implementation (if available)
  - Language: Go
  - Status: To be researched

## Comparison Criteria

### Performance Metrics
- Expression parsing time
- Expression evaluation time
- Memory usage
- Throughput (expressions per second)

### Feature Compatibility
- FhirPath specification compliance
- Supported functions
- Error handling
- Type system support

### Ease of Use
- API design
- Documentation quality
- Installation process
- Integration complexity

## Test Cases

The comparison will use standardized test cases including:
- Official FhirPath test suite
- Performance benchmarks with various FHIR resources
- Complex expression evaluation
- Error handling scenarios

## Project Structure

```
fhirpath-comparison/
├── README.md                 # This file
├── SETUP.md                 # Detailed setup and usage guide
├── test-data/               # Common test data and FHIR resources
├── test-cases/              # Standardized test cases
├── implementations/         # Language-specific implementations
│   ├── javascript/          # Node.js with fhirpath.js
│   ├── java/               # Java with HAPI FHIR
│   ├── csharp/             # C# with Hl7.FhirPath
│   ├── python/             # Python with fhirpath-py
│   ├── rust/               # Rust with fhirpath-core
│   └── go/                 # Go implementation (planned)
├── results/                 # Comparison results and reports
├── visualization/           # Interactive HTML dashboard
│   ├── index.html          # Main visualization interface
│   └── visualization.js    # Data processing and charts
└── scripts/                 # Automation scripts
```

## Features

### 🚀 **Multi-Language Support**
- **JavaScript/TypeScript**: Official fhirpath.js implementation
- **Java**: HAPI FHIR with comprehensive FHIRPath support
- **C#**: Hl7.FhirPath official .NET implementation
- **Python**: fhirpath-py library
- **Rust**: fhirpath-core (this project's implementation)

### 📊 **Interactive Visualization**
- **Real-time Dashboard**: Modern web interface with interactive charts
- **Performance Comparison**: Visual comparison of execution times across languages
- **Test Results Analysis**: Detailed breakdown of test success/failure rates
- **Filtering & Export**: Filter results by language, date range, and export reports
- **Responsive Design**: Works on desktop, tablet, and mobile devices

### 🧪 **Comprehensive Testing**
- **Official Test Suite**: Uses HL7 FHIR official FHIRPath test cases
- **Performance Benchmarks**: Standardized performance testing across implementations
- **Automated Execution**: One-command setup and execution
- **Detailed Reporting**: JSON and HTML reports with metrics and analysis

## Getting Started

1. Install dependencies for each language implementation
2. Run the setup scripts to prepare test environments
3. Execute benchmarks and compatibility tests
4. Generate comparison reports

## Contributing

Contributions are welcome! Please see the main project's CONTRIBUTING.md for guidelines.
