# FHIRPath Library Comparison - Setup Guide

This guide will help you set up and run the FHIRPath library comparison project.

## Prerequisites

Before running the comparison, ensure you have the following tools installed:

### Required Tools

- **Python 3.8+** - For the main orchestration script
- **Node.js 16+** - For JavaScript implementation
- **Java 11+** and **Maven 3.6+** - For Java implementation
- **.NET 6.0+** - For C# implementation
- **Rust 1.70+** and **Cargo** - For Rust implementation

### Optional Tools

- **Go 1.19+** - For Go implementation (if available)

## Quick Start

1. **Clone and navigate to the comparison project:**
   ```bash
   cd fhirpath-comparison
   ```

2. **Run the setup for all implementations:**
   ```bash
   python3 scripts/run-comparison.py --setup-only
   ```

3. **Run tests and benchmarks:**
   ```bash
   python3 scripts/run-comparison.py
   ```

## Manual Setup by Language

### JavaScript Setup

```bash
cd implementations/javascript
npm install
```

**Dependencies installed:**
- `fhirpath` - Official FHIRPath.js library
- `xml2js` - XML parsing for test data
- `benchmark` - Performance benchmarking

### Python Setup

```bash
cd implementations/python
pip install -r requirements.txt
```

**Dependencies installed:**
- `fhirpath-py` - Python FHIRPath implementation
- `lxml` - XML processing
- `pytest` - Testing framework
- `pytest-benchmark` - Benchmarking

### Java Setup

```bash
cd implementations/java
mvn compile
```

**Dependencies managed by Maven:**
- HAPI FHIR libraries with FHIRPath support
- JUnit 5 for testing
- Jackson for JSON processing

### C# Setup

```bash
cd implementations/csharp
dotnet restore
```

**Dependencies managed by NuGet:**
- `Hl7.Fhir.R4` - Official .NET FHIR library
- `Hl7.FhirPath` - FHIRPath implementation
- `BenchmarkDotNet` - Performance benchmarking
- `xUnit` - Testing framework

### Rust Setup

```bash
cd implementations/rust
cargo build
```

**Dependencies managed by Cargo:**
- Local `fhirpath-core` - The fhirpath-core implementation
- `criterion` - Benchmarking framework
- `serde` - Serialization
- `quick-xml` - XML processing

## Running Comparisons

### Basic Usage

```bash
# Run all tests and benchmarks for all languages
python3 scripts/run-comparison.py

# Run only specific languages
python3 scripts/run-comparison.py --languages javascript python rust

# Run only tests (skip benchmarks)
python3 scripts/run-comparison.py --tests-only

# Run only benchmarks (skip tests)
python3 scripts/run-comparison.py --benchmarks-only
```

### Individual Language Testing

You can also run tests for individual languages:

```bash
# JavaScript
cd implementations/javascript
node test-runner.js

# Python (when implemented)
cd implementations/python
python test_runner.py

# Java (when implemented)
cd implementations/java
mvn test

# C# (when implemented)
cd implementations/csharp
dotnet test

# Rust (when implemented)
cd implementations/rust
cargo test
```

## Test Data

The comparison uses official FHIRPath test data located in `test-data/`:

- `patient-example.xml` - Sample patient resource
- `observation-example.xml` - Sample observation resource
- `questionnaire-example.xml` - Sample questionnaire resource
- `valueset-example-expansion.xml` - Sample value set resource

## Test Cases

Test cases are defined in `test-cases/`:

- `tests-fhir-r4.xml` - Official FHIRPath test suite (XML format)
- `test-config.json` - Standardized test configuration (JSON format)

The JSON configuration includes:
- Sample tests for basic functionality
- Benchmark tests for performance comparison
- Test categories for organization

## Results

Results are saved in the `results/` directory:

- `comparison_report_[timestamp].json` - Complete comparison report
- `[language]_test_results_[timestamp].json` - Individual test results
- `[language]_benchmark_results_[timestamp].json` - Individual benchmark results

## Interactive Visualization

The project includes a comprehensive web-based visualization dashboard for analyzing results.

### Accessing the Dashboard

After running tests, open the visualization dashboard:

```bash
# Open the visualization in your default browser
open fhirpath-comparison/visualization/index.html

# Or navigate to the file manually
cd fhirpath-comparison/visualization
# Then open index.html in any modern web browser
```

### Dashboard Features

#### 📊 **Performance Overview**
- Total tests executed across all languages
- Overall success rate percentage
- Average execution time metrics
- Total benchmarks completed

#### 📈 **Interactive Charts**
- **Performance Comparison**: Bar charts showing execution times by language
- **Test Results**: Stacked bar charts showing passed/failed/error counts
- **Benchmark Analysis**: Detailed performance metrics visualization

#### 🔍 **Filtering & Analysis**
- **Language Filter**: Show/hide specific programming languages
- **Date Range**: Filter results by execution date
- **Real-time Updates**: Dynamic chart updates based on filters

#### 📋 **Detailed Tables**
- **Test Results**: Complete test execution details with status and timing
- **Benchmark Results**: Performance metrics including min/max/average times
- **System Information**: Platform and version details for each implementation

#### 📥 **Export Functionality**
- Export filtered results as JSON reports
- Comprehensive analysis data for further processing
- Timestamped report generation

### Using the Visualization

1. **Run Comparison Tests**:
   ```bash
   python3 scripts/run-comparison.py
   ```

2. **Open Dashboard**:
   ```bash
   open visualization/index.html
   ```

3. **Analyze Results**:
   - Use language filters to compare specific implementations
   - Examine performance charts to identify fastest/slowest languages
   - Review detailed tables for specific test case analysis
   - Export reports for documentation or further analysis

4. **Refresh Data**:
   - Click "🔄 Refresh Data" button to reload latest results
   - Run new tests and refresh to see updated comparisons

### Browser Compatibility

The visualization dashboard works with modern web browsers:
- ✅ Chrome 80+
- ✅ Firefox 75+
- ✅ Safari 13+
- ✅ Edge 80+

**Note**: Due to browser security restrictions, the dashboard works best when served from a local web server. For local file access, some browsers may require enabling local file access or using developer mode.

## Troubleshooting

### Common Issues

1. **Missing dependencies:**
   - Ensure all required tools are installed and in PATH
   - Run setup commands for each language individually

2. **Test data not found:**
   - Verify test data files exist in `test-data/` directory
   - Check file permissions

3. **Permission errors:**
   - Ensure scripts are executable: `chmod +x scripts/run-comparison.py`
   - Check write permissions for `results/` directory

### Language-Specific Issues

**JavaScript:**
- Ensure Node.js version is 16 or higher
- Try `npm cache clean --force` if installation fails

**Python:**
- Use virtual environment to avoid conflicts
- Ensure pip is up to date: `pip install --upgrade pip`

**Java:**
- Verify JAVA_HOME is set correctly
- Ensure Maven is configured properly

**C#:**
- Verify .NET SDK is installed (not just runtime)
- Try `dotnet clean` and `dotnet restore`

**Rust:**
- Ensure Rust toolchain is up to date: `rustup update`
- Try `cargo clean` if build fails

## Adding New Languages

To add support for a new language:

1. Create directory: `implementations/[language]/`
2. Add configuration file (package.json, requirements.txt, etc.)
3. Implement test runner following the standardized output format
4. Update `scripts/run-comparison.py` to include setup logic
5. Update documentation

See existing implementations as examples for the expected structure and output format.

## Performance Considerations

- Large test suites may take significant time to complete
- Benchmark iterations can be adjusted in `test-config.json`
- Consider running subsets of tests during development
- Results may vary based on system load and hardware

## Next Steps

After setup, you can:
1. Run the full comparison suite
2. Analyze results in the generated reports
3. Add custom test cases
4. Implement additional language support
5. Contribute improvements to the framework
