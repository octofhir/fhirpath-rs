import { useState } from "react";
import { Table, Select, Group, Text, Button, Badge, ActionIcon, Tooltip } from "@mantine/core";
import { IconAlertCircle, IconEye } from "@tabler/icons-react";
import { useMantineTheme } from "@mantine/core";
import type { TestResultSet } from "../../types/comparison";
import { FailedTestDetails } from "./FailedTestDetails";

interface TestResultsTableProps {
	testResults: TestResultSet[];
}

export function TestResultsTable({ testResults }: TestResultsTableProps) {
	const theme = useMantineTheme();
	// Gather all languages
	const languages = Array.from(new Set(testResults.map((r) => r.language)));
	const [selectedLanguage, setSelectedLanguage] = useState<string | null>(null);
	const [sortBy, setSortBy] = useState<"language" | "passRate" | "total">(
		"passRate",
	);
	const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");
	const [failedTestsModalOpened, setFailedTestsModalOpened] = useState(false);

	// Calculate total failed tests across all languages
	const totalFailedTests = testResults.reduce((total, resultSet) => {
		const failed = resultSet.summary && resultSet.summary.total != null && resultSet.summary.passed != null
			? resultSet.summary.total - resultSet.summary.passed
			: 0;
		return total + failed;
	}, 0);

	// Filter by language if selected
	let filtered = selectedLanguage
		? testResults.filter((r) => r.language === selectedLanguage)
		: testResults;

	// Sort
	filtered = [...filtered].sort((a, b) => {
		if (sortBy === "language") {
			return sortDir === "asc"
				? a.language.localeCompare(b.language)
				: b.language.localeCompare(a.language);
		}
		if (sortBy === "total") {
			return sortDir === "asc"
				? (a.summary?.total ?? 0) - (b.summary?.total ?? 0)
				: (b.summary?.total ?? 0) - (a.summary?.total ?? 0);
		}
		// passRate
		const aRate =
			a.summary && a.summary.total ? a.summary.passed / a.summary.total : 0;
		const bRate =
			b.summary && b.summary.total ? b.summary.passed / b.summary.total : 0;
		return sortDir === "asc" ? aRate - bRate : bRate - aRate;
	});

	return (
		<div
			style={{
				background: theme.other.surface,
				color: theme.other.text,
				borderRadius: theme.radius.md,
				padding: theme.spacing.md,
			}}
		>
			<Group mb="md" justify="space-between">
				<Select
					label="Filter by Language"
					data={languages.map((l) => ({ value: l, label: l }))}
					value={selectedLanguage}
					onChange={setSelectedLanguage}
					clearable
					placeholder="All languages"
				/>
				{totalFailedTests > 0 && (
					<Button
						leftSection={<IconAlertCircle size="1rem" />}
						variant="light"
						color="red"
						onClick={() => setFailedTestsModalOpened(true)}
					>
						View {totalFailedTests} Failed Tests
					</Button>
				)}
			</Group>
			<Table
				striped
				highlightOnHover
				withTableBorder
				withColumnBorders
				style={{
					background: theme.other.surfaceSecondary,
					color: theme.other.text,
					borderColor: theme.other.border,
				}}
			>
				<Table.Thead>
					<Table.Tr>
						<Table.Th
							onClick={() => {
								setSortBy("language");
								setSortDir(
									sortBy === "language" && sortDir === "asc" ? "desc" : "asc",
								);
							}}
							style={{ cursor: "pointer" }}
						>
							Language{" "}
							{sortBy === "language" ? (sortDir === "asc" ? "▲" : "▼") : ""}
						</Table.Th>
						<Table.Th>Test Name</Table.Th>
						<Table.Th>Passed</Table.Th>
						<Table.Th>Failed</Table.Th>
						<Table.Th
							onClick={() => {
								setSortBy("total");
								setSortDir(
									sortBy === "total" && sortDir === "asc" ? "desc" : "asc",
								);
							}}
							style={{ cursor: "pointer" }}
						>
							Total {sortBy === "total" ? (sortDir === "asc" ? "▲" : "▼") : ""}
						</Table.Th>
						<Table.Th
							onClick={() => {
								setSortBy("passRate");
								setSortDir(
									sortBy === "passRate" && sortDir === "asc" ? "desc" : "asc",
								);
							}}
							style={{ cursor: "pointer" }}
						>
							Pass Rate (%){" "}
							{sortBy === "passRate" ? (sortDir === "asc" ? "▲" : "▼") : ""}
						</Table.Th>
					</Table.Tr>
				</Table.Thead>
				<Table.Tbody>
					{filtered.map((result) => (
						<Table.Tr key={result.language}>
							<Table.Td>{result.language}</Table.Td>
							<Table.Td>{result.summary?.testSuiteName ?? "-"}</Table.Td>
							<Table.Td>
								<Badge color="green" variant="light" size="sm">
									{result.summary?.passed ?? "-"}
								</Badge>
							</Table.Td>
							<Table.Td>
								{result.summary &&
								result.summary.total != null &&
								result.summary.passed != null ? (
									<Group gap="xs">
										<Badge
											color="red"
											variant="light"
											size="sm"
										>
											{result.summary.total - result.summary.passed}
										</Badge>
										{result.summary.total - result.summary.passed > 0 && (
											<Tooltip label="View failed tests for this language">
												<ActionIcon
													size="sm"
													variant="subtle"
													color="red"
													onClick={() => setFailedTestsModalOpened(true)}
												>
													<IconEye size="0.8rem" />
												</ActionIcon>
											</Tooltip>
										)}
									</Group>
								) : (
									"-"
								)}
							</Table.Td>
							<Table.Td>{result.summary?.total ?? "-"}</Table.Td>
							<Table.Td>
								{result.summary && result.summary.total
									? (
											(result.summary.passed / result.summary.total) *
											100
										).toFixed(1)
									: "-"}
							</Table.Td>
						</Table.Tr>
					))}
				</Table.Tbody>
			</Table>
			{filtered.length === 0 && (
				<Text c="dimmed" mt="md" style={{ color: theme.other.textMuted }}>
					No results to display.
				</Text>
			)}

			{/* Failed Test Details Modal */}
			<FailedTestDetails
				testResults={testResults}
				opened={failedTestsModalOpened}
				onClose={() => setFailedTestsModalOpened(false)}
			/>
		</div>
	);
}

export default TestResultsTable;
