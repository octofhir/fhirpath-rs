/**
 * Advanced Filter Component
 * Provides multi-dimensional filtering for comparison data
 */

import { useState, useEffect } from 'react';
import {
  Card,
  Group,
  Text,
  Stack,
  Button,
  Collapse,
  RangeSlider,
  MultiSelect,
  Select,
  Switch,
  Badge,
  ActionIcon,
  Tooltip,
  NumberInput
} from '@mantine/core';
import {
  IconFilter,
  IconFilterOff,
  IconChevronDown,
  IconChevronUp,
  IconSettings,
  IconX
} from '@tabler/icons-react';
import type { BenchmarkResultSet, TestResultSet } from '../../types/comparison';
import { LANGUAGE_INFO } from '../../services/resultLoader';

export interface FilterCriteria {
  languages: string[];
  performanceRange: [number, number]; // ops per second range
  successRateRange: [number, number]; // percentage range
  executionTimeRange: [number, number]; // milliseconds range
  minBenchmarks: number;
  minTests: number;
  excludeErrors: boolean;
  onlyProductionReady: boolean;
  sortBy: 'performance' | 'compliance' | 'name';
  sortOrder: 'asc' | 'desc';
}

interface AdvancedFilterProps {
  benchmarkResults: BenchmarkResultSet[];
  testResults: TestResultSet[];
  onFilterChange: (criteria: FilterCriteria) => void;
  initialCriteria?: Partial<FilterCriteria>;
}

export function AdvancedFilter({
  benchmarkResults,
  testResults,
  onFilterChange,
  initialCriteria = {}
}: AdvancedFilterProps) {
  const [opened, setOpened] = useState(false);
  const [activeFilters, setActiveFilters] = useState(0);

  // Calculate data ranges for sliders
  const dataRanges = calculateDataRanges(benchmarkResults, testResults);

  // Initialize filter criteria
  const [criteria, setCriteria] = useState<FilterCriteria>({
    languages: initialCriteria.languages || Object.keys(LANGUAGE_INFO),
    performanceRange: initialCriteria.performanceRange || dataRanges.performance,
    successRateRange: initialCriteria.successRateRange || dataRanges.successRate,
    executionTimeRange: initialCriteria.executionTimeRange || dataRanges.executionTime,
    minBenchmarks: initialCriteria.minBenchmarks || 0,
    minTests: initialCriteria.minTests || 0,
    excludeErrors: initialCriteria.excludeErrors || false,
    onlyProductionReady: initialCriteria.onlyProductionReady || false,
    sortBy: initialCriteria.sortBy || 'performance',
    sortOrder: initialCriteria.sortOrder || 'desc'
  });

  // Update filter criteria and notify parent
  const updateCriteria = (updates: Partial<FilterCriteria>) => {
    const newCriteria = { ...criteria, ...updates };
    setCriteria(newCriteria);
    onFilterChange(newCriteria);
  };

  // Count active filters
  useEffect(() => {
    let count = 0;

    if (criteria.languages.length < Object.keys(LANGUAGE_INFO).length) count++;
    if (criteria.performanceRange[0] > dataRanges.performance[0] ||
        criteria.performanceRange[1] < dataRanges.performance[1]) count++;
    if (criteria.successRateRange[0] > dataRanges.successRate[0] ||
        criteria.successRateRange[1] < dataRanges.successRate[1]) count++;
    if (criteria.executionTimeRange[0] > dataRanges.executionTime[0] ||
        criteria.executionTimeRange[1] < dataRanges.executionTime[1]) count++;
    if (criteria.minBenchmarks > 0) count++;
    if (criteria.minTests > 0) count++;
    if (criteria.excludeErrors) count++;
    if (criteria.onlyProductionReady) count++;

    setActiveFilters(count);
  }, [criteria, dataRanges]);

  // Reset all filters
  const resetFilters = () => {
    const defaultCriteria: FilterCriteria = {
      languages: Object.keys(LANGUAGE_INFO),
      performanceRange: dataRanges.performance,
      successRateRange: dataRanges.successRate,
      executionTimeRange: dataRanges.executionTime,
      minBenchmarks: 0,
      minTests: 0,
      excludeErrors: false,
      onlyProductionReady: false,
      sortBy: 'performance',
      sortOrder: 'desc'
    };
    setCriteria(defaultCriteria);
    onFilterChange(defaultCriteria);
  };

  // Language options for MultiSelect
  const languageOptions = Object.entries(LANGUAGE_INFO).map(([key, info]) => ({
    value: key,
    label: `${info.fullName} (${key})`
  }));

  return (
    <Card padding="md" radius="md" withBorder>
      <Group justify="space-between" align="center" mb="sm">
        <Group gap="xs" align="center">
          <IconFilter size="1.1rem" />
          <Text size="sm" fw={600}>
            Advanced Filters
          </Text>
          {activeFilters > 0 && (
            <Badge size="sm" variant="filled" color="blue">
              {activeFilters} active
            </Badge>
          )}
        </Group>

        <Group gap="xs">
          {activeFilters > 0 && (
            <Tooltip label="Reset all filters">
              <ActionIcon
                variant="subtle"
                color="gray"
                size="sm"
                onClick={resetFilters}
              >
                <IconFilterOff size="1rem" />
              </ActionIcon>
            </Tooltip>
          )}
          <ActionIcon
            variant="subtle"
            onClick={() => setOpened(!opened)}
            aria-label="Toggle filters"
          >
            {opened ? <IconChevronUp size="1rem" /> : <IconChevronDown size="1rem" />}
          </ActionIcon>
        </Group>
      </Group>

      <Collapse in={opened}>
        <Stack gap="lg" mt="md">
          {/* Language Selection */}
          <div>
            <Text size="sm" fw={500} mb="xs">
              Languages
            </Text>
            <MultiSelect
              data={languageOptions}
              value={criteria.languages}
              onChange={(value) => updateCriteria({ languages: value })}
              placeholder="Select languages to compare"
              searchable
              clearable
              maxDropdownHeight={200}
            />
          </div>

          {/* Performance Range */}
          <div>
            <Group justify="space-between" align="center" mb="xs">
              <Text size="sm" fw={500}>
                Performance Range (ops/sec)
              </Text>
              <Text size="xs" c="dimmed">
                {criteria.performanceRange[0].toLocaleString()} - {criteria.performanceRange[1].toLocaleString()}
              </Text>
            </Group>
            <RangeSlider
              min={dataRanges.performance[0]}
              max={dataRanges.performance[1]}
              step={100}
              value={criteria.performanceRange}
              onChange={(value) => updateCriteria({ performanceRange: value })}
              marks={[
                { value: dataRanges.performance[0], label: dataRanges.performance[0].toLocaleString() },
                { value: dataRanges.performance[1], label: dataRanges.performance[1].toLocaleString() }
              ]}
            />
          </div>

          {/* Success Rate Range */}
          <div>
            <Group justify="space-between" align="center" mb="xs">
              <Text size="sm" fw={500}>
                Success Rate Range (%)
              </Text>
              <Text size="xs" c="dimmed">
                {criteria.successRateRange[0]}% - {criteria.successRateRange[1]}%
              </Text>
            </Group>
            <RangeSlider
              min={dataRanges.successRate[0]}
              max={dataRanges.successRate[1]}
              step={1}
              value={criteria.successRateRange}
              onChange={(value) => updateCriteria({ successRateRange: value })}
              marks={[
                { value: dataRanges.successRate[0], label: `${dataRanges.successRate[0]}%` },
                { value: dataRanges.successRate[1], label: `${dataRanges.successRate[1]}%` }
              ]}
            />
          </div>

          {/* Execution Time Range */}
          <div>
            <Group justify="space-between" align="center" mb="xs">
              <Text size="sm" fw={500}>
                Execution Time Range (ms)
              </Text>
              <Text size="xs" c="dimmed">
                {criteria.executionTimeRange[0]}ms - {criteria.executionTimeRange[1]}ms
              </Text>
            </Group>
            <RangeSlider
              min={dataRanges.executionTime[0]}
              max={dataRanges.executionTime[1]}
              step={0.1}
              value={criteria.executionTimeRange}
              onChange={(value) => updateCriteria({ executionTimeRange: value })}
              marks={[
                { value: dataRanges.executionTime[0], label: `${dataRanges.executionTime[0]}ms` },
                { value: dataRanges.executionTime[1], label: `${dataRanges.executionTime[1]}ms` }
              ]}
            />
          </div>

          {/* Minimum Requirements */}
          <Group grow>
            <div>
              <Text size="sm" fw={500} mb="xs">
                Min Benchmarks
              </Text>
              <NumberInput
                value={criteria.minBenchmarks}
                onChange={(value) => updateCriteria({ minBenchmarks: Number(value) || 0 })}
                min={0}
                max={50}
                placeholder="0"
              />
            </div>
            <div>
              <Text size="sm" fw={500} mb="xs">
                Min Tests
              </Text>
              <NumberInput
                value={criteria.minTests}
                onChange={(value) => updateCriteria({ minTests: Number(value) || 0 })}
                min={0}
                max={1000}
                placeholder="0"
              />
            </div>
          </Group>

          {/* Quality Filters */}
          <Stack gap="sm">
            <Text size="sm" fw={500}>
              Quality Filters
            </Text>
            <Switch
              label="Exclude implementations with errors"
              description="Hide languages that have test errors"
              checked={criteria.excludeErrors}
              onChange={(event) => updateCriteria({ excludeErrors: event.currentTarget.checked })}
            />
            <Switch
              label="Production-ready only"
              description="Show only stable, well-maintained implementations"
              checked={criteria.onlyProductionReady}
              onChange={(event) => updateCriteria({ onlyProductionReady: event.currentTarget.checked })}
            />
          </Stack>

          {/* Sorting */}
          <Group grow>
            <div>
              <Text size="sm" fw={500} mb="xs">
                Sort By
              </Text>
              <Select
                data={[
                  { value: 'performance', label: 'Performance' },
                  { value: 'compliance', label: 'Compliance' },
                  { value: 'name', label: 'Name' }
                ]}
                value={criteria.sortBy}
                onChange={(value) => updateCriteria({ sortBy: value as any })}
              />
            </div>
            <div>
              <Text size="sm" fw={500} mb="xs">
                Sort Order
              </Text>
              <Select
                data={[
                  { value: 'desc', label: 'Descending' },
                  { value: 'asc', label: 'Ascending' }
                ]}
                value={criteria.sortOrder}
                onChange={(value) => updateCriteria({ sortOrder: value as any })}
              />
            </div>
          </Group>
        </Stack>
      </Collapse>
    </Card>
  );
}

/**
 * Calculate data ranges for filter sliders
 */
function calculateDataRanges(
  benchmarkResults: BenchmarkResultSet[],
  testResults: TestResultSet[]
): {
  performance: [number, number];
  successRate: [number, number];
  executionTime: [number, number];
} {
  // Performance range (ops per second)
  const allOpsPerSecond = benchmarkResults.flatMap(result =>
    result.benchmarks.map(b => b.ops_per_second || 0)
  ).filter(ops => ops > 0);

  const minOps = Math.min(...allOpsPerSecond);
  const maxOps = Math.max(...allOpsPerSecond);

  // Success rate range
  const successRates = testResults.map(result => {
    const summary = result.summary || { total: 0, passed: 0 };
    return summary.total > 0 ? (summary.passed / summary.total) * 100 : 0;
  });

  const minSuccessRate = Math.floor(Math.min(...successRates));
  const maxSuccessRate = Math.ceil(Math.max(...successRates));

  // Execution time range
  const allExecutionTimes = benchmarkResults.flatMap(result =>
    result.benchmarks.map(b => b.avg_time_ms || 0)
  ).filter(time => time > 0);

  const minTime = Math.min(...allExecutionTimes);
  const maxTime = Math.max(...allExecutionTimes);

  return {
    performance: [Math.floor(minOps), Math.ceil(maxOps)],
    successRate: [minSuccessRate, maxSuccessRate],
    executionTime: [Number(minTime.toFixed(1)), Number(maxTime.toFixed(1))]
  };
}
