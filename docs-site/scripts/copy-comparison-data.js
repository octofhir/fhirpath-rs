/**
 * Script to copy comparison data from fhirpath-comparison/results to docs-site/public
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// Get current file directory
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Source and destination directories
const sourceDir = path.resolve(__dirname, '../../fhirpath-comparison/results');
const destDir = path.resolve(__dirname, '../public');

// Create destination directory if it doesn't exist
if (!fs.existsSync(destDir)) {
  fs.mkdirSync(destDir, { recursive: true });
  console.log(`Created directory: ${destDir}`);
}

// Files to copy
const filesToCopy = [
  'comparison_report.json',
  'javascript_test_results.json',
  'javascript_benchmark_results.json',
  'python_test_results.json',
  'python_benchmark_results.json',
  'java_test_results.json',
  'java_benchmark_results.json',
  'csharp_test_results.json',
  'csharp_benchmark_results.json',
  'rust_test_results.json',
  'rust_benchmark_results.json',
  'go_test_results.json',
  'go_benchmark_results.json'
];

// Copy files
let copiedCount = 0;
let errorCount = 0;

filesToCopy.forEach(file => {
  const sourcePath = path.join(sourceDir, file);
  const destPath = path.join(destDir, file);

  try {
    if (fs.existsSync(sourcePath)) {
      fs.copyFileSync(sourcePath, destPath);
      console.log(`Copied: ${file}`);
      copiedCount++;
    } else {
      console.log(`Skipped (not found): ${file}`);
    }
  } catch (error) {
    console.error(`Error copying ${file}: ${error.message}`);
    errorCount++;
  }
});

console.log(`\nCopy complete: ${copiedCount} files copied, ${errorCount} errors`);
