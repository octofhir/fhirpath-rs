<script>
    import { onMount } from 'svelte';
    import TestResultsChart from './TestResultsChart.svelte';
    import BenchmarkChart from './BenchmarkChart.svelte';
    import { getAllTestResults, getAllBenchmarkResults, createSampleData } from './resultLoader.js';

    // State variables
    let loading = true;
    let testResults = [];
    let benchmarkResults = [];
    let error = null;
    let activeTab = 'overview';
    let selectedLanguage = null;
    let showErrorModal = false;
    let errorModalData = null;

    // Language metadata
    const languageInfo = {
        javascript: {
            fullName: 'JavaScript',
            description: 'Node.js implementation using the fhirpath.js library',
            repository: 'https://github.com/HL7/fhirpath.js',
            license: 'BSD-3-Clause',
            maintainer: 'HL7 FHIR Community',
            icon: '🟨',
            libraries: ['fhirpath.js', 'antlr4-javascript-runtime'],
            runtime: 'Node.js'
        },
        python: {
            fullName: 'Python',
            description: 'Python implementation using the fhirpath_py library',
            repository: 'https://github.com/beda-software/fhirpath-py',
            license: 'MIT',
            maintainer: 'Beda Software',
            icon: '🐍',
            libraries: ['fhirpath-py', 'antlr4-python3-runtime'],
            runtime: 'Python 3'
        },
        java: {
            fullName: 'Java',
            description: 'Java implementation using the HAPI FHIR library',
            repository: 'https://github.com/hapifhir/org.hl7.fhir.core',
            license: 'Apache-2.0',
            maintainer: 'HAPI FHIR',
            icon: '☕',
            libraries: ['org.hl7.fhir.core', 'ANTLR4'],
            runtime: 'JVM'
        },
        csharp: {
            fullName: 'C#',
            description: 'C# implementation using the Firely SDK',
            repository: 'https://github.com/FirelyTeam/firely-net-sdk',
            license: 'BSD-3-Clause',
            maintainer: 'Firely',
            icon: '🔷',
            libraries: ['Hl7.Fhir.Core', 'Hl7.FhirPath'],
            runtime: '.NET'
        },
        rust: {
            fullName: 'Rust',
            description: 'Rust implementation using OctoFHIR FHIRPath',
            repository: 'https://github.com/octofhir/fhirpath-rs',
            license: 'Apache-2.0',
            maintainer: 'OctoFHIR Team',
            icon: '🦀',
            libraries: ['fhirpath-core', 'serde_json'],
            runtime: 'Native'
        },
        go: {
            fullName: 'Go',
            description: 'Go implementation',
            repository: 'https://github.com/healthiop/hipath',
            license: 'Apache-2.0',
            maintainer: 'HealthIOP',
            icon: '🔵',
            libraries: ['hipath', 'go-fhir'],
            runtime: 'Native'
        }
    };

    onMount(async () => {
        try {
            // Load test and benchmark results
            testResults = await getAllTestResults();
            benchmarkResults = await getAllBenchmarkResults();

            // If no data was loaded, use sample data for demonstration
            if (testResults.length === 0 && benchmarkResults.length === 0) {
                console.log('No data loaded, using sample data');
                const sampleData = createSampleData();
                testResults = sampleData.testResults;
                benchmarkResults = sampleData.benchmarkResults;
            }

            // Extract version information from benchmark results
            benchmarkResults.forEach(result => {
                if (result.system_info && result.language) {
                    const lang = languageInfo[result.language];
                    if (lang) {
                        lang.version = result.system_info.fhirpath_version || 'Unknown';
                        lang.platform = result.system_info.platform || 'Unknown';
                        lang.runtime_version = result.system_info.node_version ||
                                              result.system_info.python_version ||
                                              result.system_info.java_version ||
                                              result.system_info.dotnet_version ||
                                              result.system_info.rust_version ||
                                              result.system_info.go_version || 'Unknown';
                    }
                }
            });

            console.log(`Loaded ${testResults.length} test result sets and ${benchmarkResults.length} benchmark result sets`);
        } catch (e) {
            console.error('Error loading data:', e);
            error = e.message;

            // Use sample data on error
            const sampleData = createSampleData();
            testResults = sampleData.testResults;
            benchmarkResults = sampleData.benchmarkResults;
        } finally {
            loading = false;
        }
    });

    // Calculate performance metrics
    $: totalTests = testResults.reduce((sum, result) => sum + (result.summary?.total || 0), 0);
    $: totalPassed = testResults.reduce((sum, result) => sum + (result.summary?.passed || 0), 0);
    $: successRate = totalTests > 0 ? ((totalPassed / totalTests) * 100).toFixed(1) : 0;
    $: avgExecutionTime = calculateAverageExecutionTime();
    $: totalBenchmarks = benchmarkResults.reduce((sum, result) => sum + (result.benchmarks?.length || 0), 0);
    $: fastestImplementation = getFastestImplementation();
    $: mostCompliantImplementation = getMostCompliantImplementation();

    function calculateAverageExecutionTime() {
        let totalTime = 0;
        let testCount = 0;

        testResults.forEach(result => {
            if (result.tests) {
                result.tests.forEach(test => {
                    totalTime += test.execution_time_ms || 0;
                    testCount++;
                });
            }
        });

        return testCount > 0 ? totalTime / testCount : 0;
    }

    function getFastestImplementation() {
        if (benchmarkResults.length === 0) return null;

        const langPerformance = benchmarkResults.map(result => {
            if (!result.benchmarks || result.benchmarks.length === 0) return { language: result.language, avgOps: 0 };

            const avgOps = result.benchmarks.reduce((sum, b) => sum + (b.ops_per_second || 0), 0) / result.benchmarks.length;
            return { language: result.language, avgOps };
        });

        return langPerformance.reduce((fastest, current) =>
            current.avgOps > fastest.avgOps ? current : fastest, { language: '', avgOps: 0 });
    }

    function getMostCompliantImplementation() {
        if (testResults.length === 0) return null;

        const langCompliance = testResults.map(result => {
            const passRate = result.summary ? (result.summary.passed / result.summary.total) * 100 : 0;
            return { language: result.language, passRate };
        });

        return langCompliance.reduce((mostCompliant, current) =>
            current.passRate > mostCompliant.passRate ? current : mostCompliant, { language: '', passRate: 0 });
    }

    function setActiveTab(tab) {
        activeTab = tab;
    }

    function showLanguageDetails(language) {
        selectedLanguage = language;
        activeTab = 'details';
    }

    function closeLanguageDetails() {
        selectedLanguage = null;
    }

    function showErrorDetails(testResult) {
        errorModalData = testResult;
        showErrorModal = true;
    }

    function closeErrorModal() {
        showErrorModal = false;
        errorModalData = null;
    }
</script>

<main>
    {#if loading}
        <div class="loading">
            <div class="spinner"></div>
            <p>Loading comparison results...</p>
        </div>
    {:else if error}
        <div class="error">
            <h2>Error Loading Data</h2>
            <p>{error}</p>
            <p>Showing sample data instead.</p>
        </div>
        <div class="app-container">
            <!-- Sample data UI -->
            <div class="metrics-card">
                <h2>Performance Metrics</h2>
                <div class="metrics-grid">
                    <div class="metric">
                        <span class="metric-label">Total Tests</span>
                        <span class="metric-value">{totalTests}</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Success Rate</span>
                        <span class="metric-value">{successRate}%</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Avg Execution Time</span>
                        <span class="metric-value">{avgExecutionTime.toFixed(2)}ms</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Total Benchmarks</span>
                        <span class="metric-value">{totalBenchmarks}</span>
                    </div>
                </div>
            </div>

            <div class="chart-card">
                <h3>Test Results by Language Implementation</h3>
                <TestResultsChart testResults={testResults} />
            </div>

            <div class="chart-card">
                <h3>Benchmark Performance by Language Implementation</h3>
                <BenchmarkChart benchmarkResults={benchmarkResults} />
            </div>

            <div class="summary-card">
                <h3>Summary</h3>
                <p>Loaded {testResults.length} test result sets and {benchmarkResults.length} benchmark result sets</p>
                <p class="note">Note: This is sample data for demonstration purposes.</p>
            </div>
        </div>
    {:else}
        <!-- Navigation tabs -->
        <div class="tabs-container">
            <div class="tabs">
                <button class="tab-button {activeTab === 'overview' ? 'active' : ''}" on:click={() => setActiveTab('overview')}>
                    <span class="tab-icon">📊</span> Overview
                </button>
                <button class="tab-button {activeTab === 'implementations' ? 'active' : ''}" on:click={() => setActiveTab('implementations')}>
                    <span class="tab-icon">🧩</span> Implementations
                </button>
                <button class="tab-button {activeTab === 'details' ? 'active' : ''}" on:click={() => setActiveTab('details')}>
                    <span class="tab-icon">📝</span> Detailed Results
                </button>
            </div>
        </div>

        <div class="app-container">
            {#if activeTab === 'overview'}
                <!-- Overview Tab -->
                <div class="hero-section">
                    <div class="hero-content">
                        <h2 class="hero-title">FHIRPath Implementation Comparison</h2>
                        <p class="hero-subtitle">Analysis of {testResults.length} implementations across multiple languages</p>

                        {#if fastestImplementation}
                            <div class="hero-badge">
                                <span class="badge-icon">🚀</span>
                                <span class="badge-text">Fastest Implementation: <strong>{languageInfo[fastestImplementation.language]?.fullName || fastestImplementation.language}</strong> ({fastestImplementation.avgOps.toFixed(0)} ops/sec)</span>
                            </div>
                        {/if}

                        {#if mostCompliantImplementation}
                            <div class="hero-badge">
                                <span class="badge-icon">✅</span>
                                <span class="badge-text">Most Compliant: <strong>{languageInfo[mostCompliantImplementation.language]?.fullName || mostCompliantImplementation.language}</strong> ({mostCompliantImplementation.passRate.toFixed(1)}%)</span>
                            </div>
                        {/if}
                    </div>
                </div>

                <div class="metrics-card">
                    <h2>Performance Metrics</h2>
                    <div class="metrics-grid">
                        <div class="metric">
                            <span class="metric-icon">🧪</span>
                            <span class="metric-label">Total Tests</span>
                            <span class="metric-value">{totalTests}</span>
                        </div>
                        <div class="metric">
                            <span class="metric-icon">📈</span>
                            <span class="metric-label">Success Rate</span>
                            <span class="metric-value">{successRate}%</span>
                        </div>
                        <div class="metric">
                            <span class="metric-icon">⏱️</span>
                            <span class="metric-label">Avg Execution Time</span>
                            <span class="metric-value">{avgExecutionTime.toFixed(2)}ms</span>
                        </div>
                        <div class="metric">
                            <span class="metric-icon">🏁</span>
                            <span class="metric-label">Total Benchmarks</span>
                            <span class="metric-value">{totalBenchmarks}</span>
                        </div>
                    </div>
                </div>

                <div class="chart-card">
                    <h3>Test Results by Language Implementation</h3>
                    <TestResultsChart testResults={testResults} />
                </div>

                <div class="chart-card">
                    <h3>Benchmark Performance by Language Implementation</h3>
                    <BenchmarkChart benchmarkResults={benchmarkResults} />
                </div>
            {/if}

            {#if activeTab === 'implementations'}
                <!-- Implementations Tab -->
                <div class="implementations-grid">
                    {#each Object.entries(languageInfo) as [langKey, lang]}
                        <div class="implementation-card" on:click={() => showLanguageDetails(langKey)}>
                            <div class="implementation-header">
                                <span class="implementation-icon">{lang.icon}</span>
                                <h3 class="implementation-name">{lang.fullName}</h3>
                            </div>
                            <div class="implementation-body">
                                <p class="implementation-description">{lang.description}</p>
                                <div class="implementation-details">
                                    <div class="detail-item">
                                        <span class="detail-label">Version:</span>
                                        <span class="detail-value">{lang.version || 'Unknown'}</span>
                                    </div>
                                    <div class="detail-item">
                                        <span class="detail-label">Runtime:</span>
                                        <span class="detail-value">{lang.runtime}</span>
                                    </div>
                                    <div class="detail-item">
                                        <span class="detail-label">License:</span>
                                        <span class="detail-value">{lang.license}</span>
                                    </div>
                                </div>
                            </div>
                            <div class="implementation-footer">
                                <button class="view-details-btn" on:click|stopPropagation={() => showLanguageDetails(langKey)}>View Details</button>
                            </div>
                        </div>
                    {/each}
                </div>
            {/if}

            {#if activeTab === 'details'}
                <!-- Detailed Results Tab -->
                <div class="details-container">
                    <div class="details-sidebar">
                        <h3>Select Implementation</h3>
                        <div class="language-list">
                            {#each Object.entries(languageInfo) as [langKey, lang]}
                                <button
                                    class="language-button {selectedLanguage === langKey ? 'active' : ''}"
                                    on:click={() => showLanguageDetails(langKey)}
                                >
                                    <span class="language-icon">{lang.icon}</span>
                                    <span class="language-name">{lang.fullName}</span>
                                </button>
                            {/each}
                        </div>
                    </div>

                    <div class="details-content">
                        {#if selectedLanguage && languageInfo[selectedLanguage]}
                            {@const lang = languageInfo[selectedLanguage]}
                            {@const testResult = testResults.find(r => r.language === selectedLanguage)}
                            {@const benchmarkResult = benchmarkResults.find(r => r.language === selectedLanguage)}

                            <div class="language-detail-header">
                                <div class="language-detail-title">
                                    <span class="language-detail-icon">{lang.icon}</span>
                                    <h2>{lang.fullName} Implementation</h2>
                                </div>
                                <a href={lang.repository} target="_blank" rel="noopener noreferrer" class="repo-link">
                                    <span class="repo-icon">📦</span> Repository
                                </a>
                            </div>

                            <div class="language-detail-grid">
                                <div class="detail-card">
                                    <h3>Implementation Details</h3>
                                    <div class="detail-list">
                                        <div class="detail-row">
                                            <span class="detail-key">Description:</span>
                                            <span class="detail-value">{lang.description}</span>
                                        </div>
                                        <div class="detail-row">
                                            <span class="detail-key">Version:</span>
                                            <span class="detail-value">{lang.version || 'Unknown'}</span>
                                        </div>
                                        <div class="detail-row">
                                            <span class="detail-key">Runtime:</span>
                                            <span class="detail-value">{lang.runtime} {lang.runtime_version || ''}</span>
                                        </div>
                                        <div class="detail-row">
                                            <span class="detail-key">License:</span>
                                            <span class="detail-value">{lang.license}</span>
                                        </div>
                                        <div class="detail-row">
                                            <span class="detail-key">Maintainer:</span>
                                            <span class="detail-value">{lang.maintainer}</span>
                                        </div>
                                        <div class="detail-row">
                                            <span class="detail-key">Platform:</span>
                                            <span class="detail-value">{lang.platform || 'Unknown'}</span>
                                        </div>
                                    </div>
                                </div>

                                <div class="detail-card">
                                    <h3>Libraries Used</h3>
                                    <div class="libraries-list">
                                        {#each lang.libraries as library}
                                            <div class="library-item">
                                                <span class="library-icon">📚</span>
                                                <span class="library-name">{library}</span>
                                            </div>
                                        {/each}
                                    </div>
                                </div>

                                <div class="detail-card">
                                    <h3>Test Compliance</h3>
                                    {#if testResult && testResult.summary}
                                        <div class="compliance-stats">
                                            <div class="compliance-item">
                                                <span class="compliance-label">Pass Rate:</span>
                                                <span class="compliance-value">{((testResult.summary.passed / testResult.summary.total) * 100).toFixed(1)}%</span>
                                            </div>
                                            <div class="compliance-item">
                                                <span class="compliance-label">Tests Passed:</span>
                                                <span class="compliance-value">{testResult.summary.passed} / {testResult.summary.total}</span>
                                            </div>
                                            <div class="compliance-item">
                                                <span class="compliance-label">Tests Failed:</span>
                                                <span class="compliance-value">{testResult.summary.failed}</span>
                                                {#if testResult.summary.failed > 0}
                                                    <button class="error-details-btn" on:click={() => showErrorDetails(testResult)}>
                                                        View Errors
                                                    </button>
                                                {/if}
                                            </div>
                                            <div class="compliance-item">
                                                <span class="compliance-label">Errors:</span>
                                                <span class="compliance-value">{testResult.summary.errors}</span>
                                                {#if testResult.summary.errors > 0}
                                                    <button class="error-details-btn" on:click={() => showErrorDetails(testResult)}>
                                                        View Errors
                                                    </button>
                                                {/if}
                                            </div>
                                        </div>
                                    {:else}
                                        <p class="no-data">No test data available</p>
                                    {/if}
                                </div>

                                <div class="detail-card">
                                    <h3>Performance</h3>
                                    {#if benchmarkResult && benchmarkResult.benchmarks && benchmarkResult.benchmarks.length > 0}
                                        <div class="performance-stats">
                                            <div class="performance-item">
                                                <span class="performance-label">Avg Operations/Second:</span>
                                                <span class="performance-value">
                                                    {(benchmarkResult.benchmarks.reduce((sum, b) => sum + (b.ops_per_second || 0), 0) / benchmarkResult.benchmarks.length).toFixed(2)}
                                                </span>
                                            </div>
                                            <div class="performance-item">
                                                <span class="performance-label">Avg Execution Time:</span>
                                                <span class="performance-value">
                                                    {(benchmarkResult.benchmarks.reduce((sum, b) => sum + (b.avg_time_ms || 0), 0) / benchmarkResult.benchmarks.length).toFixed(3)} ms
                                                </span>
                                            </div>
                                            <div class="performance-item">
                                                <span class="performance-label">Benchmarks Run:</span>
                                                <span class="performance-value">{benchmarkResult.benchmarks.length}</span>
                                            </div>
                                        </div>
                                    {:else}
                                        <p class="no-data">No benchmark data available</p>
                                    {/if}
                                </div>
                            </div>
                        {:else}
                            <div class="select-language-prompt">
                                <p>Select an implementation from the sidebar to view detailed information</p>
                            </div>
                        {/if}
                    </div>
                </div>
            {/if}

            <div class="summary-card">
                <h3>Summary</h3>
                <p>Loaded {testResults.length} test result sets and {benchmarkResults.length} benchmark result sets</p>
                <p>Last updated: {new Date().toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}</p>
            </div>
        </div>
    {/if}

    <!-- Error Details Modal -->
    {#if showErrorModal && errorModalData}
        <div class="modal-overlay" on:click={closeErrorModal}>
            <div class="modal-content" on:click|stopPropagation>
                <div class="modal-header">
                    <h3>Error Details - {languageInfo[errorModalData.language]?.fullName || errorModalData.language}</h3>
                    <button class="modal-close" on:click={closeErrorModal}>×</button>
                </div>
                <div class="modal-body">
                    {#if errorModalData.tests && errorModalData.tests.length > 0}
                        <div class="error-list">
                            {#each errorModalData.tests.filter(test => test.status === 'failed' || test.status === 'error') as failedTest}
                                <div class="error-item">
                                    <div class="error-header">
                                        <span class="error-status {failedTest.status}">{failedTest.status}</span>
                                        <span class="error-name">{failedTest.name}</span>
                                    </div>
                                    <div class="error-description">{failedTest.description}</div>
                                    <div class="error-expression">
                                        <strong>Expression:</strong> <code>{failedTest.expression}</code>
                                    </div>
                                    {#if failedTest.error_message || failedTest.error}
                                        <div class="error-message">
                                            <strong>Error:</strong> {failedTest.error_message || failedTest.error}
                                        </div>
                                    {/if}
                                    {#if failedTest.expected !== undefined || failedTest.actual !== undefined}
                                        <div class="error-comparison">
                                            <div class="expected">
                                                <strong>Expected:</strong>
                                                <pre class="result-display">{failedTest.expected !== undefined ? JSON.stringify(failedTest.expected, null, 2) : 'Not specified'}</pre>
                                            </div>
                                            <div class="actual">
                                                <strong>Actual Result:</strong>
                                                <pre class="result-display {failedTest.actual === null ? 'null-result' : ''}">{failedTest.actual !== undefined ? JSON.stringify(failedTest.actual, null, 2) : 'Not available'}</pre>
                                            </div>
                                        </div>
                                    {/if}
                                    {#if failedTest.execution_time_ms}
                                        <div class="execution-time">
                                            <strong>Execution Time:</strong> {failedTest.execution_time_ms.toFixed(3)}ms
                                        </div>
                                    {/if}
                                </div>
                            {/each}
                        </div>
                    {:else}
                        <p class="no-errors">No detailed error information available. This may be because the test results were parsed from summary output only.</p>
                    {/if}
                </div>
            </div>
        </div>
    {/if}
</main>

<style>
    /* Base styles */
    :global(body) {
        --primary-color: #667eea;
        --secondary-color: #764ba2;
        --accent-color: #f093fb;
        --success-color: #48bb78;
        --warning-color: #ecc94b;
        --danger-color: #f56565;
        --text-color: #2d3748;
        --text-light: #718096;
        --bg-card: rgba(255, 255, 255, 0.98);
        --bg-dark: #1a202c;
        --shadow-sm: 0 4px 6px rgba(0, 0, 0, 0.1);
        --shadow-md: 0 10px 15px rgba(0, 0, 0, 0.1);
        --shadow-lg: 0 20px 25px rgba(0, 0, 0, 0.15);
        --border-radius: 20px;
        --transition-fast: 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        --transition-normal: 0.4s cubic-bezier(0.4, 0, 0.2, 1);
        --transition-slow: 0.6s cubic-bezier(0.4, 0, 0.2, 1);
    }

    main {
        position: relative;
    }

    /* Loading state */
    .loading {
        text-align: center;
        padding: 60px;
        background: var(--bg-card);
        border-radius: var(--border-radius);
        box-shadow: var(--shadow-md);
        animation: fadeIn 0.5s ease-out;
    }

    .spinner {
        width: 60px;
        height: 60px;
        border: 4px solid rgba(102, 126, 234, 0.2);
        border-top: 4px solid var(--primary-color);
        border-radius: 50%;
        animation: spin 1.2s cubic-bezier(0.5, 0.1, 0.5, 0.9) infinite;
        margin: 0 auto 20px;
        box-shadow: 0 0 15px rgba(102, 126, 234, 0.3);
    }

    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }

    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(20px); }
        to { opacity: 1; transform: translateY(0); }
    }

    @keyframes slideInRight {
        from { opacity: 0; transform: translateX(30px); }
        to { opacity: 1; transform: translateX(0); }
    }

    @keyframes slideInLeft {
        from { opacity: 0; transform: translateX(-30px); }
        to { opacity: 1; transform: translateX(0); }
    }

    @keyframes pulse {
        0% { transform: scale(1); }
        50% { transform: scale(1.05); }
        100% { transform: scale(1); }
    }

    @keyframes glow {
        0% { box-shadow: 0 0 5px rgba(102, 126, 234, 0.5); }
        50% { box-shadow: 0 0 20px rgba(102, 126, 234, 0.8); }
        100% { box-shadow: 0 0 5px rgba(102, 126, 234, 0.5); }
    }

    /* App container */
    .app-container {
        padding: 20px;
        animation: fadeIn 0.5s ease-out;
    }

    /* Error state */
    .error {
        background: linear-gradient(135deg, #fff5f5 0%, #fed7d7 100%);
        color: #c53030;
        padding: 25px;
        border-radius: var(--border-radius);
        margin-bottom: 30px;
        border-left: 5px solid #fc8181;
        box-shadow: var(--shadow-md);
        animation: fadeIn 0.5s ease-out;
    }

    /* Tabs */
    .tabs-container {
        margin-bottom: 30px;
        animation: fadeIn 0.5s ease-out;
    }

    .tabs {
        display: flex;
        background: rgba(255, 255, 255, 0.9);
        border-radius: 50px;
        padding: 8px;
        box-shadow: var(--shadow-md);
        backdrop-filter: blur(10px);
        border: 1px solid rgba(255, 255, 255, 0.2);
        max-width: 600px;
        margin: 0 auto;
    }

    .tab-button {
        flex: 1;
        background: transparent;
        border: none;
        padding: 12px 20px;
        border-radius: 40px;
        font-weight: 600;
        color: var(--text-light);
        cursor: pointer;
        transition: all var(--transition-normal);
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
    }

    .tab-button:hover {
        background: rgba(102, 126, 234, 0.1);
        color: var(--primary-color);
    }

    .tab-button.active {
        background: linear-gradient(135deg, var(--primary-color) 0%, var(--secondary-color) 100%);
        color: white;
        box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
    }

    .tab-icon {
        font-size: 1.2em;
    }

    /* Hero section */
    .hero-section {
        background: linear-gradient(135deg, rgba(102, 126, 234, 0.9) 0%, rgba(118, 75, 162, 0.9) 100%);
        border-radius: var(--border-radius);
        padding: 40px;
        margin-bottom: 30px;
        color: white;
        box-shadow: var(--shadow-lg);
        position: relative;
        overflow: hidden;
        animation: fadeIn 0.5s ease-out;
    }

    .hero-section::before {
        content: '';
        position: absolute;
        top: -50%;
        left: -50%;
        width: 200%;
        height: 200%;
        background: radial-gradient(circle, rgba(255, 255, 255, 0.1) 0%, transparent 80%);
        animation: rotate 20s linear infinite;
        z-index: 0;
    }

    @keyframes rotate {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }

    .hero-content {
        position: relative;
        z-index: 1;
    }

    .hero-title {
        font-size: 2.5em;
        margin-bottom: 15px;
        background: linear-gradient(45deg, #fff, #f0f8ff);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
        text-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
    }

    .hero-subtitle {
        font-size: 1.2em;
        opacity: 0.9;
        margin-bottom: 30px;
    }

    .hero-badge {
        display: inline-flex;
        align-items: center;
        background: rgba(255, 255, 255, 0.15);
        backdrop-filter: blur(10px);
        border-radius: 50px;
        padding: 10px 20px;
        margin-right: 15px;
        margin-bottom: 10px;
        border: 1px solid rgba(255, 255, 255, 0.2);
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
        transition: all var(--transition-normal);
        animation: slideInRight 0.5s ease-out;
    }

    .hero-badge:hover {
        transform: translateY(-3px);
        box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
        background: rgba(255, 255, 255, 0.25);
    }

    .badge-icon {
        font-size: 1.5em;
        margin-right: 10px;
    }

    .badge-text {
        font-weight: 500;
    }

    /* Cards */
    .metrics-card, .chart-card, .summary-card, .detail-card {
        background: var(--bg-card);
        border-radius: var(--border-radius);
        padding: 45px;
        margin-bottom: 60px;
        box-shadow: var(--shadow-md);
        backdrop-filter: blur(10px);
        border: 1px solid rgba(255, 255, 255, 0.2);
        transition: all var(--transition-normal);
        animation: fadeIn 0.5s ease-out;
    }

    .metrics-card:hover, .chart-card:hover, .summary-card:hover, .detail-card:hover {
        transform: translateY(-3px);
        box-shadow: 0 15px 35px rgba(0, 0, 0, 0.15);
    }

    .metrics-card h2, .chart-card h3, .summary-card h3, .detail-card h3 {
        color: var(--text-color);
        margin-bottom: 25px;
        font-weight: 700;
        position: relative;
        padding-bottom: 15px;
        display: inline-block;
    }

    .metrics-card h2::after, .chart-card h3::after, .summary-card h3::after, .detail-card h3::after {
        content: '';
        position: absolute;
        bottom: 0;
        left: 0;
        width: 50px;
        height: 4px;
        background: linear-gradient(90deg, var(--primary-color), var(--secondary-color));
        border-radius: 2px;
    }

    /* Metrics */
    .metrics-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
        gap: 50px;
    }

    .metric {
        text-align: center;
        padding: 25px;
        background: linear-gradient(145deg, #ffffff, #f8fafc);
        border-radius: var(--border-radius);
        box-shadow: var(--shadow-sm);
        transition: all var(--transition-normal);
        position: relative;
        overflow: hidden;
        border: 1px solid rgba(102, 126, 234, 0.1);
    }

    .metric::before {
        content: '';
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 5px;
        background: linear-gradient(90deg, var(--primary-color), var(--secondary-color));
    }

    .metric:hover {
        transform: translateY(-2px);
        box-shadow: var(--shadow-md);
    }

    .metric-icon {
        font-size: 2em;
        margin-bottom: 15px;
        display: block;
    }

    .metric-label {
        display: block;
        color: var(--text-light);
        font-size: 1em;
        margin-bottom: 10px;
        font-weight: 500;
    }

    .metric-value {
        display: block;
        color: var(--text-color);
        font-size: 2.5em;
        font-weight: 800;
        background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
    }

    /* Implementations grid */
    .implementations-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        gap: 45px;
        animation: fadeIn 0.5s ease-out;
    }

    .implementation-card {
        background: var(--bg-card);
        border-radius: var(--border-radius);
        overflow: hidden;
        box-shadow: var(--shadow-md);
        transition: all var(--transition-normal);
        border: 1px solid rgba(255, 255, 255, 0.2);
        cursor: pointer;
    }

    .implementation-card:hover {
        transform: translateY(-4px);
        box-shadow: 0 20px 40px rgba(0, 0, 0, 0.18);
    }

    .implementation-header {
        background: linear-gradient(135deg, var(--primary-color) 0%, var(--secondary-color) 100%);
        padding: 20px;
        color: white;
        display: flex;
        align-items: center;
        gap: 15px;
    }

    .implementation-icon {
        font-size: 2em;
    }

    .implementation-name {
        font-size: 1.5em;
        font-weight: 700;
        margin: 0;
    }

    .implementation-body {
        padding: 20px;
    }

    .implementation-description {
        color: var(--text-light);
        margin-bottom: 20px;
        line-height: 1.6;
    }

    .implementation-details {
        margin-bottom: 20px;
    }

    .detail-item {
        display: flex;
        justify-content: space-between;
        margin-bottom: 10px;
        padding-bottom: 10px;
        border-bottom: 1px solid rgba(0, 0, 0, 0.05);
    }

    .detail-label {
        font-weight: 600;
        color: var(--text-color);
    }

    .detail-value {
        color: var(--text-light);
    }

    .implementation-footer {
        padding: 15px 20px;
        background: rgba(0, 0, 0, 0.02);
        border-top: 1px solid rgba(0, 0, 0, 0.05);
    }

    .view-details-btn {
        width: 100%;
        background: linear-gradient(135deg, var(--primary-color) 0%, var(--secondary-color) 100%);
        color: white;
        border: none;
        padding: 12px;
        border-radius: 8px;
        font-weight: 600;
        cursor: pointer;
        transition: all var(--transition-normal);
    }

    .view-details-btn:hover {
        transform: translateY(-1px);
        box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
    }

    /* Details tab */
    .details-container {
        display: grid;
        grid-template-columns: 250px 1fr;
        gap: 50px;
        animation: fadeIn 0.5s ease-out;
    }

    .details-sidebar {
        background: var(--bg-card);
        border-radius: var(--border-radius);
        padding: 25px;
        box-shadow: var(--shadow-md);
        height: fit-content;
    }

    .details-sidebar h3 {
        margin-bottom: 20px;
        color: var(--text-color);
        font-weight: 600;
    }

    .language-list {
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .language-button {
        display: flex;
        align-items: center;
        gap: 10px;
        background: transparent;
        border: 1px solid rgba(0, 0, 0, 0.1);
        padding: 12px 15px;
        border-radius: 8px;
        cursor: pointer;
        transition: all var(--transition-normal);
        text-align: left;
    }

    .language-button:hover {
        background: rgba(102, 126, 234, 0.1);
        transform: translateX(5px);
    }

    .language-button.active {
        background: linear-gradient(135deg, var(--primary-color) 0%, var(--secondary-color) 100%);
        color: white;
        border-color: transparent;
        box-shadow: 0 5px 15px rgba(102, 126, 234, 0.3);
    }

    .language-icon {
        font-size: 1.5em;
    }

    .language-name {
        font-weight: 500;
    }

    .details-content {
        background: var(--bg-card);
        border-radius: var(--border-radius);
        padding: 30px;
        box-shadow: var(--shadow-md);
        animation: fadeIn 0.5s ease-out;
    }

    .language-detail-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 30px;
        padding-bottom: 20px;
        border-bottom: 1px solid rgba(0, 0, 0, 0.1);
    }

    .language-detail-title {
        display: flex;
        align-items: center;
        gap: 15px;
    }

    .language-detail-icon {
        font-size: 2.5em;
    }

    .language-detail-title h2 {
        margin: 0;
        font-size: 1.8em;
        color: var(--text-color);
    }

    .repo-link {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        background: linear-gradient(135deg, var(--primary-color) 0%, var(--secondary-color) 100%);
        color: white;
        text-decoration: none;
        padding: 10px 20px;
        border-radius: 8px;
        font-weight: 500;
        transition: all var(--transition-normal);
    }

    .repo-link:hover {
        transform: translateY(-3px);
        box-shadow: 0 5px 15px rgba(102, 126, 234, 0.4);
    }

    .language-detail-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        gap: 50px;
    }

    .detail-list {
        display: flex;
        flex-direction: column;
        gap: 15px;
    }

    .detail-row {
        display: flex;
        flex-direction: column;
        gap: 5px;
    }

    .detail-key {
        font-weight: 600;
        color: var(--text-color);
    }

    .detail-value {
        color: var(--text-light);
    }

    .libraries-list {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .library-item {
        display: flex;
        align-items: center;
        gap: 10px;
        background: rgba(102, 126, 234, 0.1);
        padding: 12px 15px;
        border-radius: 8px;
        transition: all var(--transition-normal);
    }

    .library-item:hover {
        transform: translateX(5px);
        background: rgba(102, 126, 234, 0.2);
    }

    .library-icon {
        color: var(--primary-color);
    }

    .library-name {
        font-weight: 500;
        color: var(--text-color);
    }

    .compliance-stats, .performance-stats {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
        gap: 15px;
    }

    .compliance-item, .performance-item {
        background: rgba(102, 126, 234, 0.1);
        padding: 15px;
        border-radius: 8px;
        transition: all var(--transition-normal);
    }

    .compliance-item:hover, .performance-item:hover {
        background: rgba(102, 126, 234, 0.2);
        transform: translateY(-3px);
    }

    .compliance-label, .performance-label {
        display: block;
        font-weight: 500;
        color: var(--text-color);
        margin-bottom: 8px;
        font-size: 0.9em;
    }

    .compliance-value, .performance-value {
        display: block;
        font-size: 1.5em;
        font-weight: 700;
        color: var(--primary-color);
    }

    .select-language-prompt {
        display: flex;
        justify-content: center;
        align-items: center;
        height: 300px;
        color: var(--text-light);
        font-style: italic;
    }

    .summary-card{
        margin-top: 10px;
    }
    .summary-card p {
        color: var(--text-light);
        margin: 10px 0;
        line-height: 1.6;
    }

    .note {
        font-style: italic;
        color: #856404;
        background: #fff3cd;
        padding: 15px;
        border-radius: 8px;
        margin-top: 15px;
        border-left: 4px solid #ffeeba;
    }

    /* Responsive styles */
    @media (max-width: 768px) {
        .app-container {
            padding: 25px;
        }

        .metrics-card, .chart-card, .summary-card, .detail-card {
            padding: 30px;
            margin-bottom: 35px;
        }

        .metrics-grid {
            gap: 30px;
            grid-template-columns: 1fr;
        }

        .implementations-grid {
            gap: 30px;
        }

        .language-detail-grid {
            gap: 30px;
            grid-template-columns: 1fr;
        }

        .details-container {
            gap: 35px;
            grid-template-columns: 1fr;
        }

        .tabs {
            flex-direction: column;
            border-radius: var(--border-radius);
            max-width: 100%;
        }

        .tab-button {
            border-radius: 8px;
        }

        .hero-section {
            padding: 30px;
        }

        .hero-title {
            font-size: 2em;
        }
    }

    /* Error Details Button */
    .error-details-btn {
        background: #dc2626;
        color: white;
        border: none;
        padding: 4px 8px;
        border-radius: 4px;
        font-size: 0.75rem;
        font-weight: 500;
        cursor: pointer;
        margin-left: 8px;
        transition: all 0.2s ease;
    }

    .error-details-btn:hover {
        background: #b91c1c;
        transform: translateY(-1px);
    }

    /* Error Modal */
    .modal-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
        padding: 16px;
    }

    .modal-content {
        background: white;
        border-radius: 6px;
        max-width: 800px;
        width: 100%;
        max-height: 80vh;
        overflow: hidden;
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
    }

    .modal-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 16px 20px;
        border-bottom: 1px solid #e5e7eb;
        background: #f9fafb;
    }

    .modal-header h3 {
        margin: 0;
        font-size: 1.125rem;
        font-weight: 600;
        color: #1f2937;
    }

    .modal-close {
        background: none;
        border: none;
        font-size: 1.5rem;
        cursor: pointer;
        color: #6b7280;
        padding: 4px;
        border-radius: 4px;
        transition: all 0.2s ease;
    }

    .modal-close:hover {
        background: #e5e7eb;
        color: #374151;
    }

    .modal-body {
        padding: 20px;
        max-height: 60vh;
        overflow-y: auto;
    }

    .error-list {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .error-item {
        border: 1px solid #e5e7eb;
        border-radius: 6px;
        padding: 16px;
        background: #fafafa;
    }

    .error-header {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 8px;
    }

    .error-status {
        padding: 2px 6px;
        border-radius: 3px;
        font-size: 0.75rem;
        font-weight: 500;
        text-transform: uppercase;
    }

    .error-status.failed {
        background: #fef2f2;
        color: #dc2626;
        border: 1px solid #fecaca;
    }

    .error-status.error {
        background: #fef2f2;
        color: #dc2626;
        border: 1px solid #fecaca;
    }

    .error-name {
        font-weight: 600;
        color: #1f2937;
    }

    .error-description {
        color: #6b7280;
        font-size: 0.875rem;
        margin-bottom: 8px;
    }

    .error-expression {
        margin-bottom: 8px;
    }

    .error-expression code {
        background: #f3f4f6;
        padding: 2px 4px;
        border-radius: 3px;
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        font-size: 0.875rem;
    }

    .error-message {
        background: #fef2f2;
        border: 1px solid #fecaca;
        border-radius: 4px;
        padding: 8px;
        margin-bottom: 8px;
        color: #dc2626;
        font-size: 0.875rem;
    }

    .error-comparison {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 12px;
        margin-top: 12px;
    }

    .expected, .actual {
        border: 1px solid #e5e7eb;
        border-radius: 4px;
        padding: 8px;
        background: white;
    }

    .expected strong, .actual strong {
        display: block;
        margin-bottom: 4px;
        font-size: 0.875rem;
        color: #374151;
    }

    .expected pre, .actual pre {
        margin: 0;
        font-size: 0.75rem;
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        white-space: pre-wrap;
        word-break: break-word;
        color: #1f2937;
    }

    .no-errors {
        text-align: center;
        color: #6b7280;
        font-style: italic;
        padding: 32px;
    }

    .result-display {
        background: #f8fafc;
        border: 1px solid #e2e8f0;
        border-radius: 4px;
        padding: 8px;
        font-size: 0.75rem;
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        white-space: pre-wrap;
        word-break: break-word;
        color: #1f2937;
        max-height: 200px;
        overflow-y: auto;
    }

    .null-result {
        background: #fef3c7;
        border-color: #f59e0b;
        color: #92400e;
        font-style: italic;
    }

    .execution-time {
        margin-top: 8px;
        padding: 6px 8px;
        background: #f0f9ff;
        border: 1px solid #bae6fd;
        border-radius: 4px;
        font-size: 0.75rem;
        color: #0369a1;
    }

    .execution-time strong {
        font-weight: 600;
    }

    @media (max-width: 768px) {
        .modal-content {
            margin: 16px;
            max-height: 90vh;
        }

        .error-comparison {
            grid-template-columns: 1fr;
        }

        .modal-body {
            padding: 16px;
        }
    }
</style>
