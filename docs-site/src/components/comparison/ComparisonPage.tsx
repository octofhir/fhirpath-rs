/**
 * Main Comparison Page Component
 * Entry point for the FHIRPath implementation comparison
 */

import {
	Container,
	Tabs,
	Alert,
	Stack,
	SimpleGrid,
	Title,
	Text,
	Group,
	Button,
} from "@mantine/core";
import {
	IconChartBar,
	IconCode,
	IconListDetails,
	IconAlertCircle,
	IconAnalyze,
	IconArrowLeft,
} from "@tabler/icons-react";
import {
	ComparisonThemeProvider,
	ThemeToggle,
	useComparisonTheme,
} from "./providers/ThemeProvider";

// Import hooks and components
import { useComparisonData } from "./hooks/useComparisonData";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { ErrorBoundary } from "./components/Common/ErrorBoundary";
import { LoadingSpinner } from "./components/Common/LoadingSpinner";
import {
	HeroSkeleton,
	MetricsSkeleton,
	ChartSkeleton,
	TableSkeleton,
	CardSkeleton
} from "./components/Common/SkeletonLoader";
import { HeroSection } from "./components/Overview/HeroSection";
import { MetricsCard } from "./components/Overview/MetricsCard";
import { LanguageCard } from "./components/Overview/LanguageCard";
import { TestResultsChart } from "./components/Charts/TestResultsChart";
import { BenchmarkChart } from "./components/Charts/BenchmarkChart";
import { TestResultsTable } from "./components/Details/TestResultsTable";
import { BenchmarkResultsTable } from "./components/Details/BenchmarkResultsTable";
import { ExpressionPatternView } from "./components/Analysis/ExpressionPatternView";

// Import styles
import styles from "./ComparisonPage.module.css";

import type { TabType } from "./types/comparison";
import { useState } from "react";

interface ComparisonPageProps {
	initialTab?: TabType;
}

export function ComparisonPage({
	initialTab = "overview",
}: ComparisonPageProps) {
	const [activeTab, setActiveTab] = useState<TabType>(initialTab);
	const { toggleColorScheme } = useComparisonTheme();
	const {
		testResults,
		benchmarkResults,
		languageInfo,
		metrics,
		loading,
		error,
	} = useComparisonData();

	// Keyboard shortcuts integration
	const { showHelp } = useKeyboardShortcuts({
		onTabChange: setActiveTab,
		onToggleTheme: toggleColorScheme,
		onSearch: () => {
			// Focus the first search input found on the page
			const searchInput = document.querySelector('input[placeholder*="search" i], input[placeholder*="filter" i]') as HTMLInputElement;
			if (searchInput) {
				searchInput.focus();
				searchInput.select();
			}
		},
		onExport: () => {
			// Trigger export functionality based on current tab
			const exportButton = document.querySelector('button[aria-label*="export" i], button[title*="export" i]') as HTMLButtonElement;
			if (exportButton) {
				exportButton.click();
			}
		},
		currentTab: activeTab,
		disabled: loading, // Disable shortcuts while loading
	});

	// Handle tab change
	const handleTabChange = (value: string | null) => {
		if (value && ["overview", "implementations", "details", "analysis"].includes(value)) {
			setActiveTab(value as TabType);
		}
	};

	// Render loading state with skeleton screens
	if (loading) {
		return (
			<Container size="xl" className={styles.container}>
				{/* Page Header Skeleton */}
				<Group justify="space-between" mb="xl">
					<div style={{ flex: 1 }}>
						<HeroSkeleton />
					</div>
				</Group>

				{/* Tab Navigation Skeleton */}
				<div className={styles.tabs}>
					<div className={styles.tabsList}>
						{Array.from({ length: 4 }).map((_, index) => (
							<div key={index} className={styles.tab} style={{ opacity: 0.3 }}>
								<div style={{ width: '80px', height: '16px', backgroundColor: 'var(--mantine-color-border)', borderRadius: '4px' }} />
							</div>
						))}
					</div>
					<div className={styles.tabPanel}>
						{/* Content based on active tab */}
						{activeTab === 'overview' && (
							<Stack gap="xl">
								<MetricsSkeleton />
								<ChartSkeleton />
								<ChartSkeleton />
							</Stack>
						)}
						{activeTab === 'implementations' && (
							<Stack gap="xl">
								<div>
									<div style={{ width: '300px', height: '24px', backgroundColor: 'var(--mantine-color-border)', borderRadius: '4px', marginBottom: '8px' }} />
									<div style={{ width: '500px', height: '16px', backgroundColor: 'var(--mantine-color-border)', borderRadius: '4px', marginBottom: '24px' }} />
								</div>
								<CardSkeleton count={6} />
							</Stack>
						)}
						{activeTab === 'details' && (
							<Stack gap="xl">
								<div>
									<div style={{ width: '200px', height: '24px', backgroundColor: 'var(--mantine-color-border)', borderRadius: '4px', marginBottom: '8px' }} />
									<div style={{ width: '400px', height: '16px', backgroundColor: 'var(--mantine-color-border)', borderRadius: '4px', marginBottom: '24px' }} />
								</div>
								<TableSkeleton rows={8} />
								<div style={{ marginTop: '40px' }}>
									<div style={{ width: '250px', height: '24px', backgroundColor: 'var(--mantine-color-border)', borderRadius: '4px', marginBottom: '8px' }} />
									<div style={{ width: '450px', height: '16px', backgroundColor: 'var(--mantine-color-border)', borderRadius: '4px', marginBottom: '24px' }} />
								</div>
								<TableSkeleton rows={6} />
							</Stack>
						)}
						{activeTab === 'analysis' && (
							<Stack gap="xl">
								<div>
									<div style={{ width: '280px', height: '24px', backgroundColor: 'var(--mantine-color-border)', borderRadius: '4px', marginBottom: '8px' }} />
									<div style={{ width: '600px', height: '16px', backgroundColor: 'var(--mantine-color-border)', borderRadius: '4px', marginBottom: '24px' }} />
								</div>
								<MetricsSkeleton />
								<ChartSkeleton />
								<TableSkeleton rows={10} />
							</Stack>
						)}
					</div>
				</div>
			</Container>
		);
	}

	// Render error state with data
	const ErrorAlert = error ? (
		<Alert
			icon={<IconAlertCircle size="1rem" />}
			title="Data Loading Warning"
			color="yellow"
			variant="light"
			className={styles.errorAlert}
		>
			{error} - Showing sample data for demonstration.
		</Alert>
	) : null;

	return (
		<ErrorBoundary>
			<Container size="xl" className={styles.container}>
				{/* Page Header with Back Link and Theme Toggle */}
				<Group justify="space-between" mb="xl">
					<div>
						<Title order={1} mb="xs">
							FHIRPath Implementation Comparison
						</Title>
						<Text size="lg" c="dimmed">
							Interactive visualization of FHIRPath implementation performance
							and compatibility
						</Text>
					</div>
					<Group gap="xl" className={styles.headerButtons}>
						<Button
							variant="subtle"
							leftSection={<IconArrowLeft size="1rem" />}
							component="a"
							href="/"
							size="sm"
							className={styles.backButton}
						>
							Back to Docs
						</Button>
						<ThemeToggle />
					</Group>
				</Group>

				{ErrorAlert}

				<Tabs
					value={activeTab}
					onChange={handleTabChange}
					className={styles.tabs}
				>
					<Tabs.List className={styles.tabsList}>
						<Tabs.Tab
							value="overview"
							leftSection={<IconChartBar size="1rem" />}
							className={styles.tab}
						>
							Overview
						</Tabs.Tab>
						<Tabs.Tab
							value="implementations"
							leftSection={<IconCode size="1rem" />}
							className={styles.tab}
						>
							Implementations
						</Tabs.Tab>
						<Tabs.Tab
							value="details"
							leftSection={<IconListDetails size="1rem" />}
							className={styles.tab}
						>
							Detailed Results
						</Tabs.Tab>
						<Tabs.Tab
							value="analysis"
							leftSection={<IconAnalyze size="1rem" />}
							className={styles.tab}
						>
							Expression Analysis
						</Tabs.Tab>
					</Tabs.List>

					<Tabs.Panel value="overview" className={styles.tabPanel}>
						<Stack gap="xl">
							<HeroSection
								metrics={metrics}
								testResultsCount={testResults.length}
							/>
							<MetricsCard
								metrics={metrics}
								benchmarkResults={benchmarkResults}
								testResults={testResults}
							/>

							<TestResultsChart testResults={testResults} />

							<BenchmarkChart benchmarkResults={benchmarkResults} />
						</Stack>
					</Tabs.Panel>

					<Tabs.Panel value="implementations" className={styles.tabPanel}>
						<Stack gap="xl">
							<div>
								<Title order={2} mb="md">
									FHIRPath Implementations
								</Title>
								<Text size="lg" c="dimmed" mb="xl">
									Compare different programming language implementations of the
									FHIRPath specification
								</Text>
							</div>

							<SimpleGrid cols={{ base: 1, sm: 2, lg: 3 }} spacing="lg">
								{Object.entries(languageInfo).map(([language, info]) => {
									// Find test results for this language
									const testResult = testResults.find(
										(result) => result.language === language,
									);
									const testsPassed = testResult?.summary?.passed;
									const totalTests = testResult?.summary?.total;

									// Find benchmark results for this language
									const benchmarkResult = benchmarkResults.find(
										(result) => result.language === language,
									);
									const avgPerformance = benchmarkResult?.benchmarks?.length
										? benchmarkResult.benchmarks.reduce(
												(sum, bench) => sum + bench.ops_per_second,
												0,
											) / benchmarkResult.benchmarks.length
										: undefined;

									return (
										<LanguageCard
											key={language}
											language={language}
											info={info}
											testsPassed={testsPassed}
											totalTests={totalTests}
											avgPerformance={avgPerformance}
											benchmarkResults={benchmarkResults}
											testResults={testResults}
										/>
									);
								})}
							</SimpleGrid>
						</Stack>
					</Tabs.Panel>

					<Tabs.Panel value="details" className={styles.tabPanel}>
						<Stack gap="xl">
							<Title order={2} mb="md">
								Detailed Test Results
							</Title>
							<Text size="lg" c="dimmed" mb="xl">
								Explore all test results by language. Sort and filter as needed.
							</Text>
							<TestResultsTable testResults={testResults} />
							<Title order={2} mt="xl" mb="md">
								Detailed Benchmark Results
							</Title>
							<Text size="lg" c="dimmed" mb="xl">
								Explore all benchmark results by language and operation. Sort
								and filter as needed.
							</Text>
							<BenchmarkResultsTable benchmarkResults={benchmarkResults} />
						</Stack>
					</Tabs.Panel>

					<Tabs.Panel value="analysis" className={styles.tabPanel}>
						<ExpressionPatternView
							testResults={testResults}
							benchmarkResults={benchmarkResults}
						/>
					</Tabs.Panel>
				</Tabs>
			</Container>
		</ErrorBoundary>
	);
}

/**
 * Wrapper component with theme provider
 * This ensures theme context and Mantine theme are available for the comparison page
 */
export function ComparisonPageWithProvider(props: ComparisonPageProps) {
	return (
		<ComparisonThemeProvider>
			<ComparisonPage {...props} />
		</ComparisonThemeProvider>
	);
}

// Export both versions for flexibility
export default ComparisonPageWithProvider;
