/**
 * Expression Pattern Analysis View
 * Displays comprehensive analysis of FHIRPath expression patterns across implementations
 */

import {useMemo, useState} from "react";
import {
	Accordion,
	Alert,
	Badge,
	Card,
	Center,
	Code,
	Group,
	Progress,
	RingProgress,
	Select,
	SimpleGrid,
	Stack,
	Table,
	Tabs,
	Text,
	TextInput,
	Title,
	useMantineTheme,
} from "@mantine/core";
import {
	IconAlertTriangle,
	IconChartPie,
	IconCode,
	IconFilter,
	IconSearch,
	IconSpeedboat,
	IconTarget,
} from "@tabler/icons-react";
import {
	Bar,
	BarChart,
	CartesianGrid,
	Cell,
	Pie,
	PieChart,
	ResponsiveContainer,
	Tooltip as RechartsTooltip,
	XAxis,
	YAxis
} from "recharts";

import {
	ExpressionAnalyzer,
	type ExpressionCategory,
	type ExpressionComplexity,
	type ExpressionPattern,
} from "../../services/ExpressionAnalyzer";
import type {BenchmarkResultSet, TestResultSet} from "../../types/comparison";

interface ExpressionPatternViewProps {
    testResults: TestResultSet[];
    benchmarkResults: BenchmarkResultSet[];
}

const CATEGORY_COLORS: Record<ExpressionCategory, string> = {
    navigation: "#3b82f6",
    filtering: "#10b981",
    aggregation: "#f59e0b",
    string_manipulation: "#ef4444",
    type_conversion: "#8b5cf6",
    mathematical: "#06b6d4",
    conditional: "#f97316",
    collection_operations: "#84cc16",
    date_time: "#ec4899",
    reference_handling: "#6366f1",
    complex_mixed: "#64748b",
};

const COMPLEXITY_COLORS: Record<ExpressionComplexity, string> = {
    simple: "#22c55e",
    moderate: "#eab308",
    complex: "#f97316",
    very_complex: "#ef4444",
};

export function ExpressionPatternView({
                                          testResults,
                                          benchmarkResults,
                                      }: ExpressionPatternViewProps) {
    const theme = useMantineTheme();
    const [searchTerm, setSearchTerm] = useState("");
    const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
    const [selectedComplexity, setSelectedComplexity] = useState<string | null>(null);
    const [activeTab, setActiveTab] = useState("overview");

    // Create analyzer and get analysis
    const analyzer = useMemo(
        () => new ExpressionAnalyzer(testResults, benchmarkResults),
        [testResults, benchmarkResults],
    );

    const analysis = useMemo(() => analyzer.getAnalysis(), [analyzer]);

    // Filter patterns based on search and filters
    const filteredPatterns = useMemo(() => {
        let patterns = analyzer.getAllPatterns();

        if (searchTerm) {
            patterns = analyzer.searchPatterns(searchTerm);
        }

        if (selectedCategory) {
            patterns = patterns.filter(p => p.category === selectedCategory);
        }

        if (selectedComplexity) {
            patterns = patterns.filter(p => p.complexity === selectedComplexity);
        }

        return patterns;
    }, [analyzer, searchTerm, selectedCategory, selectedComplexity]);

    // Prepare chart data
    const categoryChartData = Object.entries(analysis.categoryDistribution).map(
        ([category, count]) => ({
            category: category.replace(/_/g, " "),
            count,
            color: CATEGORY_COLORS[category as ExpressionCategory],
        }),
    );

    const complexityChartData = Object.entries(analysis.complexityDistribution).map(
        ([complexity, count]) => ({
            complexity,
            count,
            color: COMPLEXITY_COLORS[complexity as ExpressionComplexity],
        }),
    );

    const compatibilityData = [
        {
            name: "Fully Compatible",
            value: analysis.crossLanguageCompatibility.fullyCompatible,
            color: "#22c55e",
        },
        {
            name: "Mostly Compatible",
            value: analysis.crossLanguageCompatibility.mostlyCompatible,
            color: "#eab308",
        },
        {
            name: "Partially Compatible",
            value: analysis.crossLanguageCompatibility.partiallyCompatible,
            color: "#f97316",
        },
        {
            name: "Poorly Compatible",
            value: analysis.crossLanguageCompatibility.poorlyCompatible,
            color: "#ef4444",
        },
    ];

    const formatCategoryName = (category: string) => {
        return category.replace(/_/g, " ").replace(/\b\w/g, l => l.toUpperCase());
    };

    const getCompatibilityColor = (pattern: ExpressionPattern) => {
        const passedCount = pattern.languages.filter(l => l.status === "passed").length;
        const totalCount = pattern.languages.length;
        const rate = passedCount / totalCount;

        if (rate === 1) return "green";
        if (rate >= 0.75) return "yellow";
        if (rate >= 0.5) return "orange";
        return "red";
    };

    const getCompatibilityRate = (pattern: ExpressionPattern) => {
        const passedCount = pattern.languages.filter(l => l.status === "passed").length;
        const totalCount = pattern.languages.length;
        return totalCount > 0 ? Math.round((passedCount / totalCount) * 100) : 0;
    };

    return (
        <Stack gap="xl">
            {/* Header */}
            <div>
                <Title order={2} mb="md">
                    Expression Pattern Analysis
                </Title>
                <Text size="lg" c="dimmed">
                    Comprehensive analysis of FHIRPath expression patterns, complexity, and
                    cross-language compatibility
                </Text>
            </div>

            <Tabs value={activeTab} onChange={setActiveTab}>
                <Tabs.List>
                    <Tabs.Tab value="overview" leftSection={<IconChartPie size="1rem"/>}>
                        Overview
                    </Tabs.Tab>
                    <Tabs.Tab value="patterns" leftSection={<IconCode size="1rem"/>}>
                        Expression Patterns
                    </Tabs.Tab>
                    <Tabs.Tab value="compatibility" leftSection={<IconTarget size="1rem"/>}>
                        Compatibility Matrix
                    </Tabs.Tab>
                    <Tabs.Tab value="performance" leftSection={<IconSpeedboat size="1rem"/>}>
                        Performance Insights
                    </Tabs.Tab>
                </Tabs.List>

                <Tabs.Panel value="overview" pt="md">
                    <SimpleGrid cols={{base: 1, md: 2, lg: 4}} spacing="md" mb="xl">
                        <Card padding="lg" style={{background: theme.other.surface}}>
                            <Group justify="space-between">
                                <div>
                                    <Text size="xs" tt="uppercase" fw={700} c="dimmed">
                                        Total Expressions
                                    </Text>
                                    <Text fw={700} size="xl">
                                        {analysis.totalExpressions.toLocaleString()}
                                    </Text>
                                </div>
                                <IconCode size="2rem" color={theme.colors.blue[6]}/>
                            </Group>
                        </Card>

                        <Card padding="lg" style={{background: theme.other.surface}}>
                            <Group justify="space-between">
                                <div>
                                    <Text size="xs" tt="uppercase" fw={700} c="dimmed">
                                        Categories
                                    </Text>
                                    <Text fw={700} size="xl">
                                        {Object.keys(analysis.categoryDistribution).length}
                                    </Text>
                                </div>
                                <IconFilter size="2rem" color={theme.colors.green[6]}/>
                            </Group>
                        </Card>

                        <Card padding="lg" style={{background: theme.other.surface}}>
                            <Group justify="space-between">
                                <div>
                                    <Text size="xs" tt="uppercase" fw={700} c="dimmed">
                                        Fully Compatible
                                    </Text>
                                    <Text fw={700} size="xl">
                                        {analysis.crossLanguageCompatibility.fullyCompatible}
                                    </Text>
                                </div>
                                <IconTarget size="2rem" color={theme.colors.green[6]}/>
                            </Group>
                        </Card>

                        <Card padding="lg" style={{background: theme.other.surface}}>
                            <Group justify="space-between">
                                <div>
                                    <Text size="xs" tt="uppercase" fw={700} c="dimmed">
                                        Failing Expressions
                                    </Text>
                                    <Text fw={700} size="xl">
                                        {analysis.topFailingExpressions.length}
                                    </Text>
                                </div>
                                <IconAlertTriangle size="2rem" color={theme.colors.red[6]}/>
                            </Group>
                        </Card>
                    </SimpleGrid>

                    {/* Charts */}
                    <SimpleGrid cols={{base: 1, lg: 2}} spacing="xl">
                        <Card padding="lg" style={{background: theme.other.surface}}>
                            <Title order={4} mb="md">
                                Expression Categories
                            </Title>
                            <ResponsiveContainer width="100%" height={300}>
                                <BarChart data={categoryChartData}>
                                    <CartesianGrid strokeDasharray="3 3"/>
                                    <XAxis
                                        dataKey="category"
                                        angle={-45}
                                        textAnchor="end"
                                        height={100}
                                        fontSize={12}
                                    />
                                    <YAxis/>
                                    <RechartsTooltip/>
                                    <Bar dataKey="count" fill="#3b82f6"/>
                                </BarChart>
                            </ResponsiveContainer>
                        </Card>

                        <Card padding="lg" style={{background: theme.other.surface}}>
                            <Title order={4} mb="md">
                                Complexity Distribution
                            </Title>
                            <ResponsiveContainer width="100%" height={300}>
                                <PieChart>
                                    <Pie
                                        data={complexityChartData}
                                        cx="50%"
                                        cy="50%"
                                        outerRadius={100}
                                        fill="#8884d8"
                                        dataKey="count"
                                        label={({complexity, count}) => `${complexity}: ${count}`}
                                    >
                                        {complexityChartData.map((entry, index) => (
                                            <Cell key={`cell-${index}`} fill={entry.color}/>
                                        ))}
                                    </Pie>
                                    <RechartsTooltip/>
                                </PieChart>
                            </ResponsiveContainer>
                        </Card>
                    </SimpleGrid>

                    {/* Performance Insights */}
                    <Card padding="lg" style={{background: theme.other.surface}} mt="xl">
                        <Title order={4} mb="md">
                            Performance Insights
                        </Title>
                        <SimpleGrid cols={{base: 1, md: 3}} spacing="md">
                            <div>
                                <Text size="sm" fw={500} mb="xs">
                                    Fastest Category
                                </Text>
                                <Badge color="green" size="lg">
                                    {formatCategoryName(analysis.performanceInsights.fastestCategory)}
                                </Badge>
                            </div>
                            <div>
                                <Text size="sm" fw={500} mb="xs">
                                    Slowest Category
                                </Text>
                                <Badge color="red" size="lg">
                                    {formatCategoryName(analysis.performanceInsights.slowestCategory)}
                                </Badge>
                            </div>
                            <div>
                                <Text size="sm" fw={500} mb="xs">
                                    Most Variable Category
                                </Text>
                                <Badge color="orange" size="lg">
                                    {formatCategoryName(analysis.performanceInsights.mostVariableCategory)}
                                </Badge>
                            </div>
                        </SimpleGrid>
                    </Card>
                </Tabs.Panel>

                <Tabs.Panel value="patterns" pt="md">
                    {/* Filters */}
                    <Group mb="md">
                        <TextInput
                            placeholder="Search expressions..."
                            value={searchTerm}
                            onChange={(e) => setSearchTerm(e.target.value)}
                            leftSection={<IconSearch size="1rem"/>}
                            style={{flex: 1}}
                        />
                        <Select
                            placeholder="Filter by category"
                            data={Object.keys(analysis.categoryDistribution).map((cat) => ({
                                value: cat,
                                label: formatCategoryName(cat),
                            }))}
                            value={selectedCategory}
                            onChange={setSelectedCategory}
                            clearable
                            w={200}
                        />
                        <Select
                            placeholder="Filter by complexity"
                            data={Object.keys(analysis.complexityDistribution).map((comp) => ({
                                value: comp,
                                label: comp.charAt(0).toUpperCase() + comp.slice(1),
                            }))}
                            value={selectedComplexity}
                            onChange={setSelectedComplexity}
                            clearable
                            w={200}
                        />
                    </Group>

                    {/* Results */}
                    <Text size="sm" c="dimmed" mb="md">
                        Showing {filteredPatterns.length} of {analysis.totalExpressions} expressions
                    </Text>

                    <Accordion variant="separated">
                        {filteredPatterns.slice(0, 50).map((pattern, index) => (
                            <Accordion.Item
                                key={`${pattern.expression}-${index}`}
                                value={`${pattern.expression}-${index}`}
                            >
                                <Accordion.Control>
                                    <Group justify="space-between" w="100%">
                                        <Group>
                                            <Badge color={CATEGORY_COLORS[pattern.category]} variant="light">
                                                {formatCategoryName(pattern.category)}
                                            </Badge>
                                            <Badge color={COMPLEXITY_COLORS[pattern.complexity]} variant="light">
                                                {pattern.complexity}
                                            </Badge>
                                            <Badge color={getCompatibilityColor(pattern)} variant="light">
                                                {getCompatibilityRate(pattern)}% compatible
                                            </Badge>
                                        </Group>
                                    </Group>
                                </Accordion.Control>
                                <Accordion.Panel>
                                    <Stack gap="md">
                                        <div>
                                            <Text size="sm" fw={500} mb="xs">
                                                Expression:
                                            </Text>
                                            <Code
                                                block
                                                style={{
                                                    background: theme.other.surfaceSecondary,
                                                    color: theme.other.text,
                                                    border: `1px solid ${theme.other.border}`,
                                                }}
                                            >
                                                {pattern.expression}
                                            </Code>
                                        </div>

                                        <div>
                                            <Text size="sm" fw={500} mb="xs">
                                                Operations: {pattern.operations.join(", ")}
                                            </Text>
                                        </div>

                                        <div>
                                            <Text size="sm" fw={500} mb="xs">
                                                Language Results:
                                            </Text>
                                            <Table size="sm">
                                                <Table.Thead>
                                                    <Table.Tr>
                                                        <Table.Th>Language</Table.Th>
                                                        <Table.Th>Status</Table.Th>
                                                        <Table.Th>Execution Time</Table.Th>
                                                    </Table.Tr>
                                                </Table.Thead>
                                                <Table.Tbody>
                                                    {pattern.languages.map((lang, idx) => (
                                                        <Table.Tr key={idx}>
                                                            <Table.Td>{lang.language}</Table.Td>
                                                            <Table.Td>
                                                                <Badge
                                                                    color={
                                                                        lang.status === "passed"
                                                                            ? "green"
                                                                            : lang.status === "failed"
                                                                                ? "red"
                                                                                : "orange"
                                                                    }
                                                                    variant="light"
                                                                    size="sm"
                                                                >
                                                                    {lang.status}
                                                                </Badge>
                                                            </Table.Td>
                                                            <Table.Td>
                                                                {lang.execution_time_ms
                                                                    ? `${lang.execution_time_ms.toFixed(2)}ms`
                                                                    : "-"}
                                                            </Table.Td>
                                                        </Table.Tr>
                                                    ))}
                                                </Table.Tbody>
                                            </Table>
                                        </div>
                                    </Stack>
                                </Accordion.Panel>
                            </Accordion.Item>
                        ))}
                    </Accordion>

                    {filteredPatterns.length > 50 && (
                        <Alert color="blue" variant="light" mt="md">
                            Showing first 50 results. Use filters to narrow down the search.
                        </Alert>
                    )}
                </Tabs.Panel>

                <Tabs.Panel value="compatibility" pt="md">
                    <Card padding="lg" style={{background: theme.other.surface}}>
                        <Title order={4} mb="md">
                            Cross-Language Compatibility
                        </Title>
                        <SimpleGrid cols={{base: 1, md: 2}} spacing="xl">
                            <div>
                                <ResponsiveContainer width="100%" height={300}>
                                    <PieChart>
                                        <Pie
                                            data={compatibilityData}
                                            cx="50%"
                                            cy="50%"
                                            outerRadius={100}
                                            fill="#8884d8"
                                            dataKey="value"
                                            label={({name, value}) => `${name}: ${value}`}
                                        >
                                            {compatibilityData.map((entry, index) => (
                                                <Cell key={`cell-${index}`} fill={entry.color}/>
                                            ))}
                                        </Pie>
                                        <RechartsTooltip/>
                                    </PieChart>
                                </ResponsiveContainer>
                            </div>
                            <Stack gap="md">
                                {compatibilityData.map((item) => (
                                    <Group key={item.name} justify="space-between">
                                        <Group>
                                            <div
                                                style={{
                                                    width: 12,
                                                    height: 12,
                                                    backgroundColor: item.color,
                                                    borderRadius: 2,
                                                }}
                                            />
                                            <Text size="sm">{item.name}</Text>
                                        </Group>
                                        <Badge color="gray" variant="light">
                                            {item.value} expressions
                                        </Badge>
                                    </Group>
                                ))}
                            </Stack>
                        </SimpleGrid>
                    </Card>

                    {analysis.topFailingExpressions.length > 0 && (
                        <Card padding="lg" style={{background: theme.other.surface}} mt="xl">
                            <Title order={4} mb="md">
                                Top Failing Expressions
                            </Title>
                            <Stack gap="md">
                                {analysis.topFailingExpressions.slice(0, 5).map((pattern, index) => (
                                    <div key={index}>
                                        <Group justify="space-between" mb="xs">
                                            <Text size="sm" fw={500}>
                                                {pattern.expression.length > 80
                                                    ? `${pattern.expression.substring(0, 80)}...`
                                                    : pattern.expression}
                                            </Text>
                                            <Badge color="red" variant="light">
                                                {pattern.languages.filter(l => l.status === "failed" || l.status === "error").length} failures
                                            </Badge>
                                        </Group>
                                        <Progress
                                            value={getCompatibilityRate(pattern)}
                                            color={getCompatibilityColor(pattern)}
                                            size="sm"
                                        />
                                    </div>
                                ))}
                            </Stack>
                        </Card>
                    )}
                </Tabs.Panel>

                <Tabs.Panel value="performance" pt="md">
                    <Alert color="blue" variant="light" mb="md">
                        Performance analysis is based on test execution times and benchmark data
                        across different FHIRPath implementations.
                    </Alert>

                    <SimpleGrid cols={{base: 1, lg: 3}} spacing="md">
                        <Card padding="lg" style={{background: theme.other.surface}}>
                            <Center>
                                <RingProgress
                                    size={120}
                                    thickness={12}
                                    sections={[
                                        {
                                            value: (analysis.crossLanguageCompatibility.fullyCompatible / analysis.totalExpressions) * 100,
                                            color: "green",
                                        },
                                    ]}
                                    label={
                                        <Center>
                                            <div>
                                                <Text size="xs" ta="center" c="dimmed">
                                                    Fully Compatible
                                                </Text>
                                                <Text size="lg" fw={700} ta="center">
                                                    {Math.round((analysis.crossLanguageCompatibility.fullyCompatible / analysis.totalExpressions) * 100)}%
                                                </Text>
                                            </div>
                                        </Center>
                                    }
                                />
                            </Center>
                        </Card>

                        <Card padding="lg" style={{background: theme.other.surface}}>
                            <Text size="sm" fw={500} mb="md">
                                Category Performance Ranking
                            </Text>
                            <Stack gap="xs">
                                <Group justify="space-between">
                                    <Text size="xs">Fastest</Text>
                                    <Badge color="green" size="sm">
                                        {formatCategoryName(analysis.performanceInsights.fastestCategory)}
                                    </Badge>
                                </Group>
                                <Group justify="space-between">
                                    <Text size="xs">Slowest</Text>
                                    <Badge color="red" size="sm">
                                        {formatCategoryName(analysis.performanceInsights.slowestCategory)}
                                    </Badge>
                                </Group>
                                <Group justify="space-between">
                                    <Text size="xs">Most Variable</Text>
                                    <Badge color="orange" size="sm">
                                        {formatCategoryName(analysis.performanceInsights.mostVariableCategory)}
                                    </Badge>
                                </Group>
                            </Stack>
                        </Card>

                        <Card padding="lg" style={{background: theme.other.surface}}>
                            <Text size="sm" fw={500} mb="md">
                                Complexity vs Performance
                            </Text>
                            <Stack gap="xs">
                                {Object.entries(analysis.complexityDistribution).map(([complexity, count]) => (
                                    <Group key={complexity} justify="space-between">
                                        <Text size="xs" tt="capitalize">{complexity}</Text>
                                        <Badge color={COMPLEXITY_COLORS[complexity as ExpressionComplexity]}
                                               variant="light" size="sm">
                                            {count} expressions
                                        </Badge>
                                    </Group>
                                ))}
                            </Stack>
                        </Card>
                    </SimpleGrid>
                </Tabs.Panel>
            </Tabs>
        </Stack>
    );
}

export default ExpressionPatternView;
