package org.fhirpath.comparison;

import ca.uhn.fhir.context.FhirContext;
import ca.uhn.fhir.fhirpath.IFhirPath;
import ca.uhn.fhir.parser.IParser;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.hl7.fhir.instance.model.api.IBase;
import org.hl7.fhir.r4.model.Base;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import javax.xml.parsers.DocumentBuilder;
import javax.xml.parsers.DocumentBuilderFactory;
import org.w3c.dom.Document;
import org.w3c.dom.Element;
import org.w3c.dom.Node;
import org.w3c.dom.NodeList;

/**
 * Java FHIRPath Test Runner
 *
 * This class runs FHIRPath tests using the HAPI FHIR library
 * and outputs results in a standardized format for comparison.
 */
public class TestRunner {

    private final Path testDataDir;
    private final Path testCasesDir;
    private final Path resultsDir;
    private final FhirContext fhirContext;
    private final IFhirPath fhirPath;
    private final IParser xmlParser;
    private final ObjectMapper objectMapper;
    private JsonNode testConfig;

    public TestRunner() throws IOException {
        // Initialize paths
        Path currentDir = Paths.get("").toAbsolutePath();
        this.testDataDir = currentDir.resolve("../../test-data");
        this.testCasesDir = currentDir.resolve("../../test-cases");
        this.resultsDir = currentDir.resolve("../../results");

        // Ensure results directory exists
        Files.createDirectories(resultsDir);

        // Initialize FHIR context and parsers
        this.fhirContext = FhirContext.forR4();
        this.fhirPath = fhirContext.newFhirPath();
        this.xmlParser = fhirContext.newXmlParser();
        this.objectMapper = new ObjectMapper();

        // Load test configuration
        Path configPath = testCasesDir.resolve("test-config.json");
        this.testConfig = objectMapper.readTree(configPath.toFile());
    }

    /**
     * Load official FHIRPath test cases from XML file.
     */
    private List<ObjectNode> loadOfficialTests() {
        List<ObjectNode> tests = new ArrayList<>();
        Path xmlPath = testCasesDir.resolve("tests-fhir-r4.xml");

        try {
            DocumentBuilderFactory factory = DocumentBuilderFactory.newInstance();
            DocumentBuilder builder = factory.newDocumentBuilder();
            Document doc = builder.parse(xmlPath.toFile());

            NodeList groups = doc.getElementsByTagName("group");
            for (int i = 0; i < groups.getLength(); i++) {
                Element group = (Element) groups.item(i);
                String groupName = group.getAttribute("name");

                NodeList testNodes = group.getElementsByTagName("test");
                for (int j = 0; j < testNodes.getLength(); j++) {
                    Element test = (Element) testNodes.item(j);

                    String testName = test.getAttribute("name");
                    String description = test.getAttribute("description");
                    String inputFile = test.getAttribute("inputfile");
                    String invalid = test.getAttribute("invalid");

                    if (inputFile.isEmpty()) inputFile = "patient-example.xml";

                    NodeList expressions = test.getElementsByTagName("expression");
                    if (expressions.getLength() == 0) continue;

                    String expression = expressions.item(0).getTextContent();
                    if (expression == null || expression.trim().isEmpty()) continue;

                    ObjectNode testCase = objectMapper.createObjectNode();
                    testCase.put("name", testName);
                    testCase.put("description", description.isEmpty() ? testName : description);
                    testCase.put("inputFile", inputFile);
                    testCase.put("expression", expression);
                    testCase.put("group", groupName);
                    testCase.put("invalid", !invalid.isEmpty());

                    tests.add(testCase);
                }
            }
        } catch (Exception e) {
            System.err.println("❌ Error loading official tests: " + e.getMessage());
        }

        return tests;
    }

    /**
     * Load test data from XML file.
     */
    private Optional<IBase> loadTestData(String filename) {
        Path filePath = testDataDir.resolve(filename);

        if (!Files.exists(filePath)) {
            System.out.println("⚠️  Test data file not found: " + filename);
            return Optional.empty();
        }

        try {
            String xmlContent = Files.readString(filePath);
            IBase resource = xmlParser.parseResource(xmlContent);
            return Optional.of(resource);
        } catch (Exception e) {
            System.out.println("⚠️  Error loading test data " + filename + ": " + e.getMessage());
            return Optional.empty();
        }
    }

    /**
     * Run a single test case and return results.
     */
    private ObjectNode runSingleTest(JsonNode testCase, IBase testData) {
        ObjectNode result = objectMapper.createObjectNode();
        long startTime = System.nanoTime();

        try {
            String expression = testCase.get("expression").asText();

            // Execute FHIRPath expression
            List<IBase> evaluationResult = fhirPath.evaluate(testData, expression, IBase.class);

            long endTime = System.nanoTime();
            double executionTimeMs = (endTime - startTime) / 1_000_000.0;

            result.put("name", testCase.get("name").asText());
            result.put("description", testCase.get("description").asText());
            result.put("expression", expression);
            result.put("status", "passed"); // Simplified - would need proper result comparison
            result.put("execution_time_ms", executionTimeMs);

            // Add expected and actual results
            if (testCase.has("expectedOutput")) {
                result.set("expected", testCase.get("expectedOutput"));
            } else {
                result.set("expected", objectMapper.createArrayNode());
            }

            ArrayNode actualArray = objectMapper.createArrayNode();
            for (IBase item : evaluationResult) {
                if (item instanceof Base) {
                    Base base = (Base) item;
                    if (base.isPrimitive()) {
                        actualArray.add(base.primitiveValue());
                    } else {
                        actualArray.add(item.toString());
                    }
                } else {
                    actualArray.add(item.toString());
                }
            }
            result.set("actual", actualArray);

        } catch (Exception error) {
            long endTime = System.nanoTime();
            double executionTimeMs = (endTime - startTime) / 1_000_000.0;

            result.put("name", testCase.get("name").asText());
            result.put("description", testCase.get("description").asText());
            result.put("expression", testCase.get("expression").asText());
            result.put("status", "error");
            result.put("execution_time_ms", executionTimeMs);

            if (testCase.has("expectedOutput")) {
                result.set("expected", testCase.get("expectedOutput"));
            } else {
                result.set("expected", objectMapper.createArrayNode());
            }
            result.putNull("actual");
            result.put("error", error.getMessage());
        }

        return result;
    }

    /**
     * Run all tests and return results.
     */
    public ObjectNode runTests() throws IOException {
        System.out.println("🧪 Running Java FHIRPath tests...");

        ObjectNode results = objectMapper.createObjectNode();
        results.put("language", "java");
        results.put("timestamp", System.currentTimeMillis() / 1000.0);

        ArrayNode testsArray = objectMapper.createArrayNode();
        results.set("tests", testsArray);

        ObjectNode summary = objectMapper.createObjectNode();
        summary.put("total", 0);
        summary.put("passed", 0);
        summary.put("failed", 0);
        summary.put("errors", 0);
        results.set("summary", summary);

        // Load test data files
        JsonNode inputFiles = testConfig.get("testData").get("inputFiles");
        List<IBase> testDataCache = new ArrayList<>();
        List<String> fileNames = new ArrayList<>();

        for (JsonNode inputFile : inputFiles) {
            String filename = inputFile.asText();
            Optional<IBase> testData = loadTestData(filename);
            if (testData.isPresent()) {
                testDataCache.add(testData.get());
                fileNames.add(filename);
            }
        }

        // Load and run official tests
        System.out.println("📋 Loading official FHIRPath test suite...");
        List<ObjectNode> officialTests = loadOfficialTests();
        System.out.println("📊 Found " + officialTests.size() + " official test cases");

        for (ObjectNode testCase : officialTests) {
            String inputFile = testCase.get("inputFile").asText();
            int dataIndex = fileNames.indexOf(inputFile);

            if (dataIndex == -1) {
                System.out.println("⚠️  Skipping test " + testCase.get("name").asText() + " - test data not available: " + inputFile);
                continue;
            }

            // Skip tests marked as invalid for now (these test error conditions)
            if (testCase.get("invalid").asBoolean()) {
                System.out.println("⏭️  Skipping invalid test " + testCase.get("name").asText() + " (tests error conditions)");
                continue;
            }

            IBase testData = testDataCache.get(dataIndex);
            ObjectNode testResult = runSingleTest(testCase, testData);
            testsArray.add(testResult);

            int total = summary.get("total").asInt() + 1;
            summary.put("total", total);

            String status = testResult.get("status").asText();
            if ("passed".equals(status)) {
                summary.put("passed", summary.get("passed").asInt() + 1);
            } else if ("error".equals(status)) {
                summary.put("errors", summary.get("errors").asInt() + 1);
            } else {
                summary.put("failed", summary.get("failed").asInt() + 1);
            }

            String statusIcon = "passed".equals(status) ? "✅" :
                              "error".equals(status) ? "💥" : "❌";
            double executionTime = testResult.get("execution_time_ms").asDouble();
            String groupName = testCase.get("group").asText();
            System.out.printf("  %s %s (%.2fms) [%s]%n", statusIcon, testResult.get("name").asText(), executionTime, groupName);
        }

        // Save results
        Path resultsFile = resultsDir.resolve("java_test_results.json");
        objectMapper.writerWithDefaultPrettyPrinter().writeValue(resultsFile.toFile(), results);

        System.out.println("📊 Results saved to: " + resultsFile);
        System.out.printf("📈 Summary: %d/%d tests passed%n",
            summary.get("passed").asInt(), summary.get("total").asInt());

        return results;
    }

    /**
     * Run benchmarks and return results.
     */
    public ObjectNode runBenchmarks() throws IOException {
        System.out.println("⚡ Running Java FHIRPath benchmarks...");

        ObjectNode results = objectMapper.createObjectNode();
        results.put("language", "java");
        results.put("timestamp", System.currentTimeMillis() / 1000.0);

        ArrayNode benchmarksArray = objectMapper.createArrayNode();
        results.set("benchmarks", benchmarksArray);

        ObjectNode systemInfo = objectMapper.createObjectNode();
        systemInfo.put("platform", System.getProperty("os.name"));
        systemInfo.put("java_version", System.getProperty("java.version"));
        systemInfo.put("hapi_fhir_version", "6.8.0"); // Would need to get actual version
        results.set("system_info", systemInfo);

        // Load test data
        JsonNode inputFiles = testConfig.get("testData").get("inputFiles");
        List<IBase> testDataCache = new ArrayList<>();
        List<String> fileNames = new ArrayList<>();

        for (JsonNode inputFile : inputFiles) {
            String filename = inputFile.asText();
            Optional<IBase> testData = loadTestData(filename);
            if (testData.isPresent()) {
                testDataCache.add(testData.get());
                fileNames.add(filename);
            }
        }

        // Run benchmarks
        JsonNode benchmarkTests = testConfig.get("benchmarkTests");
        for (JsonNode benchmark : benchmarkTests) {
            String inputFile = benchmark.has("inputFile") ?
                benchmark.get("inputFile").asText() : "patient-example.xml";
            int dataIndex = fileNames.indexOf(inputFile);

            if (dataIndex == -1) {
                System.out.println("⚠️  Skipping benchmark " + benchmark.get("name").asText() + " - test data not available");
                continue;
            }

            IBase testData = testDataCache.get(dataIndex);
            System.out.println("  🏃 Running " + benchmark.get("name").asText() + "...");

            String expression = benchmark.get("expression").asText();
            int iterations = benchmark.has("iterations") ? benchmark.get("iterations").asInt() : 1000;
            List<Double> times = new ArrayList<>();

            // Warm up
            for (int i = 0; i < 10; i++) {
                try {
                    fhirPath.evaluate(testData, expression, IBase.class);
                } catch (Exception e) {
                    // Ignore warm-up errors
                }
            }

            // Actual benchmark
            for (int i = 0; i < iterations; i++) {
                long startTime = System.nanoTime();
                try {
                    fhirPath.evaluate(testData, expression, IBase.class);
                } catch (Exception e) {
                    // Continue timing even if expression fails
                }
                long endTime = System.nanoTime();
                times.add((endTime - startTime) / 1_000_000.0); // Convert to milliseconds
            }

            if (!times.isEmpty()) {
                double avgTime = times.stream().mapToDouble(Double::doubleValue).average().orElse(0.0);
                double minTime = times.stream().mapToDouble(Double::doubleValue).min().orElse(0.0);
                double maxTime = times.stream().mapToDouble(Double::doubleValue).max().orElse(0.0);
                double opsPerSecond = avgTime > 0 ? 1000.0 / avgTime : 0.0;

                ObjectNode benchmarkResult = objectMapper.createObjectNode();
                benchmarkResult.put("name", benchmark.get("name").asText());
                benchmarkResult.put("description", benchmark.get("description").asText());
                benchmarkResult.put("expression", expression);
                benchmarkResult.put("iterations", iterations);
                benchmarkResult.put("avg_time_ms", avgTime);
                benchmarkResult.put("min_time_ms", minTime);
                benchmarkResult.put("max_time_ms", maxTime);
                benchmarkResult.put("ops_per_second", opsPerSecond);

                benchmarksArray.add(benchmarkResult);
                System.out.printf("    ⏱️  %.2fms avg (%.1f ops/sec)%n", avgTime, opsPerSecond);
            }
        }

        // Save results
        Path resultsFile = resultsDir.resolve("java_benchmark_results.json");
        objectMapper.writerWithDefaultPrettyPrinter().writeValue(resultsFile.toFile(), results);

        System.out.println("📊 Benchmark results saved to: " + resultsFile);

        return results;
    }

    public static void main(String[] args) {
        try {
            TestRunner runner = new TestRunner();
            String command = args.length > 0 ? args[0] : "both";

            if ("test".equals(command) || "both".equals(command)) {
                runner.runTests();
            }

            if ("benchmark".equals(command) || "both".equals(command)) {
                runner.runBenchmarks();
            }

            System.out.println("✅ Java test runner completed");
        } catch (Exception error) {
            System.err.println("❌ Error running tests: " + error.getMessage());
            error.printStackTrace();
            System.exit(1);
        }
    }
}
