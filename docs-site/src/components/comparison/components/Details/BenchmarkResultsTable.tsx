import { useState } from "react";
import { Table, Select, Group, Text } from "@mantine/core";
import { useMantineTheme } from "@mantine/core";
import type { BenchmarkResultSet } from "../../types/comparison";

interface BenchmarkResultsTableProps {
	benchmarkResults: BenchmarkResultSet[];
}

export function BenchmarkResultsTable({
	benchmarkResults,
}: BenchmarkResultsTableProps) {
	const theme = useMantineTheme();
	// Flatten all benchmarks with language info
	const allBenchmarks = benchmarkResults.flatMap((result) =>
		(result.benchmarks || []).map((bench) => ({
			language: result.language,
			...bench,
		})),
	);
	const languages = Array.from(new Set(allBenchmarks.map((b) => b.language)));
	const [selectedLanguage, setSelectedLanguage] = useState<string | null>(null);
	const [sortBy, setSortBy] = useState<"language" | "ops" | "mean" | "stddev">(
		"ops",
	);
	const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");

	// Filter by language if selected
	let filtered = selectedLanguage
		? allBenchmarks.filter((b) => b.language === selectedLanguage)
		: allBenchmarks;

	// Sort
	filtered = [...filtered].sort((a, b) => {
		if (sortBy === "language") {
			return sortDir === "asc"
				? a.language.localeCompare(b.language)
				: b.language.localeCompare(a.language);
		}
		if (sortBy === "ops") {
			return sortDir === "asc"
				? a.ops_per_second - b.ops_per_second
				: b.ops_per_second - a.ops_per_second;
		}
		if (sortBy === "mean") {
			return sortDir === "asc"
				? a.avg_time_ms - b.avg_time_ms
				: b.avg_time_ms - a.avg_time_ms;
		}
		if (sortBy === "stddev") {
			return sortDir === "asc"
				? (a.std_dev ?? 0) - (b.std_dev ?? 0)
				: (b.std_dev ?? 0) - (a.std_dev ?? 0);
		}
		return 0;
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
			<Group mb="md">
				<Select
					label="Filter by Language"
					data={languages.map((l) => ({ value: l, label: l }))}
					value={selectedLanguage}
					onChange={setSelectedLanguage}
					clearable
					placeholder="All languages"
				/>
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
						<Table.Th>Benchmark Name</Table.Th>
						<Table.Th
							onClick={() => {
								setSortBy("ops");
								setSortDir(
									sortBy === "ops" && sortDir === "asc" ? "desc" : "asc",
								);
							}}
							style={{ cursor: "pointer" }}
						>
							Ops/sec {sortBy === "ops" ? (sortDir === "asc" ? "▲" : "▼") : ""}
						</Table.Th>
						<Table.Th
							onClick={() => {
								setSortBy("mean");
								setSortDir(
									sortBy === "mean" && sortDir === "asc" ? "desc" : "asc",
								);
							}}
							style={{ cursor: "pointer" }}
						>
							Mean Time (ms){" "}
							{sortBy === "mean" ? (sortDir === "asc" ? "▲" : "▼") : ""}
						</Table.Th>
						<Table.Th
							onClick={() => {
								setSortBy("stddev");
								setSortDir(
									sortBy === "stddev" && sortDir === "asc" ? "desc" : "asc",
								);
							}}
							style={{ cursor: "pointer" }}
						>
							Stddev (ms){" "}
							{sortBy === "stddev" ? (sortDir === "asc" ? "▲" : "▼") : ""}
						</Table.Th>
					</Table.Tr>
				</Table.Thead>
				<Table.Tbody>
					{filtered.map((bench) => (
						<Table.Tr key={`${bench.language}-${bench.name}`}>
							<Table.Td>{bench.language}</Table.Td>
							<Table.Td>{bench.name}</Table.Td>
							<Table.Td>
								{bench.ops_per_second?.toLocaleString(undefined, {
									maximumFractionDigits: 0,
								}) ?? "-"}
							</Table.Td>
							<Table.Td>{bench.avg_time_ms?.toFixed(2) ?? "-"}</Table.Td>
							<Table.Td>{bench.std_dev?.toFixed(2) ?? "-"}</Table.Td>
						</Table.Tr>
					))}
				</Table.Tbody>
			</Table>
			{filtered.length === 0 && (
				<Text c="dimmed" mt="md" style={{ color: theme.other.textMuted }}>
					No results to display.
				</Text>
			)}
		</div>
	);
}

export default BenchmarkResultsTable;
