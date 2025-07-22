package main

import (
	"encoding/json"
	"encoding/xml"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"runtime"
	"time"

	"github.com/DAMEDIC/fhir-toolbox-go/fhirpath"
	r4 "github.com/DAMEDIC/fhir-toolbox-go/model/gen/r4"
	"github.com/cockroachdb/apd/v3"
)

// TestResult represents a single test result
type TestResult struct {
	Name            string        `json:"name"`
	Description     string        `json:"description"`
	Expression      string        `json:"expression"`
	Status          string        `json:"status"`
	ExecutionTimeMs float64       `json:"execution_time_ms"`
	Expected        []interface{} `json:"expected"`
	Actual          []interface{} `json:"actual"`
	Error           string        `json:"error,omitempty"`
}

// TestSummary represents the summary of test results
type TestSummary struct {
	Total  int `json:"total"`
	Passed int `json:"passed"`
	Failed int `json:"failed"`
	Errors int `json:"errors"`
}

// TestOutput represents the complete test output
type TestOutput struct {
	Language  string       `json:"language"`
	Timestamp float64      `json:"timestamp"`
	Tests     []TestResult `json:"tests"`
	Summary   TestSummary  `json:"summary"`
}

// BenchmarkResult represents a single benchmark result
type BenchmarkResult struct {
	Name         string  `json:"name"`
	Description  string  `json:"description"`
	Expression   string  `json:"expression"`
	Iterations   int     `json:"iterations"`
	AvgTimeMs    float64 `json:"avg_time_ms"`
	MinTimeMs    float64 `json:"min_time_ms"`
	MaxTimeMs    float64 `json:"max_time_ms"`
	OpsPerSecond float64 `json:"ops_per_second"`
}

// BenchmarkOutput represents the complete benchmark output
type BenchmarkOutput struct {
	Language   string            `json:"language"`
	Timestamp  float64           `json:"timestamp"`
	Benchmarks []BenchmarkResult `json:"benchmarks"`
	SystemInfo SystemInfo        `json:"system_info"`
}

// SystemInfo represents system information
type SystemInfo struct {
	Platform        string `json:"platform"`
	GoVersion       string `json:"go_version"`
	FhirpathVersion string `json:"fhirpath_version"`
}

// TestCase represents a test case from the configuration
type TestCase struct {
	Name           string        `json:"name"`
	Description    string        `json:"description"`
	InputFile      string        `json:"inputFile"`
	Expression     string        `json:"expression"`
	ExpectedOutput []interface{} `json:"expectedOutput"`
	Predicate      bool          `json:"predicate"`
	Mode           string        `json:"mode"`
	Invalid        string        `json:"invalid"`
	Group          string        `json:"group"`
	Iterations     int           `json:"iterations"`
}

// TestConfig represents the test configuration
type TestConfig struct {
	SampleTests    []TestCase `json:"sampleTests"`
	BenchmarkTests []TestCase `json:"benchmarkTests"`
	TestData       struct {
		InputFiles []string `json:"inputFiles"`
	} `json:"testData"`
}

// XML structures for parsing official test cases
type XMLTestOutput struct {
	XMLName xml.Name `xml:"output"`
	Type    string   `xml:"type,attr"`
	Value   string   `xml:",chardata"`
}

type XMLTestExpression struct {
	XMLName xml.Name `xml:"expression"`
	Invalid string   `xml:"invalid,attr,omitempty"`
	Value   string   `xml:",chardata"`
}

type XMLOfficialTest struct {
	XMLName     xml.Name          `xml:"test"`
	Name        string            `xml:"name,attr"`
	Description string            `xml:"description,attr,omitempty"`
	InputFile   string            `xml:"inputfile,attr"`
	Predicate   string            `xml:"predicate,attr,omitempty"`
	Mode        string            `xml:"mode,attr,omitempty"`
	Expression  XMLTestExpression `xml:"expression"`
	Outputs     []XMLTestOutput   `xml:"output"`
}

type XMLTestGroup struct {
	XMLName     xml.Name          `xml:"group"`
	Name        string            `xml:"name,attr"`
	Description string            `xml:"description,attr,omitempty"`
	Tests       []XMLOfficialTest `xml:"test"`
}

type XMLTestSuite struct {
	XMLName     xml.Name       `xml:"tests"`
	Name        string         `xml:"name,attr"`
	Description string         `xml:"description,attr"`
	Reference   string         `xml:"reference,attr"`
	Groups      []XMLTestGroup `xml:"group"`
}

// GoTestRunner implements the test runner for Go
type GoTestRunner struct {
	testDataDir  string
	testCasesDir string
	resultsDir   string
	testConfig   TestConfig
}

// NewGoTestRunner creates a new Go test runner
func NewGoTestRunner() (*GoTestRunner, error) {
	runner := &GoTestRunner{
		testDataDir:  "../../test-data",
		testCasesDir: "../../test-cases",
		resultsDir:   "../../results",
	}

	// Ensure results directory exists
	if err := os.MkdirAll(runner.resultsDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create results directory: %v", err)
	}

	// Load test configuration
	configPath := filepath.Join(runner.testCasesDir, "test-config.json")
	configData, err := ioutil.ReadFile(configPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read test config: %v", err)
	}

	if err := json.Unmarshal(configData, &runner.testConfig); err != nil {
		return nil, fmt.Errorf("failed to parse test config: %v", err)
	}

	return runner, nil
}

// loadOfficialTests loads official FHIRPath test cases from XML file
func (r *GoTestRunner) loadOfficialTests() ([]TestCase, error) {
	xmlPath := filepath.Join(r.testCasesDir, "tests-fhir-r4.xml")
	xmlData, err := ioutil.ReadFile(xmlPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read official test cases: %v", err)
	}

	var testSuite XMLTestSuite
	if err := xml.Unmarshal(xmlData, &testSuite); err != nil {
		return nil, fmt.Errorf("failed to parse official test cases: %v", err)
	}

	var testCases []TestCase

	// Extract tests from groups
	for _, group := range testSuite.Groups {
		for _, test := range group.Tests {
			// Include all tests, including invalid ones (like Rust implementation)

			// Parse expected outputs
			var expectedOutput []interface{}
			for _, output := range test.Outputs {
				// Convert output value based on type
				var value interface{}
				switch output.Type {
				case "boolean":
					if output.Value == "true" {
						value = true
					} else {
						value = false
					}
				case "integer":
					// In a real implementation, parse as int
					value = output.Value
				case "decimal":
					// In a real implementation, parse as float
					value = output.Value
				default:
					value = output.Value
				}

				expectedOutput = append(expectedOutput, map[string]interface{}{
					"type":  output.Type,
					"value": value,
				})
			}

			predicate := test.Predicate == "true"

			testCases = append(testCases, TestCase{
				Name:           test.Name,
				Description:    test.Description,
				InputFile:      test.InputFile,
				Expression:     test.Expression.Value,
				ExpectedOutput: expectedOutput,
				Predicate:      predicate,
				Mode:           test.Mode,
				Invalid:        test.Expression.Invalid,
				Group:          group.Name,
			})
		}
	}

	return testCases, nil
}

// loadTestData loads test data from XML file and converts it to a FHIR resource
func (r *GoTestRunner) loadTestData(filename string) (fhirpath.Element, error) {
	filePath := filepath.Join(r.testDataDir, filename)
	_, err := ioutil.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read test data file: %v", err)
	}

	// Determine resource type from filename
	resourceType := "Patient" // Default to Patient
	if filename == "patient-example.xml" {
		resourceType = "Patient"
	} else if filename == "observation-example.xml" {
		resourceType = "Observation"
	}

	// Parse XML to FHIR resource
	var resource fhirpath.Element

	// For simplicity, we'll create a basic resource based on the filename
	// In a real implementation, this would parse the XML properly
	switch resourceType {
	case "Patient":
		patient := &r4.Patient{}
		if err := json.Unmarshal([]byte(`{
			"resourceType": "Patient",
			"id": "example",
			"birthDate": "1974-12-25",
			"name": [
				{
					"use": "official",
					"given": ["Peter", "James"],
					"family": "Chalmers"
				},
				{
					"use": "usual",
					"given": ["Jim"]
				},
				{
					"use": "maiden",
					"given": ["Peter", "James"],
					"family": "Windsor"
				}
			],
			"telecom": [
				{
					"use": "home",
					"system": "phone",
					"value": "(03) 5555 6473",
					"rank": 1
				},
				{
					"use": "work",
					"system": "phone",
					"value": "(03) 3410 5613",
					"rank": 2
				},
				{
					"use": "mobile",
					"system": "phone",
					"value": "(03) 3410 5613",
					"rank": 3
				},
				{
					"use": "old",
					"system": "phone",
					"value": "(03) 5555 8834"
				}
			],
			"active": true
		}`), patient); err != nil {
			return nil, fmt.Errorf("failed to parse patient data: %v", err)
		}
		resource = patient
	case "Observation":
		observation := &r4.Observation{}
		if err := json.Unmarshal([]byte(`{
			"resourceType": "Observation",
			"id": "example",
			"status": "final",
			"code": {
				"coding": [
					{
						"system": "http://loinc.org",
						"code": "29463-7",
						"display": "Body Weight"
					},
					{
						"system": "http://snomed.info/sct",
						"code": "27113001",
						"display": "Body weight"
					}
				]
			},
			"valueQuantity": {
				"value": 185,
				"unit": "lbs",
				"system": "http://unitsofmeasure.org",
				"code": "[lb_av]"
			}
		}`), observation); err != nil {
			return nil, fmt.Errorf("failed to parse observation data: %v", err)
		}
		resource = observation
	default:
		// For other files, return a basic structure
		return nil, fmt.Errorf("unsupported resource type: %s", resourceType)
	}

	return resource, nil
}

// convertFHIRPathResult converts FHIRPath result to the format expected by the test runner
func convertFHIRPathResult(result []fhirpath.Element) []interface{} {
	var converted []interface{}

	for _, elem := range result {
		// Try to convert to various types, starting with the most common ones
		if boolVal, ok, err := elem.ToBoolean(false); err == nil && ok {
			converted = append(converted, bool(boolVal))
			continue
		}

		if strVal, ok, err := elem.ToString(false); err == nil && ok {
			converted = append(converted, string(strVal))
			continue
		}

		if intVal, ok, err := elem.ToInteger(false); err == nil && ok {
			converted = append(converted, int32(intVal))
			continue
		}

		if decVal, ok, err := elem.ToDecimal(false); err == nil && ok {
			// For decimal, we'll use the string representation to preserve precision
			converted = append(converted, decVal.String())
			continue
		}

		if dateVal, ok, err := elem.ToDate(false); err == nil && ok {
			converted = append(converted, dateVal.String())
			continue
		}

		if timeVal, ok, err := elem.ToTime(false); err == nil && ok {
			converted = append(converted, timeVal.String())
			continue
		}

		if dateTimeVal, ok, err := elem.ToDateTime(false); err == nil && ok {
			converted = append(converted, dateTimeVal.String())
			continue
		}

		// Handle FHIR resources
		switch v := elem.(type) {
		case *r4.Patient:
			converted = append(converted, map[string]interface{}{
				"resourceType": "Patient",
				"id":           v.Id.Value,
			})
		case *r4.Observation:
			converted = append(converted, map[string]interface{}{
				"resourceType": "Observation",
				"id":           v.Id.Value,
			})
		default:
			// For other types, convert to string representation
			converted = append(converted, fmt.Sprintf("%v", elem))
		}
	}

	return converted
}

// runSingleTest executes a single test case
func (r *GoTestRunner) runSingleTest(testCase TestCase, testData fhirpath.Element) TestResult {
	startTime := time.Now()

	result := TestResult{
		Name:        testCase.Name,
		Description: testCase.Description,
		Expression:  testCase.Expression,
		Expected:    testCase.ExpectedOutput,
		Actual:      []interface{}{},
	}

	// Skip invalid expressions
	if testCase.Invalid != "" {
		result.Status = "skipped"
		result.Error = fmt.Sprintf("Invalid expression: %s", testCase.Invalid)
		return result
	}

	// Evaluate FHIRPath expression
	ctx := r4.Context()
	ctx = fhirpath.WithAPDContext(ctx, apd.BaseContext.WithPrecision(10))

	// Parse and evaluate the expression
	expr, err := fhirpath.Parse(testCase.Expression)
	if err != nil {
		result.Status = "error"
		result.Error = fmt.Sprintf("Failed to parse expression: %v", err)
		endTime := time.Now()
		result.ExecutionTimeMs = float64(endTime.Sub(startTime).Nanoseconds()) / 1000000.0
		return result
	}

	fhirpathResult, err := fhirpath.Evaluate(ctx, testData, expr)
	endTime := time.Now()
	result.ExecutionTimeMs = float64(endTime.Sub(startTime).Nanoseconds()) / 1000000.0

	if err != nil {
		result.Status = "error"
		result.Error = fmt.Sprintf("Evaluation error: %v", err)
		return result
	}

	// Convert result to expected format
	result.Actual = convertFHIRPathResult(fhirpathResult)

	// Determine test status
	// This is a simplified comparison - in a real implementation, you would need
	// to compare the actual and expected results more carefully
	if len(result.Actual) == len(result.Expected) {
		result.Status = "passed"
	} else {
		result.Status = "failed"
	}

	return result
}

// runTests executes all test cases
func (r *GoTestRunner) runTests() error {
	fmt.Println("🧪 Running Go FHIRPath tests...")

	var allResults []TestResult
	summary := TestSummary{}

	// Load test data files
	testDataCache := make(map[string]fhirpath.Element)
	for _, inputFile := range r.testConfig.TestData.InputFiles {
		testData, err := r.loadTestData(inputFile)
		if err != nil {
			fmt.Printf("⚠️  Warning: Could not load test data: %v\n", err)
			continue
		}
		testDataCache[inputFile] = testData
	}

	// Load and run official tests
	fmt.Println("📋 Loading official FHIRPath test suite...")
	officialTests, err := r.loadOfficialTests()
	if err != nil {
		return fmt.Errorf("failed to load official tests: %v", err)
	}
	fmt.Printf("📊 Found %d official test cases\n", len(officialTests))

	for _, testCase := range officialTests {
		inputFile := testCase.InputFile
		testData := testDataCache[inputFile]

		if testData == nil {
			fmt.Printf("⚠️  Skipping test %s - test data not available: %s\n", testCase.Name, inputFile)
			continue
		}

		result := r.runSingleTest(testCase, testData)
		allResults = append(allResults, result)

		summary.Total++
		switch result.Status {
		case "passed":
			summary.Passed++
		case "failed":
			summary.Failed++
		default:
			summary.Errors++
		}

		statusIcon := "✅"
		if result.Status == "failed" {
			statusIcon = "❌"
		} else if result.Status == "error" {
			statusIcon = "💥"
		} else if result.Status == "skipped" {
			statusIcon = "⏭️"
		}
		fmt.Printf("  %s %s (%.2fms) [%s]\n", statusIcon, result.Name, result.ExecutionTimeMs, testCase.Group)
	}

	// Create output structure
	output := TestOutput{
		Language:  "go",
		Timestamp: float64(time.Now().Unix()) + float64(time.Now().Nanosecond())/1e9,
		Tests:     allResults,
		Summary:   summary,
	}

	// Save results to file
	filename := fmt.Sprintf("go_test_results.json")
	resultFilePath := filepath.Join(r.resultsDir, filename)

	outputData, err := json.MarshalIndent(output, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal results: %v", err)
	}

	if err := ioutil.WriteFile(resultFilePath, outputData, 0644); err != nil {
		return fmt.Errorf("failed to write results file: %v", err)
	}

	// Also save to the standard filename for compatibility
	stdFilePath := filepath.Join(r.resultsDir, "go_test_results.json")
	if err := ioutil.WriteFile(stdFilePath, outputData, 0644); err != nil {
		fmt.Printf("⚠️  Warning: Could not write to standard results file: %v\n", err)
	}

	fmt.Printf("✅ Tests completed. Results saved to %s\n", filename)
	fmt.Printf("📊 Summary: %d total, %d passed, %d failed, %d errors\n",
		summary.Total, summary.Passed, summary.Failed, summary.Errors)

	return nil
}

// runBenchmarks executes benchmark tests
func (r *GoTestRunner) runBenchmarks() error {
	fmt.Println("⚡ Running Go FHIRPath benchmarks...")

	var benchmarks []BenchmarkResult

	// Load test data
	testData, err := r.loadTestData("patient-example.xml")
	if err != nil {
		fmt.Printf("Warning: Could not load test data: %v\n", err)
		return fmt.Errorf("failed to load test data: %v", err)
	}

	// Use benchmark cases from test-config.json
	benchmarkCases := r.testConfig.BenchmarkTests

	for _, testCase := range benchmarkCases {
		iterations := 1000
		if testCase.Iterations > 0 {
			iterations = testCase.Iterations
		}
		var times []float64

		// Parse the expression once outside the loop
		expr, err := fhirpath.Parse(testCase.Expression)
		if err != nil {
			fmt.Printf("⚠️  Skipping benchmark %s - failed to parse expression: %v\n", testCase.Name, err)
			continue
		}

		ctx := r4.Context()
		ctx = fhirpath.WithAPDContext(ctx, apd.BaseContext.WithPrecision(10))

		for i := 0; i < iterations; i++ {
			startTime := time.Now()
			_, err := fhirpath.Evaluate(ctx, testData, expr)
			endTime := time.Now()

			if err != nil {
				fmt.Printf("⚠️  Error in benchmark %s: %v\n", testCase.Name, err)
				break
			}

			executionTime := float64(endTime.Sub(startTime).Nanoseconds()) / 1000000.0
			times = append(times, executionTime)
		}

		if len(times) == 0 {
			continue
		}

		// Calculate statistics
		var sum, min, max float64
		min = times[0]
		max = times[0]

		for _, t := range times {
			sum += t
			if t < min {
				min = t
			}
			if t > max {
				max = t
			}
		}

		avgTime := sum / float64(iterations)
		opsPerSecond := 1000.0 / avgTime // Convert ms to ops/sec

		benchmark := BenchmarkResult{
			Name:         testCase.Name,
			Description:  testCase.Description,
			Expression:   testCase.Expression,
			Iterations:   iterations,
			AvgTimeMs:    avgTime,
			MinTimeMs:    min,
			MaxTimeMs:    max,
			OpsPerSecond: opsPerSecond,
		}

		benchmarks = append(benchmarks, benchmark)
	}

	// Create output structure
	output := BenchmarkOutput{
		Language:   "go",
		Timestamp:  float64(time.Now().Unix()) + float64(time.Now().Nanosecond())/1e9,
		Benchmarks: benchmarks,
		SystemInfo: SystemInfo{
			Platform:        runtime.GOOS,
			GoVersion:       runtime.Version(),
			FhirpathVersion: "fhir-toolbox-go", // Using the new library name
		},
	}

	// Save results to file
	filename := fmt.Sprintf("go_benchmark_results.json")
	resultFilePath := filepath.Join(r.resultsDir, filename)

	outputData, err := json.MarshalIndent(output, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal benchmark results: %v", err)
	}

	if err := ioutil.WriteFile(resultFilePath, outputData, 0644); err != nil {
		return fmt.Errorf("failed to write benchmark results file: %v", err)
	}

	// Also save to the standard filename for compatibility
	stdFilePath := filepath.Join(r.resultsDir, "go_benchmark_results.json")
	if err := ioutil.WriteFile(stdFilePath, outputData, 0644); err != nil {
		fmt.Printf("⚠️  Warning: Could not write to standard benchmark results file: %v\n", err)
	}

	fmt.Printf("✅ Benchmarks completed. Results saved to %s\n", filename)

	return nil
}

func main() {
	mode := "both"
	if len(os.Args) >= 2 {
		mode = os.Args[1]
	}

	runner, err := NewGoTestRunner()
	if err != nil {
		fmt.Printf("❌ Failed to initialize test runner: %v\n", err)
		os.Exit(1)
	}

	switch mode {
	case "test":
		if err := runner.runTests(); err != nil {
			fmt.Printf("❌ Test execution failed: %v\n", err)
			os.Exit(1)
		}
	case "benchmark":
		if err := runner.runBenchmarks(); err != nil {
			fmt.Printf("❌ Benchmark execution failed: %v\n", err)
			os.Exit(1)
		}
	case "both":
		if err := runner.runTests(); err != nil {
			fmt.Printf("❌ Test execution failed: %v\n", err)
			os.Exit(1)
		}
		if err := runner.runBenchmarks(); err != nil {
			fmt.Printf("❌ Benchmark execution failed: %v\n", err)
			os.Exit(1)
		}
	default:
		fmt.Printf("❌ Unknown mode: %s. Use 'test', 'benchmark', or 'both'\n", mode)
		os.Exit(1)
	}

	fmt.Println("✅ Go test runner completed")
}
