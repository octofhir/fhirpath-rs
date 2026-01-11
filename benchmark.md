# FHIRPath-rs Benchmark Results

Generated on: 2026-01-11 21:22:32 UTC

## Overview

This benchmark suite measures the performance of FHIRPath-rs library across three main operations:
- **Tokenization**: Converting FHIRPath expressions into tokens
- **Parsing**: Building AST from tokens
- **Evaluation**: Executing expressions against FHIR data

## Environment
- Tool: fhirpath-bench v0.4.35
- OS/Arch: macos / aarch64
- CPU cores: 8
- FHIR Schema: R5

## Expression Categories

### Simple Expressions
Basic field access and simple operations:
- `Patient.active`
- `Patient.name.family`
- `Patient.birthDate`
- `Patient.gender`
- `true`
- `false`
- `1 + 2`
- `Patient.name.count()`

### Medium Expressions
Filtered queries and basic functions:
- `Patient.name.where(use = 'official').family`
- `Patient.telecom.where(system = 'phone').value`
- `Patient.extension.where(url = 'http://example.org').value`
- `Patient.contact.name.family`
- `Patient.birthDate > @1980-01-01`
- `Patient.name.family.substring(0, 3)`
- `Patient.telecom.exists(system = 'email')`
- `Patient.identifier.where(system = 'http://example.org/mrn').value`

### Complex Expressions
Bundle operations and resolve() calls:
- `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`
- `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`
- `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`
- `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`
- `Bundle.entry.resource.descendants().where($this is Reference).reference`
- `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`
- `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`

## Benchmark Results

```
## Tokenization Benchmarks
  - `Patient.active`: 347.3K ops/sec
  - `Patient.name.family`: 226.2K ops/sec
  - `Patient.birthDate`: 356.3K ops/sec
  - `Patient.gender`: 371.8K ops/sec
  - `true`: 580.8K ops/sec
  - `false`: 573.1K ops/sec
  - `1 + 2`: 226.2K ops/sec
  - `Patient.name.count()`: 279.9K ops/sec
  - `Patient.name.where(use = 'official').family`: 131.9K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 161.4K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 142.6K ops/sec
  - `Patient.contact.name.family`: 314.1K ops/sec
  - `Patient.birthDate > @1980-01-01`: 208.6K ops/sec
  - `Patient.name.family.substring(0, 3)`: 119.5K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 167.9K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 162.1K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 120.9K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 119.7K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 99.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 100.6K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 128.6K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 85.7K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 71.3K ops/sec

## Parsing Benchmarks
  - `Patient.active`: 407.3K ops/sec
  - `Patient.name.family`: 371.7K ops/sec
  - `Patient.birthDate`: 414.2K ops/sec
  - `Patient.gender`: 416.1K ops/sec
  - `true`: 623.2K ops/sec
  - `false`: 610.1K ops/sec
  - `1 + 2`: 325.6K ops/sec
  - `Patient.name.count()`: 311.2K ops/sec
  - `Patient.name.where(use = 'official').family`: 175.9K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 181.6K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 175.2K ops/sec
  - `Patient.contact.name.family`: 323.7K ops/sec
  - `Patient.birthDate > @1980-01-01`: 245.8K ops/sec
  - `Patient.name.family.substring(0, 3)`: 157.9K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 195.6K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 173.7K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 128.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 128.2K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 106.5K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 103.7K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 132.5K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 89.1K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 75.6K ops/sec

## Evaluation Benchmarks
  - `Patient.active`: 87.1K ops/sec
  - `Patient.name.family`: 97.0K ops/sec
  - `Patient.birthDate`: 119.2K ops/sec
  - `Patient.gender`: 122.9K ops/sec
  - `true`: 199.6K ops/sec
  - `false`: 198.5K ops/sec
  - `1 + 2`: 182.6K ops/sec
  - `Patient.name.count()`: 80.6K ops/sec
  - `Patient.name.where(use = 'official').family`: 67.1K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 69.3K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 112.4K ops/sec
  - `Patient.contact.name.family`: 106.9K ops/sec
  - `Patient.birthDate > @1980-01-01`: 113.7K ops/sec
  - `Patient.name.family.substring(0, 3)`: 58.8K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 54.5K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 113.1K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 17 ops/sec (ΔRSS: 87.31 MiB)
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 17 ops/sec (ΔRSS: 240.00 KiB)
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 23 ops/sec (ΔRSS: 0 B)
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 21 ops/sec (ΔRSS: 128.00 KiB)
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 20 ops/sec (ΔRSS: 2.98 MiB)
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 21 ops/sec (ΔRSS: 0 B)
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 18 ops/sec (ΔRSS: 16.00 KiB)
```

## Performance Summary

| Category | Operation | Avg Ops/sec | Notes |
|----------|-----------|-------------|--------|
| Simple   | Tokenize  | 370.2K ops/sec | Basic expressions |
| Simple   | Parse     | 434.9K ops/sec | Basic expressions |
| Simple   | Evaluate  | 135.9K ops/sec | Basic expressions |
| Medium   | Tokenize  | 176.0K ops/sec | Filtered queries |
| Medium   | Parse     | 203.7K ops/sec | Filtered queries |
| Medium   | Evaluate  | 87.0K ops/sec | Filtered queries |
| Complex  | Tokenize  | 103.7K ops/sec | Bundle operations |
| Complex  | Parse     | 109.2K ops/sec | Bundle operations |
| Complex  | Evaluate  | 68 ops/sec | Bundle operations |

## Complex Evaluation Memory by Expression

| Expression | Ops/sec | ΔRSS |
|------------|---------|------|
| `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()` | 87 ops/sec | 87.31 MiB |
| `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()` | 240 ops/sec | 240.00 KiB |
| `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()` | 0 ops/sec | 0 B |
| `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100` | 128 ops/sec | 128.00 KiB |
| `Bundle.entry.resource.descendants().where($this is Reference).reference` | 3 ops/sec | 2.98 MiB |
| `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value` | 0 ops/sec | 0 B |
| `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value` | 16 ops/sec | 16.00 KiB |

## Memory
- RSS at start: 7.12 MiB
- RSS at end: 165.30 MiB
- RSS delta: 158.17 MiB

## Usage

To run benchmarks:
```bash
cargo bench --package fhirpath-bench
```

To profile specific expressions:
```bash
fhirpath-bench profile "Patient.active"
fhirpath-bench profile "Bundle.entry.resource.count()" --bundle
```

To generate updated results:
```bash
fhirpath-bench benchmark --run --output benchmark.md
```
