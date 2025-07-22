# Clojure FHIRPath Implementation

This directory contains the Clojure implementation for the FHIRPath comparison project using the [fhirpath.clj](https://github.com/HealthSamurai/fhirpath.clj) library by HealthSamurai.

## Prerequisites

- **Clojure CLI Tools** - Install from [clojure.org](https://clojure.org/guides/getting_started)
- **Java 8+** - Required for running Clojure on the JVM

### Installing Clojure CLI Tools

#### macOS
```bash
brew install clojure/tools/clojure
```

#### Linux
```bash
curl -O https://download.clojure.org/install/linux-install-1.11.1.1413.sh
chmod +x linux-install-1.11.1.1413.sh
sudo ./linux-install-1.11.1.1413.sh
```

#### Windows
Download and run the installer from [clojure.org](https://clojure.org/guides/getting_started#_installation_on_windows)

## Setup

1. **Install dependencies:**
   ```bash
   clojure -P
   ```
   This downloads all dependencies specified in `deps.edn`.

2. **Verify installation:**
   ```bash
   clojure -M -m test-runner --help
   ```

## Usage

### Running Tests
```bash
# Run all FHIRPath tests
clojure -M -m test-runner

# Run with the orchestration script
cd ../..
python3 scripts/run-comparison.py --languages clojure
```

### Running Benchmarks
```bash
# Run performance benchmarks
clojure -M -m test-runner benchmark

# Run with the orchestration script
cd ../..
python3 scripts/run-comparison.py --languages clojure --benchmark
```

## Dependencies

The implementation uses the following key dependencies:

- **fhirpath.clj** - The main FHIRPath library by HealthSamurai
- **org.clojure/data.json** - JSON processing for test data and results
- **org.clojure/data.xml** - XML processing for test cases
- **clj-commons/clj-yaml** - YAML processing support

## Library Information

- **Library**: [fhirpath.clj](https://github.com/HealthSamurai/fhirpath.clj)
- **Maintainer**: [HealthSamurai](https://www.health-samurai.io/)
- **Language**: Clojure (JVM-based)
- **Based on**: Reference fhirpath.js implementation
- **Parser**: ANTLR-based (similar to other implementations)

## API Usage

The library provides a simple evaluation API:

```clojure
(require '[fhirpath.core :as fp])

;; Evaluate a FHIRPath expression against FHIR data
(fp/fp "Patient.name.given" patient-data)

;; With environment variables
(fp/fp "%weight * 2.2" {} {:weight 70})
```

## Test Results

Test results are saved in JSON format to `../../results/clojure_test_results_<timestamp>.json` with the following structure:

```json
{
  "metadata": {
    "language": "clojure",
    "library": "fhirpath.clj",
    "version": "latest",
    "timestamp": "2024-01-01T12:00:00Z",
    "testSuite": "official-fhir-r4",
    "totalTests": 100,
    "successfulTests": 95,
    "successRate": 0.95
  },
  "results": [
    {
      "name": "test-name",
      "expression": "Patient.name",
      "result": [...],
      "expected": [...],
      "success": true,
      "executionTimeMs": 1.23,
      "error": null
    }
  ]
}
```

## Troubleshooting

### Common Issues

1. **Clojure CLI not found**
   - Ensure Clojure CLI tools are installed and in your PATH
   - Verify with `clojure --version`

2. **Java version issues**
   - Ensure Java 8+ is installed
   - Check with `java -version`

3. **Dependency download failures**
   - Check internet connection
   - Try clearing the cache: `rm -rf ~/.clojure/.cpcache`

4. **Test data not found**
   - Ensure you're running from the correct directory
   - Test data should be in `../../test-data/`

### Getting Help

- Check the [fhirpath.clj repository](https://github.com/HealthSamurai/fhirpath.clj) for library-specific issues
- Review the main project's [CONTRIBUTING.md](../../CONTRIBUTING.md) for general guidance
- Open an issue in the main project repository for integration problems

## Contributing

Contributions to improve the Clojure implementation are welcome! Please follow the main project's contribution guidelines and ensure:

1. Tests pass with the official FHIRPath test suite
2. Code follows Clojure style conventions
3. Documentation is updated as needed
4. Performance benchmarks are maintained or improved
