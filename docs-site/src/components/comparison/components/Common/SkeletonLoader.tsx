/**
 * Skeleton Loading Components
 * Provides content-aware skeleton screens for better loading experience
 */

import { Skeleton, Stack, Group, Card, SimpleGrid } from "@mantine/core";
import { useMantineTheme } from "@mantine/core";
import styles from "./skeleton.module.css";

interface SkeletonLoaderProps {
  variant: 'hero' | 'metrics' | 'chart' | 'table' | 'card' | 'text';
  count?: number;
  animated?: boolean;
}

export function SkeletonLoader({
  variant,
  count = 1,
  animated = true
}: SkeletonLoaderProps) {
  const theme = useMantineTheme();

  const renderSkeleton = () => {
    switch (variant) {
      case 'hero':
        return (
          <div className={styles.heroSkeleton}>
            <Stack gap="md" align="center">
              <Skeleton height={40} width="60%" animate={animated} />
              <Skeleton height={20} width="80%" animate={animated} />
              <Group gap="md" justify="center">
                <Skeleton height={32} width={120} animate={animated} />
                <Skeleton height={32} width={140} animate={animated} />
              </Group>
              <Skeleton height={16} width="50%" animate={animated} />
            </Stack>
          </div>
        );

      case 'metrics':
        return (
          <SimpleGrid cols={{ base: 1, sm: 2, lg: 4 }} spacing="md">
            {Array.from({ length: 4 }).map((_, index) => (
              <Card key={index} padding="lg" className={styles.metricSkeleton}>
                <Group justify="space-between">
                  <Stack gap="xs">
                    <Skeleton height={12} width="70%" animate={animated} />
                    <Skeleton height={24} width="50%" animate={animated} />
                  </Stack>
                  <Skeleton height={32} width={32} circle animate={animated} />
                </Group>
              </Card>
            ))}
          </SimpleGrid>
        );

      case 'chart':
        return (
          <Card padding="lg" className={styles.chartSkeleton}>
            <Stack gap="md">
              <Group justify="space-between">
                <Skeleton height={20} width="40%" animate={animated} />
                <Skeleton height={16} width="20%" animate={animated} />
              </Group>
              <div className={styles.chartArea}>
                <Stack gap="xs">
                  {Array.from({ length: 6 }).map((_, index) => (
                    <Group key={index} gap="md" align="center">
                      <Skeleton height={8} width="15%" animate={animated} />
                      <Skeleton
                        height={12}
                        width={`${Math.random() * 60 + 20}%`}
                        animate={animated}
                      />
                    </Group>
                  ))}
                </Stack>
              </div>
            </Stack>
          </Card>
        );

      case 'table':
        return (
          <Card padding="lg" className={styles.tableSkeleton}>
            <Stack gap="md">
              <Group justify="space-between">
                <Skeleton height={16} width="30%" animate={animated} />
                <Skeleton height={32} width={120} animate={animated} />
              </Group>
              <Stack gap="xs">
                {/* Table Header */}
                <Group gap="md">
                  {Array.from({ length: 5 }).map((_, index) => (
                    <Skeleton key={index} height={14} width="15%" animate={animated} />
                  ))}
                </Group>
                {/* Table Rows */}
                {Array.from({ length: count || 5 }).map((_, rowIndex) => (
                  <Group key={rowIndex} gap="md">
                    {Array.from({ length: 5 }).map((_, colIndex) => (
                      <Skeleton
                        key={colIndex}
                        height={12}
                        width={colIndex === 0 ? "20%" : "15%"}
                        animate={animated}
                      />
                    ))}
                  </Group>
                ))}
              </Stack>
            </Stack>
          </Card>
        );

      case 'card':
        return (
          <SimpleGrid cols={{ base: 1, sm: 2, lg: 3 }} spacing="md">
            {Array.from({ length: count || 3 }).map((_, index) => (
              <Card key={index} padding="lg" className={styles.cardSkeleton}>
                <Stack gap="md">
                  <Group>
                    <Skeleton height={40} width={40} circle animate={animated} />
                    <Stack gap="xs" style={{ flex: 1 }}>
                      <Skeleton height={16} width="70%" animate={animated} />
                      <Skeleton height={12} width="50%" animate={animated} />
                    </Stack>
                  </Group>
                  <Stack gap="xs">
                    <Skeleton height={10} width="100%" animate={animated} />
                    <Skeleton height={10} width="80%" animate={animated} />
                    <Skeleton height={10} width="60%" animate={animated} />
                  </Stack>
                  <Group gap="xs">
                    <Skeleton height={24} width={60} animate={animated} />
                    <Skeleton height={24} width={80} animate={animated} />
                  </Group>
                </Stack>
              </Card>
            ))}
          </SimpleGrid>
        );

      case 'text':
        return (
          <Stack gap="xs">
            {Array.from({ length: count || 3 }).map((_, index) => (
              <Skeleton
                key={index}
                height={16}
                width={index === count - 1 ? "60%" : "100%"}
                animate={animated}
              />
            ))}
          </Stack>
        );

      default:
        return <Skeleton height={20} animate={animated} />;
    }
  };

  return <div className={styles.skeletonContainer}>{renderSkeleton()}</div>;
}

// Specialized skeleton components for common use cases
export function HeroSkeleton({ animated = true }: { animated?: boolean }) {
  return <SkeletonLoader variant="hero" animated={animated} />;
}

export function MetricsSkeleton({ animated = true }: { animated?: boolean }) {
  return <SkeletonLoader variant="metrics" animated={animated} />;
}

export function ChartSkeleton({ animated = true }: { animated?: boolean }) {
  return <SkeletonLoader variant="chart" animated={animated} />;
}

export function TableSkeleton({
  rows = 5,
  animated = true
}: {
  rows?: number;
  animated?: boolean;
}) {
  return <SkeletonLoader variant="table" count={rows} animated={animated} />;
}

export function CardSkeleton({
  count = 3,
  animated = true
}: {
  count?: number;
  animated?: boolean;
}) {
  return <SkeletonLoader variant="card" count={count} animated={animated} />;
}

export function TextSkeleton({
  lines = 3,
  animated = true
}: {
  lines?: number;
  animated?: boolean;
}) {
  return <SkeletonLoader variant="text" count={lines} animated={animated} />;
}

export default SkeletonLoader;
