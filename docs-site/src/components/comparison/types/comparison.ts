/**
 * TypeScript type definitions for the FHIRPath comparison functionality
 */

export interface TestResult {
  status: 'passed' | 'failed' | 'error';
  execution_time_ms?: number;
  test_name?: string;
  expression?: string;
  expected?: any;
  actual?: any;
  error?: string;
}

export interface TestSummary {
  total: number;
  passed: number;
  failed: number;
  errors: number;
}

export interface SystemInfo {
  fhirpath_version?: string;
  platform?: string;
  node_version?: string;
  python_version?: string;
  java_version?: string;
  dotnet_version?: string;
  rust_version?: string;
  go_version?: string;
  clojure_version?: string;
}

export interface TestResultSet {
  language: string;
  tests?: TestResult[];
  summary?: TestSummary;
  system_info?: SystemInfo;
  timestamp?: string;
}

export interface BenchmarkResult {
  name: string;
  expression?: string;
  avg_time_ms: number;
  ops_per_second: number;
  min_time_ms?: number;
  max_time_ms?: number;
  std_dev?: number;
}

export interface BenchmarkResultSet {
  language: string;
  benchmarks: BenchmarkResult[];
  system_info?: SystemInfo;
  timestamp?: string;
}

export interface ComparisonReport {
  test_results: TestResultSet[];
  benchmark_results: BenchmarkResultSet[];
  generated_at?: string;
}

export interface LanguageInfo {
  fullName: string;
  description: string;
  repository: string;
  license: string;
  maintainer: string;
  icon: string;
  libraries: string[];
  runtime: string;
  version?: string;
  platform?: string;
  runtime_version?: string;
}

export interface PerformanceMetrics {
  totalTests: number;
  totalPassed: number;
  successRate: number;
  avgExecutionTime: number;
  totalBenchmarks: number;
  fastestImplementation: {
    language: string;
    avgOps: number;
  } | null;
  mostCompliantImplementation: {
    language: string;
    passRate: number;
  } | null;
}

export interface ComparisonData {
  testResults: TestResultSet[];
  benchmarkResults: BenchmarkResultSet[];
  languageInfo: Record<string, LanguageInfo>;
  metrics: PerformanceMetrics;
  loading: boolean;
  error: string | null;
}

export type TabType = 'overview' | 'implementations' | 'details' | 'analysis';

export interface ComparisonPageProps {
  initialTab?: TabType;
}

export interface ChartDataPoint {
  language: string;
  passed: number;
  failed: number;
  errors: number;
  avgTime?: number;
  avgOps?: number;
}
