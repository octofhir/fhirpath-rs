import { useState } from "react";
import {
	Modal,
	Table,
	Text,
	Badge,
	Group,
	Stack,
	Code,
	Alert,
	Accordion,
	Button,
	TextInput,
	Select,
	Pagination,
	ActionIcon,
	Tooltip,
} from "@mantine/core";
import {
	IconAlertCircle,
	IconClock,
	IconCode,
	IconX,
	IconSearch,
	IconFilter,
	IconDownload,
} from "@tabler/icons-react";
import { useMantineTheme } from "@mantine/core";
import type { TestResult, TestResultSet } from "../../types/comparison";

interface FailedTestDetailsProps {
	testResults: TestResultSet[];
	opened: boolean;
	onClose: () => void;
}

interface FailedTest extends TestResult {
	language: string;
}

const ITEMS_PER_PAGE = 10;

export function FailedTestDetails({
	testResults,
	opened,
	onClose,
}: FailedTestDetailsProps) {
	const theme = useMantineTheme();
	const [searchTerm, setSearchTerm] = useState("");
	const [selectedLanguage, setSelectedLanguage] = useState<string | null>(null);
	const [currentPage, setCurrentPage] = useState(1);

	// Extract all failed tests from all languages
	const allFailedTests: FailedTest[] = testResults.flatMap((resultSet) =>
		(resultSet.tests || [])
			.filter((test) => test.status === "failed" || test.status === "error")
			.map((test) => ({
				...test,
				language: resultSet.language,
			})),
	);

	// Get unique languages for filtering
	const languages = Array.from(
		new Set(allFailedTests.map((test) => test.language)),
	);

	// Filter tests based on search and language
	const filteredTests = allFailedTests.filter((test) => {
		const matchesSearch =
			!searchTerm ||
			test.name?.toLowerCase().includes(searchTerm.toLowerCase()) ||
			test.expression?.toLowerCase().includes(searchTerm.toLowerCase()) ||
			test.error?.toLowerCase().includes(searchTerm.toLowerCase()) ||
			test.description?.toLowerCase().includes(searchTerm.toLowerCase());

		const matchesLanguage = !selectedLanguage || test.language === selectedLanguage;

		return matchesSearch && matchesLanguage;
	});

	// Pagination
	const totalPages = Math.ceil(filteredTests.length / ITEMS_PER_PAGE);
	const startIndex = (currentPage - 1) * ITEMS_PER_PAGE;
	const paginatedTests = filteredTests.slice(
		startIndex,
		startIndex + ITEMS_PER_PAGE,
	);

	// Export failed tests as JSON
	const exportFailedTests = () => {
		const exportData = {
			exported_at: new Date().toISOString(),
			total_failed_tests: filteredTests.length,
			filters: {
				search: searchTerm || null,
				language: selectedLanguage || null,
			},
			failed_tests: filteredTests,
		};

		const blob = new Blob([JSON.stringify(exportData, null, 2)], {
			type: "application/json",
		});
		const url = URL.createObjectURL(blob);
		const a = document.createElement("a");
		a.href = url;
		a.download = `failed-tests-${new Date().toISOString().split("T")[0]}.json`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	};

	// Format expected/actual values for display
	const formatValue = (value: any): string => {
		if (value === null) return "null";
		if (value === undefined) return "undefined";
		if (Array.isArray(value)) {
			if (value.length === 0) return "[]";
			return JSON.stringify(value, null, 2);
		}
		if (typeof value === "object") {
			return JSON.stringify(value, null, 2);
		}
		return String(value);
	};

	return (
		<Modal
			opened={opened}
			onClose={onClose}
			title={
				<Group>
					<IconAlertCircle size="1.2rem" color={theme.colors.red[6]} />
					<Text fw={600}>Failed Test Details</Text>
					<Badge color="red" variant="light">
						{filteredTests.length} failed tests
					</Badge>
				</Group>
			}
			size="xl"
			styles={{
				modal: {
					background: theme.other.surface,
					color: theme.other.text,
				},
				header: {
					background: theme.other.surface,
					borderBottom: `1px solid ${theme.other.border}`,
				},
				body: {
					background: theme.other.surface,
				},
			}}
		>
			<Stack gap="md">
				{/* Filters and Search */}
				<Group>
					<TextInput
						placeholder="Search tests, expressions, or errors..."
						value={searchTerm}
						onChange={(e) => {
							setSearchTerm(e.target.value);
							setCurrentPage(1);
						}}
						leftSection={<IconSearch size="1rem" />}
						style={{ flex: 1 }}
					/>
					<Select
						placeholder="Filter by language"
						data={languages.map((lang) => ({ value: lang, label: lang }))}
						value={selectedLanguage}
						onChange={(value) => {
							setSelectedLanguage(value);
							setCurrentPage(1);
						}}
						clearable
						leftSection={<IconFilter size="1rem" />}
						w={200}
					/>
					<Tooltip label="Export failed tests">
						<ActionIcon
							onClick={exportFailedTests}
							variant="light"
							color="blue"
							size="lg"
						>
							<IconDownload size="1rem" />
						</ActionIcon>
					</Tooltip>
				</Group>

				{/* Results Summary */}
				{filteredTests.length > 0 && (
					<Alert
						icon={<IconAlertCircle size="1rem" />}
						color="orange"
						variant="light"
					>
						Showing {paginatedTests.length} of {filteredTests.length} failed tests
						{searchTerm && ` matching "${searchTerm}"`}
						{selectedLanguage && ` in ${selectedLanguage}`}
					</Alert>
				)}

				{/* Failed Tests List */}
				{paginatedTests.length > 0 ? (
					<Accordion
						variant="separated"
						styles={{
							item: {
								background: theme.other.surfaceSecondary,
								border: `1px solid ${theme.other.border}`,
							},
							control: {
								background: theme.other.surfaceSecondary,
								"&:hover": {
									background: theme.other.surface,
								},
							},
							content: {
								background: theme.other.surface,
							},
						}}
					>
						{paginatedTests.map((test, index) => (
							<Accordion.Item
								key={`${test.language}-${test.name}-${index}`}
								value={`${test.language}-${test.name}-${index}`}
							>
								<Accordion.Control>
									<Group justify="space-between" w="100%">
										<Group>
											<Badge color="red" variant="light">
												{test.language}
											</Badge>
											<Text fw={500}>{test.name || "Unnamed Test"}</Text>
											{test.status === "error" && (
												<Badge color="orange" variant="light">
													ERROR
												</Badge>
											)}
										</Group>
										<Group gap="xs">
											{test.execution_time_ms && (
												<Group gap={4}>
													<IconClock size="0.8rem" />
													<Text size="xs" c="dimmed">
														{test.execution_time_ms.toFixed(2)}ms
													</Text>
												</Group>
											)}
										</Group>
									</Group>
								</Accordion.Control>
								<Accordion.Panel>
									<Stack gap="md">
										{/* Test Description */}
										{test.description && (
											<div>
												<Text size="sm" fw={500} mb={4}>
													Description:
												</Text>
												<Text size="sm" c="dimmed">
													{test.description}
												</Text>
											</div>
										)}

										{/* FHIRPath Expression */}
										{test.expression && (
											<div>
												<Text size="sm" fw={500} mb={4}>
													FHIRPath Expression:
												</Text>
												<Code
													block
													style={{
														background: theme.other.surfaceSecondary,
														color: theme.other.text,
														border: `1px solid ${theme.other.border}`,
													}}
												>
													{test.expression}
												</Code>
											</div>
										)}

										{/* Error Message */}
										{test.error && (
											<Alert
												icon={<IconAlertCircle size="1rem" />}
												color="red"
												variant="light"
											>
												<Text size="sm" fw={500} mb={4}>
													Error:
												</Text>
												<Text size="sm">{test.error}</Text>
											</Alert>
										)}

										{/* Expected vs Actual */}
										{(test.expected !== undefined || test.actual !== undefined) && (
											<Group align="flex-start" grow>
												{test.expected !== undefined && (
													<div>
														<Text size="sm" fw={500} mb={4} c="green">
															Expected:
														</Text>
														<Code
															block
															style={{
																background: theme.other.surfaceSecondary,
																color: theme.other.text,
																border: `1px solid ${theme.colors.green[3]}`,
																maxHeight: "200px",
																overflow: "auto",
															}}
														>
															{formatValue(test.expected)}
														</Code>
													</div>
												)}
												{test.actual !== undefined && (
													<div>
														<Text size="sm" fw={500} mb={4} c="red">
															Actual:
														</Text>
														<Code
															block
															style={{
																background: theme.other.surfaceSecondary,
																color: theme.other.text,
																border: `1px solid ${theme.colors.red[3]}`,
																maxHeight: "200px",
																overflow: "auto",
															}}
														>
															{formatValue(test.actual)}
														</Code>
													</div>
												)}
											</Group>
										)}
									</Stack>
								</Accordion.Panel>
							</Accordion.Item>
						))}
					</Accordion>
				) : (
					<Alert color="blue" variant="light">
						{searchTerm || selectedLanguage
							? "No failed tests match your current filters."
							: "No failed tests found in the current results."}
					</Alert>
				)}

				{/* Pagination */}
				{totalPages > 1 && (
					<Group justify="center">
						<Pagination
							value={currentPage}
							onChange={setCurrentPage}
							total={totalPages}
							size="sm"
						/>
					</Group>
				)}
			</Stack>
		</Modal>
	);
}

export default FailedTestDetails;
