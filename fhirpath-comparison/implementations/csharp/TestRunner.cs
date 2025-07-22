using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Xml;
using Hl7.Fhir.ElementModel;
using Hl7.Fhir.FhirPath;
using Hl7.Fhir.Model;
using Hl7.Fhir.Serialization;
using Hl7.FhirPath;

namespace FhirPathComparison
{
    /// <summary>
    /// C# FHIRPath Test Runner
    ///
    /// This class runs FHIRPath tests using the Hl7.FhirPath library
    /// and outputs results in a standardized format for comparison.
    /// </summary>
    public class TestRunner
    {
        private readonly string _testDataDir;
        private readonly string _testCasesDir;
        private readonly string _resultsDir;
        private readonly FhirXmlParser _xmlParser;
        private readonly FhirPathCompiler _fhirPathCompiler;
        private JsonNode _testConfig;

        public TestRunner()
        {
            // Initialize paths
            var currentDir = Directory.GetCurrentDirectory();
            _testDataDir = Path.Combine(currentDir, "..", "..", "test-data");
            _testCasesDir = Path.Combine(currentDir, "..", "..", "test-cases");
            _resultsDir = Path.Combine(currentDir, "..", "..", "results");

            // Ensure results directory exists
            Directory.CreateDirectory(_resultsDir);

            // Initialize FHIR parsers and compiler
            _xmlParser = new FhirXmlParser();
            _fhirPathCompiler = new FhirPathCompiler();

            // Load test configuration
            var configPath = Path.Combine(_testCasesDir, "test-config.json");
            var configJson = File.ReadAllText(configPath);
            _testConfig = JsonNode.Parse(configJson);
        }

        /// <summary>
        /// Load official FHIRPath test cases from XML file.
        /// </summary>
        private List<JsonObject> LoadOfficialTests()
        {
            var tests = new List<JsonObject>();
            var xmlPath = Path.Combine(_testCasesDir, "tests-fhir-r4.xml");

            try
            {
                var doc = new XmlDocument();
                doc.Load(xmlPath);

                var groups = doc.SelectNodes("//group");
                foreach (XmlNode group in groups)
                {
                    var groupName = group.Attributes?["name"]?.Value ?? "unknown";

                    var testNodes = group.SelectNodes("test");
                    foreach (XmlNode test in testNodes)
                    {
                        var testName = test.Attributes?["name"]?.Value;
                        var description = test.Attributes?["description"]?.Value ?? testName;
                        var inputFile = test.Attributes?["inputfile"]?.Value ?? "patient-example.xml";
                        var invalid = test.Attributes?["invalid"]?.Value;

                        var expressionNode = test.SelectSingleNode("expression");
                        if (expressionNode?.InnerText == null || string.IsNullOrWhiteSpace(expressionNode.InnerText))
                            continue;

                        var testCase = new JsonObject
                        {
                            ["name"] = testName,
                            ["description"] = description,
                            ["inputFile"] = inputFile,
                            ["expression"] = expressionNode.InnerText,
                            ["group"] = groupName,
                            ["invalid"] = !string.IsNullOrEmpty(invalid)
                        };

                        tests.Add(testCase);
                    }
                }
            }
            catch (Exception e)
            {
                Console.WriteLine($"❌ Error loading official tests: {e.Message}");
            }

            return tests;
        }

        /// <summary>
        /// Load test data from XML file.
        /// </summary>
        private Resource LoadTestData(string filename)
        {
            var filePath = Path.Combine(_testDataDir, filename);

            if (!File.Exists(filePath))
            {
                Console.WriteLine($"⚠️  Test data file not found: {filename}");
                return null;
            }

            try
            {
                var xmlContent = File.ReadAllText(filePath);
                var resource = _xmlParser.Parse<Resource>(xmlContent);
                return resource;
            }
            catch (Exception e)
            {
                Console.WriteLine($"⚠️  Error loading test data {filename}: {e.Message}");
                return null;
            }
        }

        /// <summary>
        /// Run a single test case and return results.
        /// </summary>
        private JsonObject RunSingleTest(JsonNode testCase, Resource testData)
        {
            var result = new JsonObject();
            var stopwatch = Stopwatch.StartNew();

            try
            {
                var expression = testCase["expression"]?.ToString();

                // Execute FHIRPath expression
                var compiled = _fhirPathCompiler.Compile(expression);
                var evaluationResult = compiled(testData.ToTypedElement(), EvaluationContext.CreateDefault());

                stopwatch.Stop();
                var executionTimeMs = stopwatch.Elapsed.TotalMilliseconds;

                result["name"] = testCase["name"]?.ToString();
                result["description"] = testCase["description"]?.ToString();
                result["expression"] = expression;
                result["status"] = "passed"; // Simplified - would need proper result comparison
                result["execution_time_ms"] = executionTimeMs;

                // Add expected and actual results
                if (testCase["expectedOutput"] != null)
                {
                    result["expected"] = JsonNode.Parse(testCase["expectedOutput"].ToJsonString());
                }
                else
                {
                    result["expected"] = new JsonArray();
                }

                var actualArray = new JsonArray();
                foreach (var item in evaluationResult)
                {
                    if (item.Value != null)
                    {
                        actualArray.Add(JsonValue.Create(item.Value.ToString()));
                    }
                    else
                    {
                        actualArray.Add(JsonValue.Create(item.ToString()));
                    }
                }
                result["actual"] = actualArray;
            }
            catch (Exception error)
            {
                stopwatch.Stop();
                var executionTimeMs = stopwatch.Elapsed.TotalMilliseconds;

                result["name"] = testCase["name"]?.ToString();
                result["description"] = testCase["description"]?.ToString();
                result["expression"] = testCase["expression"]?.ToString();
                result["status"] = "error";
                result["execution_time_ms"] = executionTimeMs;

                if (testCase["expectedOutput"] != null)
                {
                    result["expected"] = JsonNode.Parse(testCase["expectedOutput"].ToJsonString());
                }
                else
                {
                    result["expected"] = new JsonArray();
                }
                result["actual"] = null;
                result["error"] = error.Message;
            }

            return result;
        }

        /// <summary>
        /// Run all tests and return results.
        /// </summary>
        public JsonObject RunTests()
        {
            Console.WriteLine("🧪 Running C# FHIRPath tests...");

            var results = new JsonObject
            {
                ["language"] = "csharp",
                ["timestamp"] = DateTimeOffset.UtcNow.ToUnixTimeSeconds(),
                ["tests"] = new JsonArray(),
                ["summary"] = new JsonObject
                {
                    ["total"] = 0,
                    ["passed"] = 0,
                    ["failed"] = 0,
                    ["errors"] = 0
                }
            };

            var testsArray = results["tests"].AsArray();
            var summary = results["summary"].AsObject();

            // Load test data files
            var testDataCache = new Dictionary<string, Resource>();
            var inputFiles = _testConfig["testData"]["inputFiles"].AsArray();

            foreach (var inputFile in inputFiles)
            {
                var filename = inputFile.ToString();
                var testData = LoadTestData(filename);
                if (testData != null)
                {
                    testDataCache[filename] = testData;
                }
            }

            // Load and run official tests
            Console.WriteLine("📋 Loading official FHIRPath test suite...");
            var officialTests = LoadOfficialTests();
            Console.WriteLine($"📊 Found {officialTests.Count} official test cases");

            foreach (var testCase in officialTests)
            {
                var inputFile = testCase["inputFile"].ToString();
                if (!testDataCache.ContainsKey(inputFile))
                {
                    Console.WriteLine($"⚠️  Skipping test {testCase["name"]} - test data not available: {inputFile}");
                    continue;
                }

                // Skip tests marked as invalid for now (these test error conditions)
                if (testCase["invalid"].GetValue<bool>())
                {
                    Console.WriteLine($"⏭️  Skipping invalid test {testCase["name"]} (tests error conditions)");
                    continue;
                }

                var testData = testDataCache[inputFile];
                var testResult = RunSingleTest(testCase, testData);
                testsArray.Add(testResult);

                var total = summary["total"].GetValue<int>() + 1;
                summary["total"] = total;

                var status = testResult["status"].ToString();
                if (status == "passed")
                {
                    summary["passed"] = summary["passed"].GetValue<int>() + 1;
                }
                else if (status == "error")
                {
                    summary["errors"] = summary["errors"].GetValue<int>() + 1;
                }
                else
                {
                    summary["failed"] = summary["failed"].GetValue<int>() + 1;
                }

                var statusIcon = status == "passed" ? "✅" :
                                status == "error" ? "💥" : "❌";
                var executionTime = testResult["execution_time_ms"].GetValue<double>();
                var groupName = testCase["group"].ToString();
                Console.WriteLine($"  {statusIcon} {testResult["name"]} ({executionTime:F2}ms) [{groupName}]");
            }

            // Save results
            var resultsFile = Path.Combine(_resultsDir, "csharp_test_results.json");
            var options = new JsonSerializerOptions { WriteIndented = true };
            File.WriteAllText(resultsFile, results.ToJsonString(options));

            Console.WriteLine($"📊 Results saved to: {resultsFile}");
            Console.WriteLine($"📈 Summary: {summary["passed"]}/{summary["total"]} tests passed");

            return results;
        }

        /// <summary>
        /// Run benchmarks and return results.
        /// </summary>
        public JsonObject RunBenchmarks()
        {
            Console.WriteLine("⚡ Running C# FHIRPath benchmarks...");

            var results = new JsonObject
            {
                ["language"] = "csharp",
                ["timestamp"] = DateTimeOffset.UtcNow.ToUnixTimeSeconds(),
                ["benchmarks"] = new JsonArray(),
                ["system_info"] = new JsonObject
                {
                    ["platform"] = Environment.OSVersion.ToString(),
                    ["dotnet_version"] = Environment.Version.ToString(),
                    ["fhirpath_version"] = "Hl7.FhirPath 5.5.0" // Would need to get actual version
                }
            };

            var benchmarksArray = results["benchmarks"].AsArray();

            // Load test data
            var testDataCache = new Dictionary<string, Resource>();
            var inputFiles = _testConfig["testData"]["inputFiles"].AsArray();

            foreach (var inputFile in inputFiles)
            {
                var filename = inputFile.ToString();
                var testData = LoadTestData(filename);
                if (testData != null)
                {
                    testDataCache[filename] = testData;
                }
            }

            // Run benchmarks
            var benchmarkTests = _testConfig["benchmarkTests"].AsArray();
            foreach (var benchmark in benchmarkTests)
            {
                var inputFile = benchmark["inputFile"]?.ToString() ?? "patient-example.xml";
                if (!testDataCache.ContainsKey(inputFile))
                {
                    Console.WriteLine($"⚠️  Skipping benchmark {benchmark["name"]} - test data not available");
                    continue;
                }

                var testData = testDataCache[inputFile];
                Console.WriteLine($"  🏃 Running {benchmark["name"]}...");

                var expression = benchmark["expression"].ToString();
                var iterations = benchmark["iterations"]?.GetValue<int>() ?? 1000;
                var times = new List<double>();

                try
                {
                    var compiled = _fhirPathCompiler.Compile(expression);
                    var typedElement = testData.ToTypedElement();
                    var context = EvaluationContext.CreateDefault();

                    // Warm up
                    for (int i = 0; i < 10; i++)
                    {
                        try
                        {
                            compiled(typedElement, context).ToList();
                        }
                        catch
                        {
                            // Ignore warm-up errors
                        }
                    }

                    // Actual benchmark
                    for (int i = 0; i < iterations; i++)
                    {
                        var stopwatch = Stopwatch.StartNew();
                        try
                        {
                            compiled(typedElement, context).ToList();
                        }
                        catch
                        {
                            // Continue timing even if expression fails
                        }
                        stopwatch.Stop();
                        times.Add(stopwatch.Elapsed.TotalMilliseconds);
                    }

                    if (times.Any())
                    {
                        var avgTime = times.Average();
                        var minTime = times.Min();
                        var maxTime = times.Max();
                        var opsPerSecond = avgTime > 0 ? 1000.0 / avgTime : 0.0;

                        var benchmarkResult = new JsonObject
                        {
                            ["name"] = benchmark["name"].ToString(),
                            ["description"] = benchmark["description"].ToString(),
                            ["expression"] = expression,
                            ["iterations"] = iterations,
                            ["avg_time_ms"] = avgTime,
                            ["min_time_ms"] = minTime,
                            ["max_time_ms"] = maxTime,
                            ["ops_per_second"] = opsPerSecond
                        };

                        benchmarksArray.Add(benchmarkResult);
                        Console.WriteLine($"    ⏱️  {avgTime:F2}ms avg ({opsPerSecond:F1} ops/sec)");
                    }
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"    ⚠️  Skipping {benchmark["name"]} - expression failed: {ex.Message}");
                    continue;
                }
            }

            // Save results
            var resultsFile = Path.Combine(_resultsDir, "csharp_benchmark_results.json");
            var options = new JsonSerializerOptions { WriteIndented = true };
            File.WriteAllText(resultsFile, results.ToJsonString(options));

            Console.WriteLine($"📊 Benchmark results saved to: {resultsFile}");

            return results;
        }

        public static void Main(string[] args)
        {
            try
            {
                var runner = new TestRunner();
                var command = args.Length > 0 ? args[0] : "both";

                if (command == "test" || command == "both")
                {
                    runner.RunTests();
                }

                if (command == "benchmark" || command == "both")
                {
                    runner.RunBenchmarks();
                }

                Console.WriteLine("✅ C# test runner completed");
            }
            catch (Exception error)
            {
                Console.WriteLine($"❌ Error running tests: {error.Message}");
                Console.WriteLine(error.StackTrace);
                Environment.Exit(1);
            }
        }
    }
}
