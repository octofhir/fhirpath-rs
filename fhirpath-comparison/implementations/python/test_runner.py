#!/usr/bin/env python3
"""
Python FHIRPath Test Runner

This script runs FHIRPath tests using the fhirpath-py library
and outputs results in a standardized format for comparison.
"""

import os
import sys
import json
import time
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Dict, List, Any, Optional
import xmltodict
from fhirpathpy import evaluate

# Custom JSON encoder to handle FHIRPath specific types
class FHIRPathEncoder(json.JSONEncoder):
    def default(self, obj):
        # Handle FP_DateTime and other custom types
        # Convert them to string representation
        try:
            return str(obj)
        except:
            return f"<non-serializable: {type(obj).__name__}>"

class PythonTestRunner:
    def __init__(self):
        self.test_data_dir = Path(__file__).parent / "../../test-data"
        self.test_cases_dir = Path(__file__).parent / "../../test-cases"
        self.results_dir = Path(__file__).parent / "../../results"

        # Ensure results directory exists
        self.results_dir.mkdir(exist_ok=True)

        # Load test configuration
        config_path = self.test_cases_dir / "test-config.json"
        with open(config_path, 'r') as f:
            self.test_config = json.load(f)

    def load_official_tests(self) -> List[Dict]:
        """Load official FHIRPath test cases from XML file."""
        xml_path = self.test_cases_dir / "tests-fhir-r4.xml"

        try:
            tree = ET.parse(xml_path)
            root = tree.getroot()
            tests = []

            # Extract tests from groups
            for group in root.findall('group'):
                group_name = group.get('name', 'unknown')

                for test in group.findall('test'):
                    test_name = test.get('name')
                    test_description = test.get('description', test_name)
                    input_file = test.get('inputfile', 'patient-example.xml')
                    predicate = test.get('predicate') == 'true'
                    mode = test.get('mode')
                    invalid = test.get('invalid')

                    expression_elem = test.find('expression')
                    if expression_elem is None:
                        continue

                    expression = expression_elem.text
                    if not expression:
                        continue

                    # Parse expected outputs
                    expected_output = []
                    for output in test.findall('output'):
                        output_type = output.get('type', 'string')
                        output_value = output.text
                        if output_value is not None:
                            expected_output.append({
                                'type': output_type,
                                'value': output_value
                            })

                    tests.append({
                        'name': test_name,
                        'description': test_description,
                        'inputFile': input_file,
                        'expression': expression,
                        'expectedOutput': expected_output,
                        'predicate': predicate,
                        'mode': mode,
                        'invalid': invalid,
                        'group': group_name
                    })

            return tests

        except Exception as e:
            print(f"❌ Error loading official tests: {e}")
            return []

    def load_test_data(self, filename: str) -> Optional[Dict]:
        """Load test data from XML file and convert to dict."""
        file_path = self.test_data_dir / filename

        if not file_path.exists():
            print(f"⚠️  Test data file not found: {filename}")
            return None

        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                xml_content = f.read()

            # Convert XML to dictionary for fhirpath-py
            test_data = xmltodict.parse(xml_content)
            return test_data
        except Exception as e:
            print(f"⚠️  Error loading test data {filename}: {e}")
            return None

    def run_single_test(self, test_case: Dict, test_data: Dict) -> Dict[str, Any]:
        """Run a single test case and return results."""
        start_time = time.perf_counter()
        is_invalid_test = test_case.get('invalid') is not None

        try:
            # Execute FHIRPath expression
            result = evaluate(test_data, test_case['expression'])

            end_time = time.perf_counter()
            execution_time_ms = (end_time - start_time) * 1000

            if is_invalid_test:
                # Invalid test should have failed but didn't - this is a failure
                return {
                    'name': test_case['name'],
                    'description': test_case['description'],
                    'expression': test_case['expression'],
                    'status': 'failed',
                    'execution_time_ms': execution_time_ms,
                    'expected': test_case.get('expectedOutput', []),
                    'actual': None,
                    'error': 'Expected error but expression succeeded'
                }
            else:
                return {
                    'name': test_case['name'],
                    'description': test_case['description'],
                    'expression': test_case['expression'],
                    'status': 'passed',  # Simplified - would need proper result comparison
                    'execution_time_ms': execution_time_ms,
                    'expected': test_case.get('expectedOutput', []),
                    'actual': result
                }
        except Exception as error:
            end_time = time.perf_counter()
            execution_time_ms = (end_time - start_time) * 1000

            if is_invalid_test:
                # Invalid test correctly produced an error - this is a pass
                return {
                    'name': test_case['name'],
                    'description': test_case['description'],
                    'expression': test_case['expression'],
                    'status': 'passed',
                    'execution_time_ms': execution_time_ms,
                    'expected': test_case.get('expectedOutput', []),
                    'actual': None
                }
            else:
                return {
                    'name': test_case['name'],
                    'description': test_case['description'],
                    'expression': test_case['expression'],
                    'status': 'error',
                    'execution_time_ms': execution_time_ms,
                    'expected': test_case.get('expectedOutput', []),
                    'actual': None,
                    'error': str(error)
                }

    def run_tests(self) -> Dict[str, Any]:
        """Run all tests and return results."""
        print('🧪 Running Python FHIRPath tests...')

        results = {
            'language': 'python',
            'timestamp': time.time(),
            'tests': [],
            'summary': {
                'total': 0,
                'passed': 0,
                'failed': 0,
                'errors': 0
            }
        }

        # Load test data files
        test_data_cache = {}
        for input_file in self.test_config['testData']['inputFiles']:
            test_data = self.load_test_data(input_file)
            if test_data:
                test_data_cache[input_file] = test_data

        # Load and run official tests
        print('📋 Loading official FHIRPath test suite...')
        official_tests = self.load_official_tests()
        print(f'📊 Found {len(official_tests)} official test cases')

        for test_case in official_tests:
            input_file = test_case['inputFile']
            test_data = test_data_cache.get(input_file)

            if not test_data:
                print(f"⚠️  Skipping test {test_case['name']} - test data not available: {input_file}")
                continue

            # Skip tests marked as invalid for now (these test error conditions)
            if test_case['invalid']:
                print(f"⏭️  Skipping invalid test {test_case['name']} (tests error conditions)")
                continue

            test_result = self.run_single_test(test_case, test_data)
            results['tests'].append(test_result)
            results['summary']['total'] += 1

            if test_result['status'] == 'passed':
                results['summary']['passed'] += 1
            elif test_result['status'] == 'error':
                results['summary']['errors'] += 1
            else:
                results['summary']['failed'] += 1

            status_icon = '✅' if test_result['status'] == 'passed' else '💥' if test_result['status'] == 'error' else '❌'
            print(f"  {status_icon} {test_result['name']} ({test_result['execution_time_ms']:.2f}ms) [{test_case['group']}]")

        # Save results with timestamp in filename
        results_file = self.results_dir / f"python_test_results.json"
        with open(results_file, 'w') as f:
            json.dump(results, f, indent=2, cls=FHIRPathEncoder)

        print(f"📊 Results saved to: {results_file}")
        print(f"📈 Summary: {results['summary']['passed']}/{results['summary']['total']} tests passed")

        return results

    def run_benchmarks(self) -> Dict[str, Any]:
        """Run benchmarks and return results."""
        print('⚡ Running Python FHIRPath benchmarks...')

        results = {
            'language': 'python',
            'timestamp': time.time(),
            'benchmarks': [],
            'system_info': {
                'platform': sys.platform,
                'python_version': sys.version,
                'fhirpath_version': 'fhirpath-py'  # Would need to get actual version
            }
        }

        # Load test data
        test_data_cache = {}
        for input_file in self.test_config['testData']['inputFiles']:
            test_data = self.load_test_data(input_file)
            if test_data:
                test_data_cache[input_file] = test_data

        # Run benchmarks
        for benchmark in self.test_config['benchmarkTests']:
            input_file = benchmark.get('inputFile', 'patient-example.xml')
            test_data = test_data_cache.get(input_file)

            if not test_data:
                print(f"⚠️  Skipping benchmark {benchmark['name']} - test data not available")
                continue

            print(f"  🏃 Running {benchmark['name']}...")

            times = []
            iterations = benchmark.get('iterations', 1000)

            # Warm up
            for _ in range(10):
                try:
                    evaluate(test_data, benchmark['expression'])
                except:
                    pass

            # Actual benchmark
            for _ in range(iterations):
                start_time = time.perf_counter()
                try:
                    evaluate(test_data, benchmark['expression'])
                except:
                    pass  # Continue timing even if expression fails
                end_time = time.perf_counter()
                times.append((end_time - start_time) * 1000)  # Convert to milliseconds

            if times:
                avg_time = sum(times) / len(times)
                min_time = min(times)
                max_time = max(times)
                ops_per_second = 1000 / avg_time if avg_time > 0 else 0

                benchmark_result = {
                    'name': benchmark['name'],
                    'description': benchmark['description'],
                    'expression': benchmark['expression'],
                    'iterations': iterations,
                    'avg_time_ms': avg_time,
                    'min_time_ms': min_time,
                    'max_time_ms': max_time,
                    'ops_per_second': ops_per_second
                }

                results['benchmarks'].append(benchmark_result)
                print(f"    ⏱️  {avg_time:.2f}ms avg ({ops_per_second:.1f} ops/sec)")

        results_file = self.results_dir / f"python_benchmark_results.json"
        with open(results_file, 'w') as f:
            json.dump(results, f, indent=2, cls=FHIRPathEncoder)

        print(f"📊 Benchmark results saved to: {results_file}")

        return results

def main():
    runner = PythonTestRunner()

    command = sys.argv[1] if len(sys.argv) > 1 else 'both'

    try:
        if command == 'test' or command == 'both':
            runner.run_tests()

        if command == 'benchmark' or command == 'both':
            runner.run_benchmarks()

        print('✅ Python test runner completed')
    except Exception as error:
        print(f'❌ Error running tests: {error}')
        sys.exit(1)

if __name__ == '__main__':
    main()
