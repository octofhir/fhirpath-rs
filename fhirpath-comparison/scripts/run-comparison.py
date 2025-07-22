#!/usr/bin/env python3
"""
FHIRPath Library Comparison Runner

This script orchestrates the execution of FHIRPath tests and benchmarks
across different programming language implementations.
"""

import os
import sys
import json
import subprocess
import time
import glob
from pathlib import Path
from typing import Dict, List, Any
import argparse

class ComparisonRunner:
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.implementations_dir = project_root / "implementations"
        self.test_cases_dir = project_root / "test-cases"
        self.results_dir = project_root / "results"
        self.results_dir.mkdir(exist_ok=True)

        # Load test configuration
        with open(self.test_cases_dir / "test-config.json", 'r') as f:
            self.test_config = json.load(f)

    def cleanup_old_results(self, language: str, result_type: str):
        """Remove old result files for a specific language and result type, keeping only the most recent."""
        pattern = f"{language}_{result_type}_*.json"
        result_files = list(self.results_dir.glob(pattern))

        if len(result_files) > 0:
            # Sort by modification time, newest first
            result_files.sort(key=lambda p: p.stat().st_mtime, reverse=True)

            # Remove all but the most recent file
            for old_file in result_files:
                try:
                    old_file.unlink()
                    print(f"🗑️  Removed old result file: {old_file.name}")
                except Exception as e:
                    print(f"⚠️  Could not remove {old_file.name}: {e}")

    def get_available_implementations(self) -> List[str]:
        """Get list of available language implementations."""
        implementations = []
        for impl_dir in self.implementations_dir.iterdir():
            if impl_dir.is_dir() and not impl_dir.name.startswith('.'):
                implementations.append(impl_dir.name)
        return implementations

    def setup_implementation(self, language: str) -> bool:
        """Set up dependencies for a specific language implementation."""
        impl_dir = self.implementations_dir / language
        if not impl_dir.exists():
            print(f"❌ Implementation directory not found: {language}")
            return False

        print(f"🔧 Setting up {language} implementation...")

        try:
            if language == "javascript":
                subprocess.run(["npm", "install"], cwd=impl_dir, check=True)
            elif language == "python":
                # Check if virtual environment exists, create if not
                venv_path = impl_dir / "venv"
                if not venv_path.exists():
                    print(f"Creating virtual environment at {venv_path}")
                    subprocess.run([sys.executable, "-m", "venv", "venv"], cwd=impl_dir, check=True)

                # Determine the Python executable in the virtual environment
                if sys.platform == "win32":
                    venv_python = venv_path / "Scripts" / "python.exe"
                else:
                    venv_python = venv_path / "bin" / "python"

                # Install requirements using the virtual environment Python
                subprocess.run([str(venv_python), "-m", "pip", "install", "-r", "requirements.txt"],
                             cwd=impl_dir, check=True)
            elif language == "java":
                subprocess.run(["mvn", "compile"], cwd=impl_dir, check=True)
            elif language == "csharp":
                subprocess.run(["dotnet", "restore"], cwd=impl_dir, check=True)
            elif language == "rust":
                subprocess.run(["cargo", "build"], cwd=impl_dir, check=True)
            elif language == "go":
                subprocess.run(["go", "mod", "tidy"], cwd=impl_dir, check=True)
                subprocess.run(["go", "build"], cwd=impl_dir, check=True)
            elif language == "clojure":
                subprocess.run(["clojure", "-P"], cwd=impl_dir, check=True)

            print(f"✅ {language} setup completed")
            return True

        except subprocess.CalledProcessError as e:
            print(f"❌ Failed to setup {language}: {e}")
            return False
        except FileNotFoundError:
            print(f"❌ Required tools not found for {language}")
            return False

    def run_tests(self, language: str) -> Dict[str, Any]:
        """Run tests for a specific language implementation."""
        print(f"🧪 Running tests for {language}...")

        # Clean up old test result files for this language
        self.cleanup_old_results(language, "test_results")

        impl_dir = self.implementations_dir / language

        try:
            if language == "javascript":
                result = subprocess.run(
                    ["node", "test-runner.js", "test"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            elif language == "python":
                # Check for virtual environment
                venv_python = None
                venv_dirs = ["venv", ".venv", "env", ".env"]
                for venv_dir in venv_dirs:
                    venv_path = impl_dir / venv_dir
                    if venv_path.exists():
                        if sys.platform == "win32":
                            venv_python = venv_path / "Scripts" / "python.exe"
                        else:
                            venv_python = venv_path / "bin" / "python"
                        if venv_python.exists():
                            break
                        else:
                            venv_python = None

                # Use virtual environment Python if found, otherwise use sys.executable
                python_executable = str(venv_python) if venv_python else sys.executable
                print(f"Using Python interpreter: {python_executable}")

                result = subprocess.run(
                    [python_executable, "test_runner.py", "test"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            elif language == "java":
                # Compile and run Java test runner
                subprocess.run(["mvn", "compile"], cwd=impl_dir, check=True)
                result = subprocess.run(
                    ["mvn", "exec:java", "-Dexec.mainClass=org.fhirpath.comparison.TestRunner", "-Dexec.args=test"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            elif language == "csharp":
                result = subprocess.run(
                    ["dotnet", "run", "--", "test"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            elif language == "rust":
                result = subprocess.run(
                    ["cargo", "run", "--", "test"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            elif language == "go":
                result = subprocess.run(
                    ["go", "run", "main.go", "test"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            elif language == "clojure":
                result = subprocess.run(
                    ["clojure", "-M", "-m", "test-runner"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            else:
                print(f"❌ Unknown language: {language}")
                return self._create_error_result(language, f"Unknown language: {language}")

            # Try to find and parse the results file
            results_pattern = f"{language}_test_results*.json"
            results_files = list(self.results_dir.glob(results_pattern))

            if results_files:
                # Get the most recent results file
                latest_results = max(results_files, key=lambda p: p.stat().st_mtime)
                with open(latest_results, 'r') as f:
                    return json.load(f)
            else:
                # Fallback: parse output for basic info
                print(f"⚠️  No results file found for {language}, using basic parsing")
                return self._parse_output_for_results(language, result.stdout)

        except subprocess.CalledProcessError as e:
            print(f"❌ Failed to run tests for {language}: {e}")
            print(f"stdout: {e.stdout}")
            print(f"stderr: {e.stderr}")
            return self._create_error_result(language, str(e))
        except Exception as e:
            print(f"❌ Error running tests for {language}: {e}")
            return self._create_error_result(language, str(e))

    def run_benchmarks(self, language: str) -> Dict[str, Any]:
        """Run benchmarks for a specific language implementation."""
        print(f"⚡ Running benchmarks for {language}...")

        # Clean up old benchmark result files for this language
        self.cleanup_old_results(language, "benchmark_results")

        impl_dir = self.implementations_dir / language

        try:
            if language == "javascript":
                result = subprocess.run(
                    ["node", "test-runner.js", "benchmark"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            elif language == "python":
                # Check for virtual environment
                venv_python = None
                venv_dirs = ["venv", ".venv", "env", ".env"]
                for venv_dir in venv_dirs:
                    venv_path = impl_dir / venv_dir
                    if venv_path.exists():
                        if sys.platform == "win32":
                            venv_python = venv_path / "Scripts" / "python.exe"
                        else:
                            venv_python = venv_path / "bin" / "python"
                        if venv_python.exists():
                            break
                        else:
                            venv_python = None

                # Use virtual environment Python if found, otherwise use sys.executable
                python_executable = str(venv_python) if venv_python else sys.executable
                print(f"Using Python interpreter for benchmarks: {python_executable}")

                result = subprocess.run(
                    [python_executable, "test_runner.py", "benchmark"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            elif language == "java":
                result = subprocess.run(
                    ["mvn", "exec:java", "-Dexec.mainClass=org.fhirpath.comparison.TestRunner", "-Dexec.args=benchmark"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            elif language == "csharp":
                result = subprocess.run(
                    ["dotnet", "run", "--", "benchmark"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            elif language == "rust":
                result = subprocess.run(
                    ["cargo", "run", "--", "benchmark"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            elif language == "go":
                result = subprocess.run(
                    ["go", "run", "main.go", "benchmark"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            elif language == "clojure":
                result = subprocess.run(
                    ["clojure", "-M", "-m", "test-runner", "benchmark"],
                    cwd=impl_dir,
                    capture_output=True,
                    text=True,
                    check=True
                )
            else:
                print(f"❌ Unknown language: {language}")
                return self._create_error_benchmark_result(language, f"Unknown language: {language}")

            # Try to find and parse the results file
            results_pattern = f"{language}_benchmark_results.json"
            results_files = list(self.results_dir.glob(results_pattern))

            if results_files:
                # Get the most recent results file
                latest_results = max(results_files, key=lambda p: p.stat().st_mtime)
                with open(latest_results, 'r') as f:
                    return json.load(f)
            else:
                # Fallback: create basic benchmark result
                print(f"⚠️  No benchmark results file found for {language}")
                return self._create_error_benchmark_result(language, "No results file generated")

        except subprocess.CalledProcessError as e:
            print(f"❌ Failed to run benchmarks for {language}: {e}")
            print(f"stdout: {e.stdout}")
            print(f"stderr: {e.stderr}")
            return self._create_error_benchmark_result(language, str(e))
        except Exception as e:
            print(f"❌ Error running benchmarks for {language}: {e}")
            return self._create_error_benchmark_result(language, str(e))

    def _create_error_result(self, language: str, error_message: str) -> Dict[str, Any]:
        """Create an error result structure for failed test runs."""
        return {
            "language": language,
            "timestamp": time.time(),
            "tests": [],
            "summary": {
                "total": 0,
                "passed": 0,
                "failed": 0,
                "errors": 1
            },
            "error": error_message
        }

    def _create_error_benchmark_result(self, language: str, error_message: str) -> Dict[str, Any]:
        """Create an error result structure for failed benchmark runs."""
        return {
            "language": language,
            "timestamp": time.time(),
            "benchmarks": [],
            "system_info": {
                "platform": sys.platform,
                "error": error_message
            },
            "error": error_message
        }

    def _parse_output_for_results(self, language: str, output: str) -> Dict[str, Any]:
        """Parse test runner output to extract basic results when results file is not available."""
        # Basic parsing - look for common patterns in output
        lines = output.split('\n')

        passed_count = 0
        total_count = 0

        for line in lines:
            if '✅' in line or 'passed' in line.lower():
                passed_count += 1
                total_count += 1
            elif '❌' in line or 'failed' in line.lower() or 'error' in line.lower():
                total_count += 1

        return {
            "language": language,
            "timestamp": time.time(),
            "tests": [],
            "summary": {
                "total": total_count,
                "passed": passed_count,
                "failed": total_count - passed_count,
                "errors": 0
            },
            "note": "Results parsed from output - detailed test data not available"
        }

    def generate_report(self, test_results: List[Dict], benchmark_results: List[Dict]):
        """Generate comparison report from results."""
        report = {
            "comparison_report": {
                "timestamp": time.time(),
                "test_results": test_results,
                "benchmark_results": benchmark_results,
                "summary": {
                    "languages_tested": len(test_results),
                    "total_tests": sum(r["summary"]["total"] for r in test_results),
                    "total_benchmarks": sum(len(r["benchmarks"]) for r in benchmark_results)
                }
            }
        }

        # Save detailed results
        report_file = self.results_dir / "comparison_report.json"
        with open(report_file, 'w') as f:
            json.dump(report, f, indent=2)

        print(f"📊 Report saved to: {report_file}")

        # Print summary
        print("\n" + "="*60)
        print("FHIRPATH LIBRARY COMPARISON SUMMARY")
        print("="*60)

        for result in test_results:
            lang = result["language"]
            summary = result["summary"]
            print(f"{lang:12} | Tests: {summary['passed']:3}/{summary['total']:3} passed")

        print("\nBenchmark Results (avg time in ms):")
        for result in benchmark_results:
            lang = result["language"]
            print(f"\n{lang}:")
            for bench in result["benchmarks"]:
                print(f"  {bench['name']:25} | {bench['avg_time_ms']:6.2f} ms")

def main():
    parser = argparse.ArgumentParser(description="Run FHIRPath library comparison")
    parser.add_argument("--languages", nargs="+",
                       help="Specific languages to test (default: all available)")
    parser.add_argument("--setup-only", action="store_true",
                       help="Only setup dependencies, don't run tests")
    parser.add_argument("--tests-only", action="store_true",
                       help="Only run tests, skip benchmarks")
    parser.add_argument("--benchmarks-only", action="store_true",
                       help="Only run benchmarks, skip tests")

    args = parser.parse_args()

    # Find project root
    script_dir = Path(__file__).parent
    project_root = script_dir.parent

    runner = ComparisonRunner(project_root)

    # Determine which languages to test
    available_languages = runner.get_available_implementations()
    languages_to_test = args.languages if args.languages else available_languages

    print(f"🚀 Starting FHIRPath library comparison")
    print(f"📁 Project root: {project_root}")
    print(f"🔍 Available implementations: {', '.join(available_languages)}")
    print(f"🎯 Testing languages: {', '.join(languages_to_test)}")

    # Setup implementations
    setup_success = {}
    for language in languages_to_test:
        setup_success[language] = runner.setup_implementation(language)

    if args.setup_only:
        print("✅ Setup completed")
        return

    # Run tests and benchmarks
    test_results = []
    benchmark_results = []

    for language in languages_to_test:
        if not setup_success[language]:
            print(f"⏭️  Skipping {language} due to setup failure")
            continue

        if not args.benchmarks_only:
            test_result = runner.run_tests(language)
            test_results.append(test_result)

        if not args.tests_only:
            benchmark_result = runner.run_benchmarks(language)
            benchmark_results.append(benchmark_result)

    # Generate report
    if test_results or benchmark_results:
        runner.generate_report(test_results, benchmark_results)

    print("\n🎉 Comparison completed!")

if __name__ == "__main__":
    main()
