/**
 * Language card component for displaying programming language implementation details
 */

import { Card, Group, Text, Stack, Badge, Anchor, Image } from '@mantine/core';
import { IconExternalLink, IconBook, IconUsers, IconTrophy, IconTarget, IconAlertTriangle } from '@tabler/icons-react';
import type { LanguageInfo, BenchmarkResultSet, TestResultSet } from '../../types/comparison';
import { CompetitivePositioning } from '../../services/CompetitivePositioning';
import styles from './OverviewTab.module.css';

interface LanguageCardProps {
  language: string;
  info: LanguageInfo;
  testsPassed?: number;
  totalTests?: number;
  avgPerformance?: number;
  benchmarkResults?: BenchmarkResultSet[];
  testResults?: TestResultSet[];
}

export function LanguageCard({
  language,
  info,
  testsPassed,
  totalTests,
  avgPerformance,
  benchmarkResults,
  testResults
}: LanguageCardProps) {
  const successRate = testsPassed && totalTests ? Math.round((testsPassed / totalTests) * 100) : null;
  const logoPath = `/logos/${language}.svg`;

  // Calculate competitive positioning for Rust implementation
  let competitiveMetrics = null;
  const isRust = language === 'rust';

  if (isRust && benchmarkResults && testResults) {
    try {
      const competitivePositioning = new CompetitivePositioning(benchmarkResults, testResults);
      competitiveMetrics = competitivePositioning.calculateRustVsBest();
    } catch (error) {
      console.warn('Could not calculate competitive metrics for Rust:', error);
    }
  }

  return (
    <Card className={styles.languageCard} padding="lg" radius="md" withBorder>
      <Card.Section className={styles.languageCardHeader}>
        <Group justify="space-between" align="flex-start">
          <Group gap="md" align="center">
            <div className={styles.languageLogo}>
              <Image
                src={logoPath}
                alt={`${info.fullName} logo`}
                width={40}
                height={40}
                fallback={
                  <div className={styles.logoFallback}>
                    {info.icon}
                  </div>
                }
              />
            </div>
            <Stack gap="xs">
              <Text size="lg" fw={600} className={styles.languageName}>
                {info.fullName}
              </Text>
              <Text size="sm" c="dimmed" className={styles.languageRuntime}>
                {info.runtime} {info.runtime_version && `• ${info.runtime_version}`}
              </Text>
            </Stack>
          </Group>

          {successRate !== null && (
            <Stack gap="xs" align="flex-end">
              <Badge
                color={successRate >= 95 ? 'green' : successRate >= 85 ? 'yellow' : 'red'}
                variant="light"
                size="lg"
              >
                {successRate}% passed
              </Badge>
              {isRust && competitiveMetrics && (
                <Badge
                  color={competitiveMetrics.complianceGap > 0 ? 'green' : 'orange'}
                  variant="outline"
                  size="sm"
                  leftSection={competitiveMetrics.complianceGap > 0 ? <IconTrophy size="0.8rem" /> : <IconTarget size="0.8rem" />}
                >
                  {competitiveMetrics.complianceGap > 0 ? 'Leading' : `${Math.abs(competitiveMetrics.complianceGap).toFixed(1)}% behind`}
                </Badge>
              )}
            </Stack>
          )}
        </Group>
      </Card.Section>

      <Stack gap="md" className={styles.languageCardContent}>
        <Text size="sm" className={styles.languageDescription}>
          {info.description}
        </Text>

        <Group gap="xs" className={styles.languageLibraries}>
          {info.libraries.map((lib, index) => (
            <Badge key={index} variant="outline" size="sm">
              {lib}
            </Badge>
          ))}
        </Group>

        <Group justify="space-between" className={styles.languageStats}>
          <Stack gap="xs">
            <Text size="xs" c="dimmed">Maintainer</Text>
            <Group gap="xs" align="center">
              <IconUsers size="0.8rem" />
              <Text size="sm">{info.maintainer}</Text>
            </Group>
          </Stack>

          {avgPerformance && (
            <Stack gap="xs" align="flex-end">
              <Text size="xs" c="dimmed">Avg Performance</Text>
              <Text size="sm" fw={500}>
                {avgPerformance.toLocaleString()} ops/sec
              </Text>
              {isRust && competitiveMetrics && (
                <Badge
                  color={competitiveMetrics.performanceGap > -20 ? 'yellow' : competitiveMetrics.performanceGap > -50 ? 'orange' : 'red'}
                  variant="outline"
                  size="xs"
                  leftSection={competitiveMetrics.performanceGap > -20 ? <IconTarget size="0.7rem" /> : <IconAlertTriangle size="0.7rem" />}
                >
                  {Math.abs(competitiveMetrics.performanceGap).toFixed(0)}% behind {competitiveMetrics.bestPerformer}
                </Badge>
              )}
            </Stack>
          )}
        </Group>

        <Group justify="space-between" className={styles.languageActions}>
          <Anchor
            href={info.repository}
            target="_blank"
            rel="noopener noreferrer"
            size="sm"
            className={styles.languageLink}
          >
            <Group gap="xs" align="center">
              <IconBook size="0.8rem" />
              Repository
              <IconExternalLink size="0.7rem" />
            </Group>
          </Anchor>

          <Badge variant="outline" size="sm" color="gray">
            {info.license}
          </Badge>
        </Group>
      </Stack>
    </Card>
  );
}
