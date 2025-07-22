/**
 * Custom hook for calculating performance metrics from test and benchmark data
 */

import { useMemo } from 'react';
import type {
  TestResultSet,
  BenchmarkResultSet,
  PerformanceMetrics
} from '../types/comparison';

export function usePerformanceMetrics(
  testResults: TestResultSet[],
  benchmarkResults: BenchmarkResultSet[]
): PerformanceMetrics {
  return useMemo(() => {
    // Calculate total tests and passed tests
    const totalTests = testResults.reduce((sum, result) => sum + (result.summary?.total || 0), 0);
    const totalPassed = testResults.reduce((sum, result) => sum + (result.summary?.passed || 0), 0);
    const successRate = totalTests > 0 ? ((totalPassed / totalTests) * 100) : 0;

    // Calculate average execution time
    const avgExecutionTime = calculateAverageExecutionTime(testResults);

    // Calculate total benchmarks
    const totalBenchmarks = benchmarkResults.reduce((sum, result) => sum + (result.benchmarks?.length || 0), 0);

    // Find fastest implementation
    const fastestImplementation = getFastestImplementation(benchmarkResults);

    // Find most compliant implementation
    const mostCompliantImplementation = getMostCompliantImplementation(testResults);

    return {
      totalTests,
      totalPassed,
      successRate: Number(successRate.toFixed(1)),
      avgExecutionTime: Number(avgExecutionTime.toFixed(2)),
      totalBenchmarks,
      fastestImplementation,
      mostCompliantImplementation
    };
  }, [testResults, benchmarkResults]);
}

/**
 * Calculate average execution time across all tests
 */
function calculateAverageExecutionTime(testResults: TestResultSet[]): number {
  let totalTime = 0;
  let testCount = 0;

  testResults.forEach(result => {
    if (result.tests) {
      result.tests.forEach(test => {
        if (test.execution_time_ms !== undefined) {
          totalTime += test.execution_time_ms;
          testCount++;
        }
      });
    }
  });

  return testCount > 0 ? totalTime / testCount : 0;
}

/**
 * Find the fastest implementation based on average operations per second
 */
function getFastestImplementation(benchmarkResults: BenchmarkResultSet[]): {
  language: string;
  avgOps: number;
} | null {
  if (benchmarkResults.length === 0) return null;

  const langPerformance = benchmarkResults.map(result => {
    if (!result.benchmarks || result.benchmarks.length === 0) {
      return { language: result.language, avgOps: 0 };
    }

    const avgOps = result.benchmarks.reduce((sum, b) => sum + (b.ops_per_second || 0), 0) / result.benchmarks.length;
    return { language: result.language, avgOps };
  });

  return langPerformance.reduce((fastest, current) =>
    current.avgOps > fastest.avgOps ? current : fastest, { language: '', avgOps: 0 });
}

/**
 * Find the most compliant implementation based on test pass rate
 */
function getMostCompliantImplementation(testResults: TestResultSet[]): {
  language: string;
  passRate: number;
} | null {
  if (testResults.length === 0) return null;

  const langCompliance = testResults.map(result => {
    const passRate = result.summary ? (result.summary.passed / result.summary.total) * 100 : 0;
    return { language: result.language, passRate };
  });

  return langCompliance.reduce((mostCompliant, current) =>
    current.passRate > mostCompliant.passRate ? current : mostCompliant, { language: '', passRate: 0 });
}
