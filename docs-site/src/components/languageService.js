/**
 * Language Service
 *
 * Centralized service for managing language metadata and information
 * across the FHIRPath comparison application.
 */

// Language metadata with comprehensive information
// Logos sourced from devicons (https://github.com/devicons/devicon) under MIT License
export const languageInfo = {
    javascript: {
        fullName: 'JavaScript',
        description: 'Node.js implementation using the fhirpath.js library',
        repository: 'https://github.com/HL7/fhirpath.js',
        license: 'BSD-3-Clause',
        maintainer: 'HL7 FHIR Community',
        logo: '/fhirpath-rs/logos/javascript.svg',
        icon: '🟨',
        libraries: ['fhirpath.js', 'antlr4-javascript-runtime'],
        runtime: 'Node.js',
        color: '#f7df1e'
    },
    python: {
        fullName: 'Python',
        description: 'Python implementation using the fhirpath_py library',
        repository: 'https://github.com/beda-software/fhirpath-py',
        license: 'MIT',
        maintainer: 'Beda Software',
        logo: '/fhirpath-rs/logos/python.svg',
        icon: '🐍',
        libraries: ['fhirpath-py', 'antlr4-python3-runtime'],
        runtime: 'Python 3',
        color: '#3776ab'
    },
    java: {
        fullName: 'Java',
        description: 'Java implementation using the HAPI FHIR library',
        repository: 'https://github.com/hapifhir/org.hl7.fhir.core',
        license: 'Apache-2.0',
        maintainer: 'HAPI FHIR',
        logo: '/fhirpath-rs/logos/java.svg',
        icon: '☕',
        libraries: ['org.hl7.fhir.core', 'ANTLR4'],
        runtime: 'JVM',
        color: '#ed8b00'
    },
    csharp: {
        fullName: 'C#',
        description: 'C# implementation using the Firely SDK',
        repository: 'https://github.com/FirelyTeam/firely-net-sdk',
        license: 'BSD-3-Clause',
        maintainer: 'Firely',
        logo: '/fhirpath-rs/logos/csharp.svg',
        icon: '🔷',
        libraries: ['Hl7.Fhir.Core', 'Hl7.FhirPath'],
        runtime: '.NET',
        color: '#239120'
    },
    rust: {
        fullName: 'Rust',
        description: 'Rust implementation using OctoFHIR FHIRPath',
        repository: 'https://github.com/octofhir/fhirpath-rs',
        license: 'Apache-2.0',
        maintainer: 'OctoFHIR Team',
        logo: '/fhirpath-rs/logos/rust.svg',
        icon: '🦀',
        libraries: ['fhirpath-core', 'serde_json'],
        runtime: 'Native',
        color: '#ce422b'
    },
    go: {
        fullName: 'Go',
        description: 'Go implementation using fhir-toolbox-go library',
        repository: 'https://github.com/DAMEDIC/fhir-toolbox-go',
        license: 'Apache-2.0',
        maintainer: 'DAMEDIC',
        logo: '/fhirpath-rs/logos/go.svg',
        icon: '🔵',
        libraries: ['fhir-toolbox-go'],
        runtime: 'Native',
        color: '#00add8'
    },
    clojure: {
        fullName: 'Clojure',
        description: 'Clojure implementation using fhirpath.clj library',
        repository: 'https://github.com/HealthSamurai/fhirpath.clj',
        license: 'MIT',
        maintainer: 'HealthSamurai',
        logo: '/fhirpath-rs/logos/clojure.svg',
        icon: '🟢',
        libraries: ['fhirpath.clj', 'ANTLR4'],
        runtime: 'JVM',
        color: '#5881d8'
    }
};

/**
 * Get information for a specific language
 * @param {string} languageId - The language identifier
 * @returns {Object|null} Language information object or null if not found
 */
export function getLanguageInfo(languageId) {
    return languageInfo[languageId] || null;
}

/**
 * Get all available languages
 * @returns {Array} Array of language identifiers
 */
export function getAllLanguages() {
    return Object.keys(languageInfo);
}

/**
 * Get language information with runtime data merged
 * @param {Array} testResults - Test results array
 * @param {Array} benchmarkResults - Benchmark results array
 * @returns {Object} Enhanced language information with runtime data
 */
export function getEnhancedLanguageInfo(testResults = [], benchmarkResults = []) {
    const enhanced = { ...languageInfo };

    // Merge version information from benchmark results
    benchmarkResults.forEach(result => {
        if (result.system_info && result.language && enhanced[result.language]) {
            enhanced[result.language] = {
                ...enhanced[result.language],
                version: result.system_info.fhirpath_version || 'Unknown',
                platform: result.system_info.platform || 'Unknown',
                runtimeVersion: result.system_info.node_version ||
                               result.system_info.python_version ||
                               result.system_info.java_version ||
                               result.system_info.dotnet_version ||
                               result.system_info.rust_version ||
                               result.system_info.go_version ||
                               result.system_info.clojure_version ||
                               'Unknown'
            };
        }
    });

    // Add test statistics
    testResults.forEach(result => {
        if (result.language && enhanced[result.language]) {
            enhanced[result.language] = {
                ...enhanced[result.language],
                testStats: {
                    total: result.summary?.total || 0,
                    passed: result.summary?.passed || 0,
                    failed: result.summary?.failed || 0,
                    errors: result.summary?.errors || 0,
                    successRate: result.summary?.total > 0 ?
                        ((result.summary.passed / result.summary.total) * 100).toFixed(1) : '0.0'
                }
            };
        }
    });

    return enhanced;
}

/**
 * Get color scheme for charts
 * @returns {Object} Color mapping for each language
 */
export function getLanguageColors() {
    const colors = {};
    Object.keys(languageInfo).forEach(lang => {
        const color = languageInfo[lang].color;
        colors[lang] = {
            bg: `${color}B3`, // 70% opacity
            border: color
        };
    });
    return colors;
}

/**
 * Format language name for display
 * @param {string} languageId - The language identifier
 * @returns {string} Formatted display name
 */
export function formatLanguageName(languageId) {
    const info = getLanguageInfo(languageId);
    return info ? info.fullName : languageId.charAt(0).toUpperCase() + languageId.slice(1);
}

/**
 * Get language statistics summary
 * @param {Array} testResults - Test results array
 * @param {Array} benchmarkResults - Benchmark results array
 * @returns {Object} Summary statistics
 */
export function getLanguageStatsSummary(testResults = [], benchmarkResults = []) {
    const stats = {
        totalLanguages: 0,
        totalTests: 0,
        totalBenchmarks: 0,
        averageSuccessRate: 0,
        languageBreakdown: {}
    };

    const languages = new Set();

    testResults.forEach(result => {
        if (result.language) {
            languages.add(result.language);
            stats.totalTests += result.summary?.total || 0;

            if (!stats.languageBreakdown[result.language]) {
                stats.languageBreakdown[result.language] = {
                    tests: 0,
                    benchmarks: 0,
                    successRate: 0
                };
            }

            stats.languageBreakdown[result.language].tests = result.summary?.total || 0;
            stats.languageBreakdown[result.language].successRate =
                result.summary?.total > 0 ?
                    ((result.summary.passed / result.summary.total) * 100) : 0;
        }
    });

    benchmarkResults.forEach(result => {
        if (result.language) {
            languages.add(result.language);
            stats.totalBenchmarks += result.benchmarks?.length || 0;

            if (!stats.languageBreakdown[result.language]) {
                stats.languageBreakdown[result.language] = {
                    tests: 0,
                    benchmarks: 0,
                    successRate: 0
                };
            }

            stats.languageBreakdown[result.language].benchmarks = result.benchmarks?.length || 0;
        }
    });

    stats.totalLanguages = languages.size;

    // Calculate average success rate
    const successRates = Object.values(stats.languageBreakdown)
        .map(lang => lang.successRate)
        .filter(rate => rate > 0);

    stats.averageSuccessRate = successRates.length > 0 ?
        (successRates.reduce((sum, rate) => sum + rate, 0) / successRates.length) : 0;

    return stats;
}
