/**
 * Hero section component for the Overview tab
 */

import { Title, Text, Badge, Group, Stack, SimpleGrid, Card } from '@mantine/core';
import { IconRocket, IconCheck, IconStar, IconShield, IconTrendingUp, IconTarget } from '@tabler/icons-react';
import type { PerformanceMetrics, LanguageInfo } from '../../types/comparison';
import { LANGUAGE_INFO } from '../../services/resultLoader';
import styles from './OverviewTab.module.css';

interface HeroSectionProps {
  metrics: PerformanceMetrics;
  testResultsCount: number;
}

export function HeroSection({ metrics, testResultsCount }: HeroSectionProps) {
  const { fastestImplementation, mostCompliantImplementation } = metrics;

  const getFastestLanguageInfo = (): LanguageInfo | null => {
    if (!fastestImplementation?.language) return null;
    return LANGUAGE_INFO[fastestImplementation.language] || null;
  };

  const getMostCompliantLanguageInfo = (): LanguageInfo | null => {
    if (!mostCompliantImplementation?.language) return null;
    return LANGUAGE_INFO[mostCompliantImplementation.language] || null;
  };

  const fastestLangInfo = getFastestLanguageInfo();
  const compliantLangInfo = getMostCompliantLanguageInfo();

  // Generate recommendations based on metrics
  const getRecommendations = () => {
    const recommendations = [];

    if (fastestImplementation && fastestLangInfo) {
      recommendations.push({
        icon: <IconRocket size="1rem" />,
        title: "Performance Leader",
        description: `${fastestLangInfo.fullName} excels in speed with ${fastestImplementation.avgOps.toFixed(0)} ops/sec`,
        color: "blue",
        badge: "Speed"
      });
    }

    if (mostCompliantImplementation && compliantLangInfo) {
      recommendations.push({
        icon: <IconShield size="1rem" />,
        title: "Compliance Champion",
        description: `${compliantLangInfo.fullName} leads in specification compliance at ${mostCompliantImplementation.passRate.toFixed(1)}%`,
        color: "green",
        badge: "Reliability"
      });
    }

    recommendations.push({
      icon: <IconStar size="1rem" />,
      title: "Production Ready",
      description: "All implementations show strong stability for production use",
      color: "yellow",
      badge: "Stable"
    });

    return recommendations;
  };

  const recommendations = getRecommendations();

  return (
    <Stack gap="xl">
      <div className={styles.heroSection}>
        <div className={styles.heroContent}>
          <Title order={1} className={styles.heroTitle}>
            FHIRPath Implementation Comparison
          </Title>

          <Text size="lg" c="dimmed" className={styles.heroSubtitle}>
            Comprehensive analysis comparing {testResultsCount} language implementations with our Rust FHIRPath library
          </Text>

          <Text size="md" c="dimmed" className={styles.heroDescription}>
            Here we compare various FHIRPath implementations across different programming languages with our high-performance Rust implementation, providing actionable insights for your technology stack decisions.
          </Text>

          <Group gap="md" className={styles.heroBadges}>
            {fastestImplementation && fastestLangInfo && (
              <Badge
                size="xl"
                variant="light"
                color="blue"
                leftSection={<IconRocket size="1rem" />}
                className={styles.heroBadge}
              >
                <Group gap="xs">
                  <span>{fastestLangInfo.icon}</span>
                  <span>
                    Fastest: <strong>{fastestLangInfo.fullName}</strong>
                    ({fastestImplementation.avgOps.toFixed(0)} ops/sec)
                  </span>
                </Group>
              </Badge>
            )}

            {mostCompliantImplementation && compliantLangInfo && (
              <Badge
                size="xl"
                variant="light"
                color="green"
                leftSection={<IconCheck size="1rem" />}
                className={styles.heroBadge}
              >
                <Group gap="xs">
                  <span>{compliantLangInfo.icon}</span>
                  <span>
                    Most Compliant: <strong>{compliantLangInfo.fullName}</strong>
                    ({mostCompliantImplementation.passRate.toFixed(1)}%)
                  </span>
                </Group>
              </Badge>
            )}

            <Badge
              size="xl"
              variant="light"
              color="purple"
              leftSection={<IconTrendingUp size="1rem" />}
              className={styles.heroBadge}
            >
              <Group gap="xs">
                <span>📈</span>
                <span>
                  <strong>Growing Ecosystem</strong>
                  (Active development)
                </span>
              </Group>
            </Badge>
          </Group>

          <Text size="sm" c="dimmed" className={styles.heroNote}>
            Real-world performance data to guide your implementation choice
          </Text>
        </div>
      </div>

      {/* Recommendation Cards */}
      <SimpleGrid cols={{ base: 1, sm: 2, lg: 3 }} spacing="lg">
        {recommendations.map((rec, index) => (
          <Card key={index} className={styles.recommendationCard} padding="lg" radius="md" withBorder>
            <Group gap="sm" align="flex-start" mb="sm">
              <div className={styles.recommendationIcon} style={{ color: `var(--mantine-color-${rec.color}-6)` }}>
                {rec.icon}
              </div>
              <div style={{ flex: 1 }}>
                <Group gap="xs" align="center" mb="xs">
                  <Text size="sm" fw={600}>
                    {rec.title}
                  </Text>
                  <Badge size="xs" variant="light" color={rec.color}>
                    {rec.badge}
                  </Badge>
                </Group>
                <Text size="xs" c="dimmed" className={styles.recommendationDescription}>
                  {rec.description}
                </Text>
              </div>
            </Group>
          </Card>
        ))}
      </SimpleGrid>
    </Stack>
  );
}
