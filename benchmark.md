# FHIRPath-rs Benchmark Results

Generated on: 2025-09-23 18:31:29 UTC

## Overview

This benchmark suite measures the performance of FHIRPath-rs library across three main operations:
- **Tokenization**: Converting FHIRPath expressions into tokens
- **Parsing**: Building AST from tokens  
- **Evaluation**: Executing expressions against FHIR data

## Environment
- Tool: fhirpath-bench v0.4.24
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
  - `Patient.active`: 32.8K ops/sec
  - `Patient.name.family`: 32.3K ops/sec
  - `Patient.birthDate`: 35.0K ops/sec
  - `Patient.gender`: 35.6K ops/sec
  - `true`: 48.7K ops/sec
  - `false`: 47.4K ops/sec
  - `1 + 2`: 22.6K ops/sec
  - `Patient.name.count()`: 25.9K ops/sec
  - `Patient.name.where(use = 'official').family`: 14.1K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 14.6K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 14.7K ops/sec
  - `Patient.contact.name.family`: 30.7K ops/sec
  - `Patient.birthDate > @1980-01-01`: 22.5K ops/sec
  - `Patient.name.family.substring(0, 3)`: 11.6K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 15.9K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 14.6K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 10.3K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 10.7K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 8.5K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 8.6K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 10.9K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 3.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 5.7K ops/sec

## Parsing Benchmarks
  - `Patient.active`: 32.0K ops/sec
  - `Patient.name.family`: 28.3K ops/sec
  - `Patient.birthDate`: 29.8K ops/sec
  - `Patient.gender`: 31.7K ops/sec
  - `true`: 45.1K ops/sec
  - `false`: 40.2K ops/sec
  - `1 + 2`: 21.2K ops/sec
  - `Patient.name.count()`: 24.8K ops/sec
  - `Patient.name.where(use = 'official').family`: 13.6K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 14.4K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 14.7K ops/sec
  - `Patient.contact.name.family`: 30.3K ops/sec
  - `Patient.birthDate > @1980-01-01`: 22.5K ops/sec
  - `Patient.name.family.substring(0, 3)`: 11.2K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 15.9K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 14.7K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 10.6K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 10.6K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 8.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 8.6K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 11.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 7.5K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 6.3K ops/sec

## Evaluation Benchmarks
  - `Patient.active`: 11.1K ops/sec
  - `Patient.name.family`: 7.9K ops/sec
  - `Patient.birthDate`: 10.2K ops/sec
  - `Patient.gender`: 11.1K ops/sec
  - `true`: 18.4K ops/sec
  - `false`: 18.9K ops/sec
  - `1 + 2`: 12.5K ops/sec
  - `Patient.name.count()`: 7.3K ops/sec
  - `Patient.name.where(use = 'official').family`: 4.6K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 4.9K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 6.6K ops/sec
  - `Patient.contact.name.family`: 7.2K ops/sec
  - `Patient.birthDate > @1980-01-01`: 8.7K ops/sec
  - `Patient.name.family.substring(0, 3)`: 4.3K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 5.5K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 6.7K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 4.1K ops/sec (ΔRSS: 1.16 GiB)
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 4.2K ops/sec (ΔRSS: 32.00 MiB)
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 2.7K ops/sec (ΔRSS: 160.00 MiB)
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 3.5K ops/sec (ΔRSS: 16.00 MiB)
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 565 ops/sec (ΔRSS: 96.00 MiB)
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 2.8K ops/sec (ΔRSS: 16.00 MiB)
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 2.5K ops/sec (ΔRSS: 48.00 MiB)
```

## Performance Summary

| Category | Operation | Avg Ops/sec | Notes |
|----------|-----------|-------------|--------|
| Simple   | Tokenize  | 35.0K ops/sec | Basic expressions |
| Simple   | Parse     | 31.6K ops/sec | Basic expressions |
| Simple   | Evaluate  | 12.2K ops/sec | Basic expressions |
| Medium   | Tokenize  | 17.3K ops/sec | Filtered queries |
| Medium   | Parse     | 17.2K ops/sec | Filtered queries |
| Medium   | Evaluate  | 6.1K ops/sec | Filtered queries |
| Complex  | Tokenize  | 8.2K ops/sec | Bundle operations |
| Complex  | Parse     | 9.2K ops/sec | Bundle operations |
| Complex  | Evaluate  | 53 ops/sec | Bundle operations |

## Complex Evaluation Memory by Expression

| Expression | Ops/sec | ΔRSS |
|------------|---------|------|
| `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()` | 1 ops/sec | 1.16 GiB |
| `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()` | 32 ops/sec | 32.00 MiB |
| `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()` | 160 ops/sec | 160.00 MiB |
| `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100` | 16 ops/sec | 16.00 MiB |
| `Bundle.entry.resource.descendants().where($this is Reference).reference` | 96 ops/sec | 96.00 MiB |
| `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value` | 16 ops/sec | 16.00 MiB |
| `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value` | 48 ops/sec | 48.00 MiB |

## Memory
- RSS at start: 9.14 GiB
- RSS at end: 41.47 GiB
- RSS delta: 32.33 GiB

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
