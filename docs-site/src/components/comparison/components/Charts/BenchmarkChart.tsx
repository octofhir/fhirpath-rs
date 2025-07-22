/**
 * Benchmark Chart Component using Mantine Charts
 */

import { Card, Text, Stack, SimpleGrid, Badge, Group, Tabs, NumberFormatter } from '@mantine/core';
import { IconChartBar, IconClock, IconTrendingUp } from '@tabler/icons-react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, ErrorBar } from 'recharts';
import type { BenchmarkResultSet } from '../../types/comparison';
import { LANGUAGE_INFO } from '../../services/resultLoader';
import { useComparisonTheme } from '../../providers/ThemeProvider';
import styles from './charts.module.css';
import { useState } from 'react';

interface BenchmarkChartProps {
  benchmarkResults: BenchmarkResultSet[];
  title?: string;
}

interface BenchmarkDataPoint {
  language: string;
  avgTime: number;
  avgOps: number;
  benchmarkCount: number;
  minTime: number;
  maxTime: number;
  errorLow: number;
  errorHigh: number;
  varianceCoefficient: number;
}

export function BenchmarkChart({
  benchmarkResults,
  title = 'Benchmark Performance by Language Implementation'
}: BenchmarkChartProps) {
  const { chartColors } = useComparisonTheme();

  // Transform benchmark data for charts
  const chartData: BenchmarkDataPoint[] = benchmarkResults.map(result => {
    const langInfo = LANGUAGE_INFO[result.language];

    if (!result.benchmarks || result.benchmarks.length === 0) {
      return {
        language: langInfo?.fullName || result.language,
        avgTime: 0,
        avgOps: 0,
        benchmarkCount: 0,
        minTime: 0,
        maxTime: 0,
        errorLow: 0,
        errorHigh: 0,
        varianceCoefficient: 0
      };
    }

    const avgTime = result.benchmarks.reduce((sum, b) => sum + (b.avg_time_ms || 0), 0) / result.benchmarks.length;
    const avgOps = result.benchmarks.reduce((sum, b) => sum + (b.ops_per_second || 0), 0) / result.benchmarks.length;
    const minTime = Math.min(...result.benchmarks.map(b => b.min_time_ms || 0));
    const maxTime = Math.max(...result.benchmarks.map(b => b.max_time_ms || 0));

    // Calculate error bar ranges
    const errorLow = avgTime - minTime;
    const errorHigh = maxTime - avgTime;

    // Calculate variance coefficient as a quality indicator
    const varianceCoefficient = avgTime > 0 ? ((maxTime - minTime) / avgTime) * 100 : 0;

    return {
      language: langInfo?.fullName || result.language,
      avgTime: Number(avgTime.toFixed(2)),
      avgOps: Number(avgOps.toFixed(0)),
      benchmarkCount: result.benchmarks.length,
      minTime: Number(minTime.toFixed(2)),
      maxTime: Number(maxTime.toFixed(2)),
      errorLow: Number(errorLow.toFixed(2)),
      errorHigh: Number(errorHigh.toFixed(2)),
      varianceCoefficient: Number(varianceCoefficient.toFixed(1))
    };
  });

  // Sort by performance (operations per second, descending)
  const sortedData = [...chartData].sort((a, b) => b.avgOps - a.avgOps);

  // If no data, show empty state
  if (chartData.length === 0) {
    return (
      <Card className={styles.chartCard} padding="xl" radius="md" withBorder>
        <Stack align="center" gap="md" className={styles.emptyState}>
          <Text size="lg" fw={600} c="dimmed">
            No benchmark results available
          </Text>
          <Text size="sm" c="dimmed" ta="center">
            Benchmark results will appear here once data is loaded
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
          Performance comparison across {benchmarkResults.length} implementations
        </Text>
      </Card.Section>

      {/* Enhanced Performance Summary */}
      <div className={styles.performanceSummary}>
        <SimpleGrid cols={{ base: 1, sm: 3 }} spacing="md">
          {sortedData.slice(0, 3).map((data, index) => {
            const consistencyColor = data.varianceCoefficient < 10 ? 'green' :
                                   data.varianceCoefficient < 25 ? 'yellow' : 'red';
            const consistencyLabel = data.varianceCoefficient < 10 ? 'High' :
                                   data.varianceCoefficient < 25 ? 'Medium' : 'Low';

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
                    color={consistencyColor}
                  >
                    {consistencyLabel} Consistency
                  </Badge>
                </Group>
                <Stack gap="xs">
                  <Text size="xs" c="dimmed">
                    {data.avgOps.toLocaleString()} ops/sec
                  </Text>
                  <Text size="xs" c="dimmed">
                    Avg: {data.avgTime}ms (±{data.varianceCoefficient.toFixed(1)}%)
                  </Text>
                  <Text size="xs" c="dimmed">
                    Range: {data.minTime}ms - {data.maxTime}ms
                  </Text>
                </Stack>
              </div>
            );
          })}
        </SimpleGrid>
      </div>

      {/* Operations per Second Chart */}
      <div className={styles.chartContainer}>
        <Text size="md" fw={500} mb="md">
          Operations per Second (Higher is Better)
        </Text>
        <ResponsiveContainer width="100%" height={300}>
          <BarChart
            data={sortedData}
            margin={{ top: 20, right: 30, left: 20, bottom: 60 }}
          >
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis
              dataKey="language"
              angle={-45}
              textAnchor="end"
              height={80}
              interval={0}
            />
            <YAxis label={{ value: 'Operations per Second', angle: -90, position: 'insideLeft' }} />
            <Tooltip
              formatter={(value: number, name: string, props: any) => {
                const data = props.payload;
                if (name === 'avgOps') {
                  return [
                    `${value.toLocaleString()} ops/sec (avg)`,
                    'Average Performance',
                    `Consistency: ${data.varianceCoefficient < 10 ? 'High' : data.varianceCoefficient < 25 ? 'Medium' : 'Low'} (${data.varianceCoefficient}% variance)`
                  ];
                }
                return [value, name];
              }}
              labelFormatter={(label) => `Implementation: ${label}`}
            />
            <Bar dataKey="avgOps" fill={chartColors.rust} />
          </BarChart>
        </ResponsiveContainer>
      </div>

      {/* Execution Time Chart with Error Bars */}
      <div className={styles.chartContainer}>
        <Text size="md" fw={500} mb="md">
          Average Execution Time with Variance (Lower is Better)
        </Text>
        <ResponsiveContainer width="100%" height={300}>
          <BarChart
            data={chartData.sort((a, b) => a.avgTime - b.avgTime)}
            margin={{ top: 20, right: 30, left: 20, bottom: 60 }}
          >
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis
              dataKey="language"
              angle={-45}
              textAnchor="end"
              height={80}
              interval={0}
            />
            <YAxis label={{ value: 'Execution Time (ms)', angle: -90, position: 'insideLeft' }} />
            <Tooltip
              formatter={(value: number, name: string, props: any) => {
                const data = props.payload;
                if (name === 'avgTime') {
                  return [
                    `${value.toFixed(2)} ms (avg)`,
                    'Average Time',
                    `Range: ${data.minTime.toFixed(2)} - ${data.maxTime.toFixed(2)} ms`,
                    `Variance: ${data.varianceCoefficient}%`
                  ];
                }
                return [value, name];
              }}
              labelFormatter={(label) => `Implementation: ${label}`}
            />
            <Bar dataKey="avgTime" fill={chartColors.java}>
              <ErrorBar
                dataKey="errorLow"
                width={4}
                stroke="#ef4444"
                strokeWidth={2}
              />
              <ErrorBar
                dataKey="errorHigh"
                width={4}
                stroke="#ef4444"
                strokeWidth={2}
              />
            </Bar>
          </BarChart>
        </ResponsiveContainer>
      </div>

      <div className={styles.chartFooter}>
        <Text size="xs" c="dimmed">
          Performance metrics are averaged across all benchmark tests for each implementation.
          Red error bars show performance variance (min-max range). Consistency badges indicate
          reliability: High (&lt;10% variance), Medium (10-25%), Low (&gt;25%). Higher operations
          per second and lower execution times with lower variance indicate better, more reliable performance.
        </Text>
      </div>
    </Card>
  );
}
