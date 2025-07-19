<script>
    import { onMount } from 'svelte';
    import { getAllBenchmarkResults } from './resultLoader.js';

    // State variables
    let loading = true;
    let benchmarkResults = [];
    let error = null;
    let selectedLanguage = 'all';
    let selectedBenchmark = 'all';
    let sortBy = 'avg_time_ms';
    let sortOrder = 'asc';

    // Language metadata
    const languageInfo = {
        javascript: { fullName: 'JavaScript', icon: '🟨', color: '#f7df1e' },
        python: { fullName: 'Python', icon: '🐍', color: '#3776ab' },
        java: { fullName: 'Java', icon: '☕', color: '#ed8b00' },
        csharp: { fullName: 'C#', icon: '🔷', color: '#239120' },
        rust: { fullName: 'Rust', icon: '🦀', color: '#ce422b' },
        go: { fullName: 'Go', icon: '🔵', color: '#00add8' }
    };

    let availableLanguages = [];
    let availableBenchmarks = [];
    let filteredResults = [];

    onMount(async () => {
        try {
            benchmarkResults = await getAllBenchmarkResults();

            // Extract available languages and benchmarks
            availableLanguages = [...new Set(benchmarkResults.map(r => r.language))];

            const allBenchmarks = new Set();
            benchmarkResults.forEach(result => {
                if (result.benchmarks) {
                    result.benchmarks.forEach(benchmark => {
                        allBenchmarks.add(benchmark.name);
                    });
                }
            });
            availableBenchmarks = [...allBenchmarks];

            updateFilteredResults();
            loading = false;
        } catch (e) {
            error = e.message;
            loading = false;
        }
    });

    function updateFilteredResults() {
        let results = [];

        benchmarkResults.forEach(langResult => {
            if (selectedLanguage !== 'all' && langResult.language !== selectedLanguage) {
                return;
            }

            if (langResult.benchmarks) {
                langResult.benchmarks.forEach(benchmark => {
                    if (selectedBenchmark !== 'all' && benchmark.name !== selectedBenchmark) {
                        return;
                    }

                    results.push({
                        ...benchmark,
                        language: langResult.language,
                        timestamp: langResult.timestamp,
                        system_info: langResult.system_info
                    });
                });
            }
        });

        // Sort results
        results.sort((a, b) => {
            let aVal = a[sortBy];
            let bVal = b[sortBy];

            if (typeof aVal === 'string') {
                aVal = aVal.toLowerCase();
                bVal = bVal.toLowerCase();
            }

            if (sortOrder === 'asc') {
                return aVal < bVal ? -1 : aVal > bVal ? 1 : 0;
            } else {
                return aVal > bVal ? -1 : aVal < bVal ? 1 : 0;
            }
        });

        filteredResults = results;
    }

    function handleLanguageChange() {
        updateFilteredResults();
    }

    function handleBenchmarkChange() {
        updateFilteredResults();
    }

    function handleSort(column) {
        if (sortBy === column) {
            sortOrder = sortOrder === 'asc' ? 'desc' : 'asc';
        } else {
            sortBy = column;
            sortOrder = 'asc';
        }
        updateFilteredResults();
    }

    function formatTime(ms) {
        if (ms < 1) {
            return `${(ms * 1000).toFixed(2)}μs`;
        } else if (ms < 1000) {
            return `${ms.toFixed(2)}ms`;
        } else {
            return `${(ms / 1000).toFixed(2)}s`;
        }
    }

    function formatOps(ops) {
        if (ops > 1000000) {
            return `${(ops / 1000000).toFixed(2)}M ops/s`;
        } else if (ops > 1000) {
            return `${(ops / 1000).toFixed(2)}K ops/s`;
        } else {
            return `${ops.toFixed(2)} ops/s`;
        }
    }

    function getPerformanceClass(value, min, max) {
        const range = max - min;
        const normalized = (value - min) / range;

        if (normalized <= 0.33) return 'performance-good';
        if (normalized <= 0.66) return 'performance-medium';
        return 'performance-poor';
    }

    $: {
        if (selectedLanguage || selectedBenchmark || sortBy || sortOrder) {
            updateFilteredResults();
        }
    }
</script>

<div class="benchmark-details">
    {#if loading}
        <div class="loading">
            <div class="spinner"></div>
            <p>Loading benchmark data...</p>
        </div>
    {:else if error}
        <div class="error">
            <h3>Error Loading Data</h3>
            <p>{error}</p>
        </div>
    {:else}
        <!-- Filters -->
        <div class="filters">
            <div class="filter-group">
                <label for="language-select">Language:</label>
                <select id="language-select" bind:value={selectedLanguage} on:change={handleLanguageChange}>
                    <option value="all">All Languages</option>
                    {#each availableLanguages as lang}
                        <option value={lang}>
                            {languageInfo[lang]?.fullName || lang}
                        </option>
                    {/each}
                </select>
            </div>

            <div class="filter-group">
                <label for="benchmark-select">Benchmark:</label>
                <select id="benchmark-select" bind:value={selectedBenchmark} on:change={handleBenchmarkChange}>
                    <option value="all">All Benchmarks</option>
                    {#each availableBenchmarks as benchmark}
                        <option value={benchmark}>{benchmark}</option>
                    {/each}
                </select>
            </div>
        </div>

        <!-- Results Summary -->
        <div class="summary">
            <div class="summary-card">
                <h3>Total Results</h3>
                <div class="summary-value">{filteredResults.length}</div>
            </div>
            <div class="summary-card">
                <h3>Languages</h3>
                <div class="summary-value">
                    {selectedLanguage === 'all' ? availableLanguages.length : 1}
                </div>
            </div>
            <div class="summary-card">
                <h3>Benchmarks</h3>
                <div class="summary-value">
                    {selectedBenchmark === 'all' ? availableBenchmarks.length : 1}
                </div>
            </div>
        </div>

        <!-- Results Table -->
        <div class="results-table">
            <table>
                <thead>
                    <tr>
                        <th on:click={() => handleSort('language')} class="sortable">
                            Language
                            {#if sortBy === 'language'}
                                <span class="sort-indicator">{sortOrder === 'asc' ? '↑' : '↓'}</span>
                            {/if}
                        </th>
                        <th on:click={() => handleSort('name')} class="sortable">
                            Benchmark
                            {#if sortBy === 'name'}
                                <span class="sort-indicator">{sortOrder === 'asc' ? '↑' : '↓'}</span>
                            {/if}
                        </th>
                        <th on:click={() => handleSort('avg_time_ms')} class="sortable">
                            Avg Time
                            {#if sortBy === 'avg_time_ms'}
                                <span class="sort-indicator">{sortOrder === 'asc' ? '↑' : '↓'}</span>
                            {/if}
                        </th>
                        <th on:click={() => handleSort('min_time_ms')} class="sortable">
                            Min Time
                            {#if sortBy === 'min_time_ms'}
                                <span class="sort-indicator">{sortOrder === 'asc' ? '↑' : '↓'}</span>
                            {/if}
                        </th>
                        <th on:click={() => handleSort('max_time_ms')} class="sortable">
                            Max Time
                            {#if sortBy === 'max_time_ms'}
                                <span class="sort-indicator">{sortOrder === 'asc' ? '↑' : '↓'}</span>
                            {/if}
                        </th>
                        <th on:click={() => handleSort('ops_per_second')} class="sortable">
                            Ops/Second
                            {#if sortBy === 'ops_per_second'}
                                <span class="sort-indicator">{sortOrder === 'asc' ? '↑' : '↓'}</span>
                            {/if}
                        </th>
                        <th>Expression</th>
                    </tr>
                </thead>
                <tbody>
                    {#each filteredResults as result}
                        <tr>
                            <td>
                                <div class="language-cell">
                                    <span class="language-icon">{languageInfo[result.language]?.icon || '📄'}</span>
                                    <span class="language-name">{languageInfo[result.language]?.fullName || result.language}</span>
                                </div>
                            </td>
                            <td>
                                <div class="benchmark-name">{result.name}</div>
                                <div class="benchmark-description">{result.description}</div>
                            </td>
                            <td class="time-cell">
                                <span class="time-value">{formatTime(result.avg_time_ms)}</span>
                            </td>
                            <td class="time-cell">
                                <span class="time-value">{formatTime(result.min_time_ms)}</span>
                            </td>
                            <td class="time-cell">
                                <span class="time-value">{formatTime(result.max_time_ms)}</span>
                            </td>
                            <td class="ops-cell">
                                <span class="ops-value">{formatOps(result.ops_per_second)}</span>
                            </td>
                            <td class="expression-cell">
                                <code class="expression">{result.expression}</code>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>

<style>
    .benchmark-details {
        width: 100%;
    }

    .loading {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 48px;
        color: #6b7280;
    }

    .spinner {
        width: 32px;
        height: 32px;
        border: 3px solid #e5e7eb;
        border-top: 3px solid #3b82f6;
        border-radius: 50%;
        animation: spin 1s linear infinite;
        margin-bottom: 16px;
    }

    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }

    .error {
        padding: 24px;
        background: #fef2f2;
        border: 1px solid #fecaca;
        border-radius: 6px;
        color: #dc2626;
    }

    .filters {
        display: flex;
        gap: 16px;
        margin-bottom: 24px;
        flex-wrap: wrap;
    }

    .filter-group {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .filter-group label {
        font-size: 0.875rem;
        font-weight: 500;
        color: #374151;
    }

    .filter-group select {
        padding: 6px 8px;
        border: 1px solid #d1d5db;
        border-radius: 4px;
        background: white;
        font-size: 0.875rem;
        min-width: 150px;
    }

    .summary {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
        gap: 16px;
        margin-bottom: 24px;
    }

    .summary-card {
        background: #f9fafb;
        border: 1px solid #e5e7eb;
        border-radius: 6px;
        padding: 16px;
        text-align: center;
    }

    .summary-card h3 {
        font-size: 0.875rem;
        font-weight: 500;
        color: #6b7280;
        margin-bottom: 8px;
    }

    .summary-value {
        font-size: 1.5rem;
        font-weight: 600;
        color: #1f2937;
    }

    .results-table {
        overflow-x: auto;
        border: 1px solid #e5e7eb;
        border-radius: 6px;
    }

    table {
        width: 100%;
        border-collapse: collapse;
        background: white;
    }

    th, td {
        padding: 8px 12px;
        text-align: left;
        border-bottom: 1px solid #e5e7eb;
    }

    th {
        background: #f9fafb;
        font-weight: 600;
        font-size: 0.875rem;
        color: #374151;
    }

    th.sortable {
        cursor: pointer;
        user-select: none;
    }

    th.sortable:hover {
        background: #f3f4f6;
    }

    .sort-indicator {
        margin-left: 4px;
        color: #3b82f6;
    }

    .language-cell {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .language-icon {
        font-size: 1.2rem;
    }

    .language-name {
        font-weight: 500;
    }

    .benchmark-name {
        font-weight: 500;
        margin-bottom: 2px;
    }

    .benchmark-description {
        font-size: 0.75rem;
        color: #6b7280;
    }

    .time-cell, .ops-cell {
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        font-size: 0.875rem;
    }

    .expression-cell {
        max-width: 300px;
    }

    .expression {
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        font-size: 0.75rem;
        background: #f3f4f6;
        padding: 2px 4px;
        border-radius: 3px;
        word-break: break-all;
        display: block;
        max-height: 60px;
        overflow-y: auto;
    }

    @media (max-width: 768px) {
        .filters {
            flex-direction: column;
        }

        .filter-group select {
            min-width: 100%;
        }

        .summary {
            grid-template-columns: 1fr;
        }

        th, td {
            padding: 6px 8px;
            font-size: 0.8rem;
        }

        .expression {
            font-size: 0.7rem;
        }
    }
</style>
