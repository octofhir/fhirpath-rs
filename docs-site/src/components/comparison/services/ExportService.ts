/**
 * Export Service for FHIRPath Comparison Data
 * Provides functionality to export comparison results as PDF reports and CSV data
 */

import type { BenchmarkResultSet, TestResultSet, PerformanceMetrics } from '../types/comparison';
import { LANGUAGE_INFO } from './resultLoader';

export interface ExportOptions {
  includeCharts?: boolean;
  includeDetailedResults?: boolean;
  format: 'pdf' | 'csv' | 'json';
  selectedLanguages?: string[];
}

export interface ExportData {
  metadata: {
    exportDate: string;
    totalLanguages: number;
    totalTests: number;
    totalBenchmarks: number;
  };
  summary: PerformanceMetrics;
  languageResults: Array<{
    language: string;
    fullName: string;
    testResults: {
      total: number;
      passed: number;
      failed: number;
      errors: number;
      successRate: number;
    };
    benchmarkResults: {
      totalBenchmarks: number;
      avgOpsPerSecond: number;
      avgExecutionTime: number;
      bestBenchmark: string;
      worstBenchmark: string;
    };
  }>;
}

export class ExportService {
  private benchmarkResults: BenchmarkResultSet[];
  private testResults: TestResultSet[];
  private metrics: PerformanceMetrics;

  constructor(
    benchmarkResults: BenchmarkResultSet[],
    testResults: TestResultSet[],
    metrics: PerformanceMetrics
  ) {
    this.benchmarkResults = benchmarkResults;
    this.testResults = testResults;
    this.metrics = metrics;
  }

  /**
   * Export comparison data in the specified format
   */
  async exportData(options: ExportOptions): Promise<void> {
    const exportData = this.prepareExportData(options);

    switch (options.format) {
      case 'csv':
        this.exportAsCSV(exportData, options);
        break;
      case 'json':
        this.exportAsJSON(exportData);
        break;
      case 'pdf':
        await this.exportAsPDF(exportData, options);
        break;
      default:
        throw new Error(`Unsupported export format: ${options.format}`);
    }
  }

  /**
   * Generate shareable URL with current comparison state
   */
  generateShareableURL(selectedLanguages?: string[], activeTab?: string): string {
    const baseUrl = window.location.origin + window.location.pathname;
    const params = new URLSearchParams();

    if (selectedLanguages && selectedLanguages.length > 0) {
      params.set('languages', selectedLanguages.join(','));
    }

    if (activeTab) {
      params.set('tab', activeTab);
    }

    params.set('shared', 'true');
    params.set('timestamp', Date.now().toString());

    return `${baseUrl}?${params.toString()}`;
  }

  /**
   * Prepare data for export
   */
  private prepareExportData(options: ExportOptions): ExportData {
    const selectedLanguages = options.selectedLanguages ||
      this.benchmarkResults.map(r => r.language);

    const filteredBenchmarks = this.benchmarkResults.filter(r =>
      selectedLanguages.includes(r.language)
    );
    const filteredTests = this.testResults.filter(r =>
      selectedLanguages.includes(r.language)
    );

    return {
      metadata: {
        exportDate: new Date().toISOString(),
        totalLanguages: selectedLanguages.length,
        totalTests: filteredTests.reduce((sum, r) => sum + (r.summary?.total || 0), 0),
        totalBenchmarks: filteredBenchmarks.reduce((sum, r) => sum + r.benchmarks.length, 0)
      },
      summary: this.metrics,
      languageResults: selectedLanguages.map(language => {
        const langInfo = LANGUAGE_INFO[language];
        const testResult = filteredTests.find(r => r.language === language);
        const benchmarkResult = filteredBenchmarks.find(r => r.language === language);

        const testSummary = testResult?.summary || { total: 0, passed: 0, failed: 0, errors: 0 };
        const successRate = testSummary.total > 0 ?
          (testSummary.passed / testSummary.total) * 100 : 0;

        let benchmarkSummary = {
          totalBenchmarks: 0,
          avgOpsPerSecond: 0,
          avgExecutionTime: 0,
          bestBenchmark: 'N/A',
          worstBenchmark: 'N/A'
        };

        if (benchmarkResult && benchmarkResult.benchmarks.length > 0) {
          const benchmarks = benchmarkResult.benchmarks;
          const avgOps = benchmarks.reduce((sum, b) => sum + (b.ops_per_second || 0), 0) / benchmarks.length;
          const avgTime = benchmarks.reduce((sum, b) => sum + (b.avg_time_ms || 0), 0) / benchmarks.length;

          const bestBench = benchmarks.reduce((best, current) =>
            (current.ops_per_second || 0) > (best.ops_per_second || 0) ? current : best
          );
          const worstBench = benchmarks.reduce((worst, current) =>
            (current.ops_per_second || 0) < (worst.ops_per_second || 0) ? current : worst
          );

          benchmarkSummary = {
            totalBenchmarks: benchmarks.length,
            avgOpsPerSecond: Number(avgOps.toFixed(0)),
            avgExecutionTime: Number(avgTime.toFixed(2)),
            bestBenchmark: bestBench.name,
            worstBenchmark: worstBench.name
          };
        }

        return {
          language,
          fullName: langInfo?.fullName || language,
          testResults: {
            ...testSummary,
            successRate: Number(successRate.toFixed(1))
          },
          benchmarkResults: benchmarkSummary
        };
      })
    };
  }

  /**
   * Export data as CSV
   */
  private exportAsCSV(data: ExportData, options: ExportOptions): void {
    const csvRows: string[] = [];

    // Header
    csvRows.push([
      'Language',
      'Full Name',
      'Total Tests',
      'Tests Passed',
      'Tests Failed',
      'Test Errors',
      'Success Rate (%)',
      'Total Benchmarks',
      'Avg Ops/Second',
      'Avg Execution Time (ms)',
      'Best Benchmark',
      'Worst Benchmark'
    ].join(','));

    // Data rows
    data.languageResults.forEach(result => {
      csvRows.push([
        result.language,
        `"${result.fullName}"`,
        result.testResults.total,
        result.testResults.passed,
        result.testResults.failed,
        result.testResults.errors,
        result.testResults.successRate,
        result.benchmarkResults.totalBenchmarks,
        result.benchmarkResults.avgOpsPerSecond,
        result.benchmarkResults.avgExecutionTime,
        `"${result.benchmarkResults.bestBenchmark}"`,
        `"${result.benchmarkResults.worstBenchmark}"`
      ].join(','));
    });

    const csvContent = csvRows.join('\n');
    this.downloadFile(csvContent, 'fhirpath-comparison-results.csv', 'text/csv');
  }

  /**
   * Export data as JSON
   */
  private exportAsJSON(data: ExportData): void {
    const jsonContent = JSON.stringify(data, null, 2);
    this.downloadFile(jsonContent, 'fhirpath-comparison-results.json', 'application/json');
  }

  /**
   * Export data as PDF (simplified implementation)
   */
  private async exportAsPDF(data: ExportData, options: ExportOptions): Promise<void> {
    // For now, create a simple HTML report that can be printed as PDF
    const htmlContent = this.generateHTMLReport(data, options);

    // Open in new window for printing
    const printWindow = window.open('', '_blank');
    if (printWindow) {
      printWindow.document.write(htmlContent);
      printWindow.document.close();
      printWindow.focus();

      // Trigger print dialog after content loads
      setTimeout(() => {
        printWindow.print();
      }, 500);
    }
  }

  /**
   * Generate HTML report for PDF export
   */
  private generateHTMLReport(data: ExportData, options: ExportOptions): string {
    return `
<!DOCTYPE html>
<html>
<head>
    <title>FHIRPath Implementation Comparison Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { text-align: center; margin-bottom: 30px; }
        .metadata { background: #f5f5f5; padding: 15px; margin-bottom: 20px; }
        .summary { margin-bottom: 30px; }
        .language-section { margin-bottom: 25px; border-bottom: 1px solid #ddd; padding-bottom: 15px; }
        .metrics { display: flex; gap: 20px; margin: 10px 0; }
        .metric { flex: 1; text-align: center; }
        table { width: 100%; border-collapse: collapse; margin: 15px 0; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        @media print { body { margin: 0; } }
    </style>
</head>
<body>
    <div class="header">
        <h1>FHIRPath Implementation Comparison Report</h1>
        <p>Generated on ${new Date(data.metadata.exportDate).toLocaleString()}</p>
    </div>
    
    <div class="metadata">
        <h2>Report Summary</h2>
        <p><strong>Languages Compared:</strong> ${data.metadata.totalLanguages}</p>
        <p><strong>Total Tests:</strong> ${data.metadata.totalTests.toLocaleString()}</p>
        <p><strong>Total Benchmarks:</strong> ${data.metadata.totalBenchmarks.toLocaleString()}</p>
    </div>

    <div class="summary">
        <h2>Overall Metrics</h2>
        <div class="metrics">
            <div class="metric">
                <h3>${data.summary.totalTests.toLocaleString()}</h3>
                <p>Total Tests</p>
            </div>
            <div class="metric">
                <h3>${data.summary.successRate}%</h3>
                <p>Average Success Rate</p>
            </div>
            <div class="metric">
                <h3>${data.summary.avgExecutionTime}ms</h3>
                <p>Average Execution Time</p>
            </div>
            <div class="metric">
                <h3>${data.summary.totalBenchmarks}</h3>
                <p>Total Benchmarks</p>
            </div>
        </div>
    </div>

    <h2>Language Implementation Results</h2>
    ${data.languageResults.map(result => `
        <div class="language-section">
            <h3>${result.fullName} (${result.language})</h3>
            
            <h4>Test Results</h4>
            <table>
                <tr><th>Metric</th><th>Value</th></tr>
                <tr><td>Total Tests</td><td>${result.testResults.total}</td></tr>
                <tr><td>Tests Passed</td><td>${result.testResults.passed}</td></tr>
                <tr><td>Tests Failed</td><td>${result.testResults.failed}</td></tr>
                <tr><td>Test Errors</td><td>${result.testResults.errors}</td></tr>
                <tr><td>Success Rate</td><td>${result.testResults.successRate}%</td></tr>
            </table>

            <h4>Benchmark Results</h4>
            <table>
                <tr><th>Metric</th><th>Value</th></tr>
                <tr><td>Total Benchmarks</td><td>${result.benchmarkResults.totalBenchmarks}</td></tr>
                <tr><td>Average Ops/Second</td><td>${result.benchmarkResults.avgOpsPerSecond.toLocaleString()}</td></tr>
                <tr><td>Average Execution Time</td><td>${result.benchmarkResults.avgExecutionTime}ms</td></tr>
                <tr><td>Best Benchmark</td><td>${result.benchmarkResults.bestBenchmark}</td></tr>
                <tr><td>Worst Benchmark</td><td>${result.benchmarkResults.worstBenchmark}</td></tr>
            </table>
        </div>
    `).join('')}
</body>
</html>`;
  }

  /**
   * Download file helper
   */
  private downloadFile(content: string, filename: string, mimeType: string): void {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);

    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    link.style.display = 'none';

    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);

    URL.revokeObjectURL(url);
  }
}
