# FHIRPath-rs Benchmark Results

Generated on: 2026-11-04 00:05:47 UTC

## Overview

This benchmark suite measures the performance of FHIRPath-rs library across three main operations:
- **Tokenization**: Converting FHIRPath expressions into tokens
- **Parsing**: Building AST from tokens
- **Evaluation**: Executing expressions against FHIR data

## Environment
- Tool: fhirpath-bench v0.5.5
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
  - `Patient.active`: 21.3K ops/sec
  - `Patient.name.family`: 20.4K ops/sec
  - `Patient.birthDate`: 21.1K ops/sec
  - `Patient.gender`: 21.2K ops/sec
  - `true`: 30.9K ops/sec
  - `false`: 28.8K ops/sec
  - `1 + 2`: 16.7K ops/sec
  - `Patient.name.count()`: 16.1K ops/sec
  - `Patient.name.where(use = 'official').family`: 9.1K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 9.0K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 8.6K ops/sec
  - `Patient.contact.name.family`: 18.7K ops/sec
  - `Patient.birthDate > @1980-01-01`: 13.3K ops/sec
  - `Patient.name.family.substring(0, 3)`: 7.1K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 9.4K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 8.6K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 6.5K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 6.4K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 5.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 2.6K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 5.1K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 3.4K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 3.0K ops/sec

## Parsing Benchmarks
  - `Patient.active`: 20.6K ops/sec
  - `Patient.name.family`: 19.7K ops/sec
  - `Patient.birthDate`: 17.7K ops/sec
  - `Patient.gender`: 16.2K ops/sec
  - `true`: 28.8K ops/sec
  - `false`: 21.9K ops/sec
  - `1 + 2`: 13.6K ops/sec
  - `Patient.name.count()`: 8.6K ops/sec
  - `Patient.name.where(use = 'official').family`: 5.9K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 5.6K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 6.9K ops/sec
  - `Patient.contact.name.family`: 17.7K ops/sec
  - `Patient.birthDate > @1980-01-01`: 11.8K ops/sec
  - `Patient.name.family.substring(0, 3)`: 7.8K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 9.0K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 8.3K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 6.3K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 6.3K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 5.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 5.0K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 7.2K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 4.1K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 3.2K ops/sec

## Evaluation Benchmarks
  - `Patient.active`: 14.4K ops/sec
  - `Patient.name.family`: 14.7K ops/sec
  - `Patient.birthDate`: 15.4K ops/sec
  - `Patient.gender`: 15.9K ops/sec
  - `true`: 17.7K ops/sec
  - `false`: 17.9K ops/sec
  - `1 + 2`: 17.8K ops/sec
  - `Patient.name.count()`: 15.8K ops/sec
  - `Patient.name.where(use = 'official').family`: 12.4K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 12.4K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 14.4K ops/sec
  - `Patient.contact.name.family`: 14.7K ops/sec
  - `Patient.birthDate > @1980-01-01`: 14.7K ops/sec
  - `Patient.name.family.substring(0, 3)`: 14.3K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 13.4K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 14.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 13 ops/sec (ΔRSS: 41.81 MiB)
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 13 ops/sec (ΔRSS: 112.00 KiB)
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 14 ops/sec (ΔRSS: 80.00 KiB)
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 13 ops/sec (ΔRSS: 288.00 KiB)
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 14 ops/sec (ΔRSS: 1.94 MiB)
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 14 ops/sec (ΔRSS: 0 B)
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 15 ops/sec (ΔRSS: 0 B)
```

## Performance Summary

| Category | Operation | Avg Ops/sec | Notes |
|----------|-----------|-------------|--------|
| Simple   | Tokenize  | 22.1K ops/sec | Basic expressions |
| Simple   | Parse     | 18.4K ops/sec | Basic expressions |
| Simple   | Evaluate  | 16.2K ops/sec | Basic expressions |
| Medium   | Tokenize  | 10.5K ops/sec | Filtered queries |
| Medium   | Parse     | 9.1K ops/sec | Filtered queries |
| Medium   | Evaluate  | 13.9K ops/sec | Filtered queries |
| Complex  | Tokenize  | 4.6K ops/sec | Bundle operations |
| Complex  | Parse     | 5.3K ops/sec | Bundle operations |
| Complex  | Evaluate  | 75 ops/sec | Bundle operations |

## Complex Evaluation Memory by Expression

| Expression | Ops/sec | ΔRSS |
|------------|---------|------|
| `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()` | 42 ops/sec | 41.81 MiB |
| `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()` | 112 ops/sec | 112.00 KiB |
| `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()` | 80 ops/sec | 80.00 KiB |
| `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100` | 288 ops/sec | 288.00 KiB |
| `Bundle.entry.resource.descendants().where($this is Reference).reference` | 2 ops/sec | 1.94 MiB |
| `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value` | 0 ops/sec | 0 B |
| `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value` | 0 ops/sec | 0 B |

## Memory
- RSS at start: 8.88 MiB
- RSS at end: 123.77 MiB
- RSS delta: 114.89 MiB

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
