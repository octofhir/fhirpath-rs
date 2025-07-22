/**
 * Result Loader Service for FHIRPath Comparison
 * TypeScript service for loading test and benchmark results
 */

import type {
  TestResult,
  TestSummary,
  TestResultSet,
  BenchmarkResultSet,
  ComparisonReport,
  LanguageInfo
} from '../types/comparison';

// Language metadata
export const LANGUAGE_INFO: Record<string, LanguageInfo> = {
  javascript: {
    fullName: 'JavaScript',
    description: 'Node.js implementation using the fhirpath.js library',
    repository: 'https://github.com/HL7/fhirpath.js',
    license: 'BSD-3-Clause',
    maintainer: 'HL7 FHIR Community',
    icon: '🟨',
    libraries: ['fhirpath.js', 'antlr4-javascript-runtime'],
    runtime: 'Node.js'
  },
  python: {
    fullName: 'Python',
    description: 'Python implementation using the fhirpath_py library',
    repository: 'https://github.com/beda-software/fhirpath-py',
    license: 'MIT',
    maintainer: 'Beda Software',
    icon: '🐍',
    libraries: ['fhirpath-py', 'antlr4-python3-runtime'],
    runtime: 'Python 3'
  },
  java: {
    fullName: 'Java',
    description: 'Java implementation using the HAPI FHIR library',
    repository: 'https://github.com/hapifhir/org.hl7.fhir.core',
    license: 'Apache-2.0',
    maintainer: 'HAPI FHIR',
    icon: '☕',
    libraries: ['org.hl7.fhir.core', 'ANTLR4'],
    runtime: 'JVM'
  },
  csharp: {
    fullName: 'C#',
    description: 'C# implementation using the Firely SDK',
    repository: 'https://github.com/FirelyTeam/firely-net-sdk',
    license: 'BSD-3-Clause',
    maintainer: 'Firely',
    icon: '🔷',
    libraries: ['Hl7.Fhir.Core', 'Hl7.FhirPath'],
    runtime: '.NET'
  },
  rust: {
    fullName: 'Rust',
    description: 'Rust implementation using OctoFHIR FHIRPath',
    repository: 'https://github.com/octofhir/fhirpath-rs',
    license: 'Apache-2.0',
    maintainer: 'OctoFHIR Team',
    icon: '🦀',
    libraries: ['fhirpath-core', 'serde_json'],
    runtime: 'Native'
  },
  go: {
    fullName: 'Go',
    description: 'Go implementation using fhir-toolbox-go library',
    repository: 'https://github.com/DAMEDIC/fhir-toolbox-go',
    license: 'Apache-2.0',
    maintainer: 'DAMEDIC',
    icon: '🔵',
    libraries: ['fhir-toolbox-go'],
    runtime: 'Native'
  },
  clojure: {
    fullName: 'Clojure',
    description: 'Clojure implementation using fhirpath.clj library',
    repository: 'https://github.com/HealthSamurai/fhirpath.clj',
    license: 'MIT',
    maintainer: 'HealthSamurai',
    icon: '🟢',
    libraries: ['fhirpath.clj', 'ANTLR4'],
    runtime: 'JVM'
  }
};

// Supported languages
const SUPPORTED_LANGUAGES = Object.keys(LANGUAGE_INFO);

/**
 * Generate a test summary from test results
 */
function generateTestSummary(tests: TestResult[]): TestSummary {
  const summary: TestSummary = {
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
 * Get the base URL for fetching resources
 */
function getBaseUrl(): string {
  if (typeof window !== 'undefined' && (window as any).PUBLIC_BASE_URL) {
    return (window as any).PUBLIC_BASE_URL;
  }
  return '/';
}

/**
 * Fetch JSON data with error handling and timeout
 */
async function fetchJson<T>(url: string, timeoutMs: number = 10000): Promise<T | null> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const response = await fetch(url, {
      signal: controller.signal
    });

    clearTimeout(timeoutId);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
    return await response.json();
  } catch (error) {
    clearTimeout(timeoutId);

    if (error instanceof Error && error.name === 'AbortError') {
      console.warn(`Request to ${url} timed out after ${timeoutMs}ms`);
    } else {
      console.warn(`Failed to fetch ${url}:`, error);
    }
    return null;
  }
}

/**
 * Get the comparison report
 */
export async function getComparisonReport(): Promise<ComparisonReport> {
  const baseUrl = getBaseUrl();
  const url = `${baseUrl}comparison_report.json`;

  const report = await fetchJson<{ comparison_report: ComparisonReport }>(url);

  if (report?.comparison_report) {
    return report.comparison_report;
  }

  return {
    test_results: [],
    benchmark_results: []
  };
}

/**
 * Load test results for a specific language
 */
async function loadLanguageTestResults(language: string): Promise<TestResultSet | null> {
  const baseUrl = getBaseUrl();
  const url = `${baseUrl}${language}_test_results.json`;

  const result = await fetchJson<TestResultSet>(url);

  if (!result) {
    return null;
  }

  // Ensure the language field is set correctly
  if (!result.language) {
    result.language = language;
  }

  // Generate summary for the test data if it doesn't exist
  if (result.tests && !result.summary) {
    result.summary = generateTestSummary(result.tests);
  }

  return result;
}

/**
 * Load benchmark results for a specific language
 */
async function loadLanguageBenchmarkResults(language: string): Promise<BenchmarkResultSet | null> {
  const baseUrl = getBaseUrl();
  const url = `${baseUrl}${language}_benchmark_results.json`;

  const result = await fetchJson<BenchmarkResultSet>(url);

  if (!result) {
    return null;
  }

  // Ensure the language field is set correctly
  if (!result.language) {
    result.language = language;
  }

  return result;
}

/**
 * Get test results for all languages
 */
export async function getAllTestResults(): Promise<TestResultSet[]> {
  const results: TestResultSet[] = [];

  // Get comparison report first
  const report = await getComparisonReport();

  // Add test results from comparison report if available
  if (report.test_results) {
    results.push(...report.test_results);
  }

  // Load any missing test results directly
  const loadPromises = SUPPORTED_LANGUAGES.map(async (language) => {
    // Check if we already have this language's results
    if (results.some(r => r.language === language)) {
      console.log(`${language} test results already included from comparison report`);
      return;
    }

    const result = await loadLanguageTestResults(language);
    if (result) {
      results.push(result);
      console.log(`Added ${language} test results to final results array`);
    }
  });

  await Promise.all(loadPromises);

  return results;
}

/**
 * Get benchmark results for all languages
 */
export async function getAllBenchmarkResults(): Promise<BenchmarkResultSet[]> {
  const results: BenchmarkResultSet[] = [];

  // Get comparison report first
  const report = await getComparisonReport();

  // Add benchmark results from comparison report if available
  if (report.benchmark_results) {
    results.push(...report.benchmark_results);
  }

  // Load any missing benchmark results directly
  const loadPromises = SUPPORTED_LANGUAGES.map(async (language) => {
    // Check if we already have this language's results
    if (results.some(r => r.language === language)) {
      console.log(`${language} benchmark results already included from comparison report`);
      return;
    }

    const result = await loadLanguageBenchmarkResults(language);
    if (result) {
      results.push(result);
      console.log(`Added ${language} benchmark results to final results array`);
    }
  });

  await Promise.all(loadPromises);

  return results;
}

/**
 * Create sample data for demonstration purposes
 */
export function createSampleData(): {
  testResults: TestResultSet[];
  benchmarkResults: BenchmarkResultSet[];
} {
  const sampleTestResults: TestResultSet[] = [
    {
      language: 'rust',
      summary: { total: 100, passed: 98, failed: 2, errors: 0 },
      tests: []
    },
    {
      language: 'javascript',
      summary: { total: 100, passed: 95, failed: 4, errors: 1 },
      tests: []
    },
    {
      language: 'python',
      summary: { total: 100, passed: 92, failed: 6, errors: 2 },
      tests: []
    }
  ];

  const sampleBenchmarkResults: BenchmarkResultSet[] = [
    {
      language: 'rust',
      benchmarks: [
        { name: 'simple_path', avg_time_ms: 0.05, ops_per_second: 20000 },
        { name: 'complex_path', avg_time_ms: 0.15, ops_per_second: 6666 }
      ]
    },
    {
      language: 'javascript',
      benchmarks: [
        { name: 'simple_path', avg_time_ms: 0.8, ops_per_second: 1250 },
        { name: 'complex_path', avg_time_ms: 2.1, ops_per_second: 476 }
      ]
    },
    {
      language: 'python',
      benchmarks: [
        { name: 'simple_path', avg_time_ms: 1.2, ops_per_second: 833 },
        { name: 'complex_path', avg_time_ms: 3.5, ops_per_second: 286 }
      ]
    }
  ];

  return {
    testResults: sampleTestResults,
    benchmarkResults: sampleBenchmarkResults
  };
}

/**
 * Enhanced language info with version information
 */
export function getEnhancedLanguageInfo(
  testResults: TestResultSet[],
  benchmarkResults: BenchmarkResultSet[]
): Record<string, LanguageInfo> {
  const enhancedInfo = { ...LANGUAGE_INFO };

  // Extract version information from benchmark results
  benchmarkResults.forEach(result => {
    if (result.system_info && result.language) {
      const lang = enhancedInfo[result.language];
      if (lang) {
        lang.version = result.system_info.fhirpath_version || 'Unknown';
        lang.platform = result.system_info.platform || 'Unknown';
        lang.runtime_version = result.system_info.node_version ||
                              result.system_info.python_version ||
                              result.system_info.java_version ||
                              result.system_info.dotnet_version ||
                              result.system_info.rust_version ||
                              result.system_info.go_version ||
                              result.system_info.clojure_version || 'Unknown';
      }
    }
  });

  return enhancedInfo;
}
