/**
 * Competitive Positioning Service
 * Calculates how the Rust FHIRPath library compares to best-in-class implementations
 */

import type { BenchmarkResultSet, TestResultSet } from '../types/comparison';

export interface CompetitiveMetrics {
  performanceGap: number; // percentage difference in performance
  complianceGap: number; // percentage difference in compliance
  reliabilityGap: number; // percentage difference in reliability
  status: 'leading' | 'competitive' | 'behind' | 'significantly-behind';
  bestPerformer: string; // name of the best performing language
  rustMetrics: {
    avgOpsPerSecond: number;
    successRate: number;
    errorRate: number;
  };
  bestMetrics: {
    avgOpsPerSecond: number;
    successRate: number;
    errorRate: number;
  };
}

export interface BenchmarkComparison {
  benchmarkName: string;
  rustPerformance: number;
  bestPerformance: number;
  bestPerformer: string;
  percentageDifference: number;
  status: 'leading' | 'competitive' | 'behind' | 'significantly-behind';
}

export class CompetitivePositioning {
  private benchmarkResults: BenchmarkResultSet[];
  private testResults: TestResultSet[];

  constructor(benchmarkResults: BenchmarkResultSet[], testResults: TestResultSet[]) {
    this.benchmarkResults = benchmarkResults;
    this.testResults = testResults;
  }

  /**
   * Calculate overall competitive metrics for Rust vs best performers
   */
  calculateRustVsBest(): CompetitiveMetrics {
    const rustBenchmarks = this.benchmarkResults.find(r => r.language === 'rust');
    const rustTests = this.testResults.find(r => r.language === 'rust');

    if (!rustBenchmarks || !rustTests) {
      throw new Error('Rust benchmark or test results not found');
    }

    // Calculate Rust metrics
    const rustAvgOps = this.calculateAverageOpsPerSecond(rustBenchmarks);
    const rustSuccessRate = this.calculateSuccessRate(rustTests);
    const rustErrorRate = this.calculateErrorRate(rustTests);

    // Find best performers
    const bestPerformanceBenchmarks = this.findBestPerformer('performance');
    const bestComplianceTests = this.findBestPerformer('compliance');

    const bestAvgOps = this.calculateAverageOpsPerSecond(bestPerformanceBenchmarks);
    const bestSuccessRate = this.calculateSuccessRate(bestComplianceTests);
    const bestErrorRate = this.calculateErrorRate(bestComplianceTests);

    // Calculate percentage differences
    const performanceGap = this.calculatePercentageDifference(rustAvgOps, bestAvgOps);
    const complianceGap = this.calculatePercentageDifference(rustSuccessRate, bestSuccessRate);
    const reliabilityGap = this.calculatePercentageDifference(rustErrorRate, bestErrorRate, true); // lower is better for errors

    // Determine overall competitive status
    const status = this.determineCompetitiveStatus([performanceGap, complianceGap, reliabilityGap]);

    return {
      performanceGap,
      complianceGap,
      reliabilityGap,
      status,
      bestPerformer: bestPerformanceBenchmarks.language,
      rustMetrics: {
        avgOpsPerSecond: rustAvgOps,
        successRate: rustSuccessRate,
        errorRate: rustErrorRate
      },
      bestMetrics: {
        avgOpsPerSecond: bestAvgOps,
        successRate: bestSuccessRate,
        errorRate: bestErrorRate
      }
    };
  }

  /**
   * Get detailed benchmark comparisons for each test
   */
  getBenchmarkComparisons(): BenchmarkComparison[] {
    const rustBenchmarks = this.benchmarkResults.find(r => r.language === 'rust');
    if (!rustBenchmarks) return [];

    const comparisons: BenchmarkComparison[] = [];

    rustBenchmarks.benchmarks.forEach(rustBench => {
      const bestForThisBenchmark = this.findBestPerformerForBenchmark(rustBench.name);
      if (!bestForThisBenchmark) return;

      const rustOps = rustBench.ops_per_second || 0;
      const bestOps = bestForThisBenchmark.ops_per_second || 0;
      const percentageDifference = this.calculatePercentageDifference(rustOps, bestOps);

      comparisons.push({
        benchmarkName: rustBench.name,
        rustPerformance: rustOps,
        bestPerformance: bestOps,
        bestPerformer: bestForThisBenchmark.language,
        percentageDifference,
        status: this.determineCompetitiveStatus([percentageDifference])
      });
    });

    return comparisons;
  }

  /**
   * Format percentage difference for display
   */
  formatPercentageDifference(value: number): string {
    const sign = value >= 0 ? '+' : '';
    return `${sign}${value.toFixed(1)}%`;
  }

  /**
   * Get color for competitive status
   */
  getCompetitiveStatusColor(status: string): string {
    switch (status) {
      case 'leading': return 'green';
      case 'competitive': return 'yellow';
      case 'behind': return 'orange';
      case 'significantly-behind': return 'red';
      default: return 'gray';
    }
  }

  /**
   * Get icon for competitive status
   */
  getCompetitiveStatusIcon(status: string): string {
    switch (status) {
      case 'leading': return '🏆';
      case 'competitive': return '🎯';
      case 'behind': return '⚠️';
      case 'significantly-behind': return '🚨';
      default: return '📊';
    }
  }

  // Private helper methods

  private findBestPerformer(metric: 'performance' | 'compliance'): BenchmarkResultSet | TestResultSet {
    if (metric === 'performance') {
      return this.benchmarkResults.reduce((best, current) => {
        const bestAvg = this.calculateAverageOpsPerSecond(best);
        const currentAvg = this.calculateAverageOpsPerSecond(current);
        return currentAvg > bestAvg ? current : best;
      });
    } else {
      return this.testResults.reduce((best, current) => {
        const bestRate = this.calculateSuccessRate(best);
        const currentRate = this.calculateSuccessRate(current);
        return currentRate > bestRate ? current : best;
      });
    }
  }

  private findBestPerformerForBenchmark(benchmarkName: string): { ops_per_second: number; language: string } | null {
    let bestPerformance = 0;
    let bestLanguage = '';

    this.benchmarkResults.forEach(result => {
      const benchmark = result.benchmarks.find(b => b.name === benchmarkName);
      if (benchmark && benchmark.ops_per_second > bestPerformance) {
        bestPerformance = benchmark.ops_per_second;
        bestLanguage = result.language;
      }
    });

    return bestPerformance > 0 ? { ops_per_second: bestPerformance, language: bestLanguage } : null;
  }

  private calculateAverageOpsPerSecond(benchmarkResult: BenchmarkResultSet): number {
    if (!benchmarkResult.benchmarks || benchmarkResult.benchmarks.length === 0) return 0;

    const total = benchmarkResult.benchmarks.reduce((sum, bench) => sum + (bench.ops_per_second || 0), 0);
    return total / benchmarkResult.benchmarks.length;
  }

  private calculateSuccessRate(testResult: TestResultSet): number {
    if (!testResult.summary) return 0;
    const { total, passed } = testResult.summary;
    return total > 0 ? (passed / total) * 100 : 0;
  }

  private calculateErrorRate(testResult: TestResultSet): number {
    if (!testResult.summary) return 0;
    const { total, errors } = testResult.summary;
    return total > 0 ? (errors / total) * 100 : 0;
  }

  private calculatePercentageDifference(current: number, best: number, lowerIsBetter: boolean = false): number {
    if (best === 0) return 0;

    const difference = ((current - best) / best) * 100;
    return lowerIsBetter ? -difference : difference; // Invert for metrics where lower is better
  }

  private determineCompetitiveStatus(gaps: number[]): 'leading' | 'competitive' | 'behind' | 'significantly-behind' {
    const avgGap = gaps.reduce((sum, gap) => sum + Math.abs(gap), 0) / gaps.length;

    if (gaps.some(gap => gap > 0)) return 'leading'; // Leading in at least one area
    if (avgGap <= 20) return 'competitive'; // Within 20% on average
    if (avgGap <= 50) return 'behind'; // 20-50% behind
    return 'significantly-behind'; // More than 50% behind
  }
}
