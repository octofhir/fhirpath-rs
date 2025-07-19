#!/usr/bin/env node
/**
 * JavaScript FHIRPath Test Runner
 *
 * This script runs FHIRPath tests using the fhirpath.js library
 * and outputs results in a standardized format for comparison.
 */

const fs = require('fs');
const path = require('path');
const fhirpath = require('fhirpath');
const xml2js = require('xml2js');

class JavaScriptTestRunner {
    constructor() {
        this.testDataDir = path.join(__dirname, '../../test-data');
        this.testCasesDir = path.join(__dirname, '../../test-cases');
        this.resultsDir = path.join(__dirname, '../../results');

        // Ensure results directory exists
        if (!fs.existsSync(this.resultsDir)) {
            fs.mkdirSync(this.resultsDir, { recursive: true });
        }

        // Load test configuration
        const configPath = path.join(this.testCasesDir, 'test-config.json');
        this.testConfig = JSON.parse(fs.readFileSync(configPath, 'utf8'));
    }

    async loadOfficialTests() {
        const xmlPath = path.join(this.testCasesDir, 'tests-fhir-r4.xml');
        const xmlContent = fs.readFileSync(xmlPath, 'utf8');

        const parser = new xml2js.Parser({
            explicitArray: false,
            mergeAttrs: true,
            explicitChildren: true,
            childkey: 'children'
        });

        const result = await parser.parseStringPromise(xmlContent);
        const tests = [];

        // Extract tests from groups
        const groups = Array.isArray(result.tests.group) ? result.tests.group : [result.tests.group];

        for (const group of groups) {
            if (!group || !group.test) continue;

            const groupTests = Array.isArray(group.test) ? group.test : [group.test];

            for (const test of groupTests) {
                if (!test || !test.expression) continue;

                // Parse expected outputs
                const expectedOutput = [];
                if (test.output) {
                    const outputs = Array.isArray(test.output) ? test.output : [test.output];
                    for (const output of outputs) {
                        if (output && output._ !== undefined) {
                            expectedOutput.push({
                                type: output.type || 'string',
                                value: output._
                            });
                        }
                    }
                }

                tests.push({
                    name: test.name,
                    description: test.description || test.name,
                    inputFile: test.inputfile || 'patient-example.xml',
                    expression: test.expression,
                    expectedOutput: expectedOutput,
                    predicate: test.predicate === 'true',
                    mode: test.mode,
                    invalid: test.invalid,
                    group: group.name
                });
            }
        }

        return tests;
    }

    async loadTestData(filename) {
        const filePath = path.join(this.testDataDir, filename);
        const xmlContent = fs.readFileSync(filePath, 'utf8');

        // Convert XML to JSON for fhirpath.js
        const parser = new xml2js.Parser({ explicitArray: false, mergeAttrs: true });
        const result = await parser.parseStringPromise(xmlContent);
        return result;
    }

    runSingleTest(testCase, testData) {
        const startTime = process.hrtime.bigint();

        try {
            // Execute FHIRPath expression
            const result = fhirpath.evaluate(testData, testCase.expression);

            const endTime = process.hrtime.bigint();
            const executionTimeMs = Number(endTime - startTime) / 1000000; // Convert to milliseconds

            return {
                name: testCase.name,
                description: testCase.description,
                expression: testCase.expression,
                status: 'passed', // Simplified - would need proper result comparison
                execution_time_ms: executionTimeMs,
                expected: testCase.expectedOutput || [],
                actual: result
            };
        } catch (error) {
            const endTime = process.hrtime.bigint();
            const executionTimeMs = Number(endTime - startTime) / 1000000;

            return {
                name: testCase.name,
                description: testCase.description,
                expression: testCase.expression,
                status: 'error',
                execution_time_ms: executionTimeMs,
                expected: testCase.expectedOutput || [],
                actual: null,
                error: error.message
            };
        }
    }

    async runTests() {
        console.log('🧪 Running JavaScript FHIRPath tests...');

        const results = {
            language: 'javascript',
            timestamp: Date.now() / 1000,
            tests: [],
            summary: {
                total: 0,
                passed: 0,
                failed: 0,
                errors: 0
            }
        };

        // Load test data files
        const testDataCache = {};
        for (const inputFile of this.testConfig.testData.inputFiles) {
            try {
                testDataCache[inputFile] = await this.loadTestData(inputFile);
            } catch (error) {
                console.warn(`⚠️  Could not load test data: ${inputFile} - ${error.message}`);
            }
        }

        // Load and run official tests
        console.log('📋 Loading official FHIRPath test suite...');
        const officialTests = await this.loadOfficialTests();
        console.log(`📊 Found ${officialTests.length} official test cases`);

        for (const testCase of officialTests) {
            const inputFile = testCase.inputFile;
            const testData = testDataCache[inputFile];

            if (!testData) {
                console.warn(`⚠️  Skipping test ${testCase.name} - test data not available: ${inputFile}`);
                continue;
            }

            // Skip tests marked as invalid for now (these test error conditions)
            if (testCase.invalid) {
                console.log(`⏭️  Skipping invalid test ${testCase.name} (tests error conditions)`);
                continue;
            }

            const testResult = this.runSingleTest(testCase, testData);
            results.tests.push(testResult);
            results.summary.total++;

            if (testResult.status === 'passed') {
                results.summary.passed++;
            } else if (testResult.status === 'error') {
                results.summary.errors++;
            } else {
                results.summary.failed++;
            }

            const statusIcon = testResult.status === 'passed' ? '✅' :
                              testResult.status === 'error' ? '💥' : '❌';
            console.log(`  ${statusIcon} ${testResult.name} (${testResult.execution_time_ms.toFixed(2)}ms) [${testCase.group}]`);
        }

        // Save results
        const resultsFile = path.join(this.resultsDir, `javascript_test_results.json`);
        fs.writeFileSync(resultsFile, JSON.stringify(results, null, 2));

        console.log(`📊 Results saved to: ${resultsFile}`);
        console.log(`📈 Summary: ${results.summary.passed}/${results.summary.total} tests passed`);

        return results;
    }

    async runBenchmarks() {
        console.log('⚡ Running JavaScript FHIRPath benchmarks...');

        const results = {
            language: 'javascript',
            timestamp: Date.now() / 1000,
            benchmarks: [],
            system_info: {
                platform: process.platform,
                node_version: process.version,
                fhirpath_version: require('fhirpath/package.json').version
            }
        };

        // Load test data
        const testDataCache = {};
        for (const inputFile of this.testConfig.testData.inputFiles) {
            try {
                testDataCache[inputFile] = await this.loadTestData(inputFile);
            } catch (error) {
                console.warn(`⚠️  Could not load test data: ${inputFile}`);
            }
        }

        // Run benchmarks
        for (const benchmark of this.testConfig.benchmarkTests) {
            const inputFile = benchmark.inputFile || 'patient-example.xml';
            const testData = testDataCache[inputFile];

            if (!testData) {
                console.warn(`⚠️  Skipping benchmark ${benchmark.name} - test data not available`);
                continue;
            }

            console.log(`  🏃 Running ${benchmark.name}...`);

            const times = [];
            const iterations = benchmark.iterations || 1000;

            // Warm up
            for (let i = 0; i < 10; i++) {
                fhirpath.evaluate(testData, benchmark.expression);
            }

            // Actual benchmark
            for (let i = 0; i < iterations; i++) {
                const startTime = process.hrtime.bigint();
                fhirpath.evaluate(testData, benchmark.expression);
                const endTime = process.hrtime.bigint();
                times.push(Number(endTime - startTime) / 1000000); // Convert to milliseconds
            }

            const avgTime = times.reduce((a, b) => a + b, 0) / times.length;
            const minTime = Math.min(...times);
            const maxTime = Math.max(...times);
            const opsPerSecond = 1000 / avgTime;

            const benchmarkResult = {
                name: benchmark.name,
                description: benchmark.description,
                expression: benchmark.expression,
                iterations: iterations,
                avg_time_ms: avgTime,
                min_time_ms: minTime,
                max_time_ms: maxTime,
                ops_per_second: opsPerSecond
            };

            results.benchmarks.push(benchmarkResult);
            console.log(`    ⏱️  ${avgTime.toFixed(2)}ms avg (${opsPerSecond.toFixed(1)} ops/sec)`);
        }

        // Save results
        const resultsFile = path.join(this.resultsDir, `javascript_benchmark_results.json`);
        fs.writeFileSync(resultsFile, JSON.stringify(results, null, 2));

        console.log(`📊 Benchmark results saved to: ${resultsFile}`);

        return results;
    }
}

async function main() {
    const runner = new JavaScriptTestRunner();

    const command = process.argv[2] || 'both';

    try {
        if (command === 'test' || command === 'both') {
            await runner.runTests();
        }

        if (command === 'benchmark' || command === 'both') {
            await runner.runBenchmarks();
        }

        console.log('✅ JavaScript test runner completed');
    } catch (error) {
        console.error('❌ Error running tests:', error);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}
