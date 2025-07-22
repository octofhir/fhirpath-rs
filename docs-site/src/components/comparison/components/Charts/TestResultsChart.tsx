/**
 * Test Results Chart Component using Mantine Charts
 */

import { Card, Text, Stack, SimpleGrid, Badge, Group } from '@mantine/core';
import { BarChart } from '@mantine/charts';
import type { TestResultSet } from '../../types/comparison';

interface ChartDataPoint {
  language: string;
  passed: number;
  failed: number;
  errors: number;
  total: number;
  successRate: number;
}
import { LANGUAGE_INFO } from '../../services/resultLoader';
import { useComparisonTheme } from '../../providers/ThemeProvider';
import styles from './charts.module.css';

interface TestResultsChartProps {
  testResults: TestResultSet[];
  title?: string;
}


export function TestResultsChart({ testResults, title = 'Test Results by Language Implementation' }: TestResultsChartProps) {
  const { chartColors } = useComparisonTheme();

  // Transform test results data for the chart
  const chartData: ChartDataPoint[] = testResults.map(result => {
    const summary = result.summary || { total: 0, passed: 0, failed: 0, errors: 0 };
    const langInfo = LANGUAGE_INFO[result.language];
    const total = summary.total || (summary.passed + summary.failed + summary.errors);
    const successRate = total > 0 ? (summary.passed / total) * 100 : 0;

    return {
      language: langInfo?.fullName || result.language,
      passed: summary.passed,
      failed: summary.failed,
      errors: summary.errors,
      total: total,
      successRate: Number(successRate.toFixed(1))
    };
  });

  // If no data, show empty state
  if (chartData.length === 0) {
    return (
      <Card className={styles.chartCard} padding="xl" radius="md" withBorder>
        <Stack align="center" gap="md" className={styles.emptyState}>
          <Text size="lg" fw={600} c="dimmed">
            No test results available
          </Text>
          <Text size="sm" c="dimmed" ta="center">
            Test results will appear here once data is loaded
          </Text>
        </Stack>
      </Card>
    );
  }

  return (
    <Card className={styles.chartCard} padding="xl" radius="md" withBorder>
      <Card.Section className={styles.chartHeader}>
        <Text size="lg" fw={600}>
          {title}
        </Text>
        <Text size="sm" c="dimmed">
          Comparison of test results across {testResults.length} implementations
        </Text>
      </Card.Section>

      {/* Test Quality Summary */}
      <div className={styles.performanceSummary}>
        <SimpleGrid cols={{ base: 1, sm: 3 }} spacing="md">
          {chartData
            .sort((a, b) => b.successRate - a.successRate)
            .slice(0, 3)
            .map((data, index) => {
              const qualityColor = data.successRate >= 95 ? 'green' :
                                 data.successRate >= 85 ? 'blue' :
                                 data.successRate >= 70 ? 'yellow' : 'red';
              const qualityLabel = data.successRate >= 95 ? 'Excellent' :
                                 data.successRate >= 85 ? 'Good' :
                                 data.successRate >= 70 ? 'Fair' : 'Poor';

              return (
                <div key={data.language} className={styles.performanceItem}>
                  <Group gap="xs" align="center" mb="xs">
                    <Badge
                      size="sm"
                      variant="light"
                      color={index === 0 ? 'green' : index === 1 ? 'blue' : 'orange'}
                    >
                      #{index + 1}
                    </Badge>
                    <Text size="sm" fw={600}>
                      {data.language}
                    </Text>
                    <Badge
                      size="xs"
                      variant="dot"
                      color={qualityColor}
                    >
                      {qualityLabel}
                    </Badge>
                  </Group>
                  <Stack gap="xs">
                    <Text size="xs" c="dimmed">
                      {data.successRate}% success rate
                    </Text>
                    <Text size="xs" c="dimmed">
                      {data.passed}/{data.total} tests passed
                    </Text>
                    <Text size="xs" c="dimmed">
                      {data.failed} failed, {data.errors} errors
                    </Text>
                  </Stack>
                </div>
              );
            })}
        </SimpleGrid>
      </div>

      <div className={styles.chartContainer}>
        <BarChart
          h={400}
          data={chartData}
          dataKey="language"
          series={[
            {
              name: 'passed',
              color: chartColors.python,
              label: 'Tests Passed'
            },
            {
              name: 'failed',
              color: chartColors.csharp,
              label: 'Tests Failed'
            },
            {
              name: 'errors',
              color: '#ef4444',
              label: 'Tests with Errors'
            }
          ]}
          tickLine="xy"
          gridAxis="xy"
          withYAxisLabel
          xAxisLabel="Implementation"
          yAxisLabel="Number of Tests"
          withTooltip
          tooltipAnimationDuration={200}
          className={styles.chart}
        />
      </div>

      <div className={styles.chartFooter}>
        <Text size="xs" c="dimmed">
          Test results show implementation compliance with FHIRPath specification.
          Green sections show successful tests, orange shows failures, and red shows errors.
          Quality badges indicate reliability: Excellent (≥95%), Good (≥85%), Fair (≥70%), Poor (&lt;70%).
          Higher success rates with fewer errors indicate better specification compliance.
        </Text>
      </div>
    </Card>
  );
}
