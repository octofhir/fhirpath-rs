/**
 * Result Loader Module for Docs Site
 *
 * This module handles loading result files from the public directory.
 */

// Import test and benchmark result data
// These will be copied to the public directory during build

// Function to generate a test summary
function generateTestSummary(tests) {
  const summary = {
    total: tests.length,
    passed: 0,
    failed: 0,
    errors: 0
  };

  tests.forEach(test => {
    switch (test.status) {
      case 'passed':
        summary.passed++;
        break;
      case 'failed':
        summary.failed++;
        break;
      case 'error':
        summary.errors++;
        break;
      default:
        // Handle any other status as error
        summary.errors++;
    }
  });

  return summary;
}

/**
 * Get the comparison report
 * @returns {Promise<Object>} The comparison report data
 */
export async function getComparisonReport() {
  try {
    const response = await fetch(`${window.PUBLIC_BASE_URL}comparison_report.json`);
    if (!response.ok) {
      throw new Error(`Failed to fetch comparison report: ${response.status}`);
    }
    return await response.json();
  } catch (error) {
    console.error('Error loading comparison report:', error);
    return { comparison_report: { test_results: [], benchmark_results: [] } };
  }
}

/**
 * Get test results for all languages
 * @returns {Promise<Array>} Promise resolving to array of test results for all available languages
 */
export async function getAllTestResults() {
  const results = [];
  const languages = ['javascript', 'python', 'java', 'csharp', 'rust', 'go', 'clojure'];

  // Get comparison report
  const report = await getComparisonReport();

  // Add test results from comparison report if available
  if (report && report.comparison_report && report.comparison_report.test_results) {
    results.push(...report.comparison_report.test_results);
  }

  // Try to load any missing test results directly
  const loadTestResults = async (language) => {
    try {
      // Check if we already have this language's results
      if (results.some(r => r.language === language)) {
        console.log(`${language} test results already included from comparison report`);
        return;
      }

      // Try to fetch the test results
      const response = await fetch(`${window.PUBLIC_BASE_URL}${language}_test_results.json`);
      if (!response.ok) {
        console.log(`No test results available for ${language}`);
        return;
      }

      const result = await response.json();

      // Ensure the language field is set correctly
      if (!result.language) {
        result.language = language;
      }

      // Generate summary for the test data if it doesn't exist
      if (result.tests && !result.summary) {
        result.summary = generateTestSummary(result.tests);
      }

      results.push(result);
      console.log(`Added ${language} test results to final results array`);
    } catch (error) {
      console.log(`Failed to load ${language} test results:`, error);
    }
  };

  // Load test results for all languages
  await Promise.all(languages.map(lang => loadTestResults(lang)));

  return results;
}

/**
 * Get benchmark results for all languages
 * @returns {Promise<Array>} Promise resolving to array of benchmark results for all available languages
 */
export async function getAllBenchmarkResults() {
  const results = [];
  const languages = ['javascript', 'python', 'java', 'csharp', 'rust', 'go', 'clojure'];

  // Get comparison report
  const report = await getComparisonReport();

  // Add benchmark results from comparison report if available
  if (report && report.comparison_report && report.comparison_report.benchmark_results) {
    results.push(...report.comparison_report.benchmark_results);
  }

  // Try to load any missing benchmark results directly
  const loadBenchmarkResults = async (language) => {
    try {
      // Check if we already have this language's results
      if (results.some(r => r.language === language)) {
        console.log(`${language} benchmark results already included from comparison report`);
        return;
      }

      // Try to fetch the benchmark results
      const response = await fetch(`${window.PUBLIC_BASE_URL}${language}_benchmark_results.json`);
      if (!response.ok) {
        console.log(`No benchmark results available for ${language}`);
        return;
      }

      const result = await response.json();

      // Ensure the language field is set correctly
      if (!result.language) {
        result.language = language;
      }

      results.push(result);
      console.log(`Added ${language} benchmark results to final results array`);
    } catch (error) {
      console.log(`Failed to load ${language} benchmark results:`, error);
    }
  };

  // Load benchmark results for all languages
  await Promise.all(languages.map(lang => loadBenchmarkResults(lang)));

  return results;
}

// Create sample data for testing when no real data is available
export function createSampleData() {
  const sampleTestResult = {
    language: 'javascript',
    timestamp: Date.now() / 1000,
    tests: [
      {
        name: 'testExtractBirthDate',
        description: 'Extract birthDate',
        expression: 'birthDate',
        status: 'passed',
        execution_time_ms: 14.23,
        expected: [{type: 'date', value: '1974-12-25'}],
        actual: []
      },
      {
        name: 'testSimple',
        description: 'Simple path navigation',
        expression: 'name.given',
        status: 'passed',
        execution_time_ms: 2.27,
        expected: [
          {type: 'string', value: 'Peter'},
          {type: 'string', value: 'James'}
        ],
        actual: []
      }
    ],
    summary: {total: 2, passed: 2, failed: 0, errors: 0}
  };

  const sampleBenchmarkResult = {
    language: 'javascript',
    timestamp: Date.now() / 1000,
    benchmarks: [
      {
        name: 'complexPathNavigation',
        description: 'Complex path navigation performance test',
        expression: "Patient.name.where(use = 'official').given.first()",
        iterations: 1000,
        avg_time_ms: 0.29,
        min_time_ms: 0.12,
        max_time_ms: 7.23,
        ops_per_second: 3415.72
      }
    ],
    system_info: {
      platform: 'darwin',
      node_version: 'v22.16.0',
      fhirpath_version: '3.18.0'
    }
  };

  return {
    testResults: [sampleTestResult],
    benchmarkResults: [sampleBenchmarkResult]
  };
}
