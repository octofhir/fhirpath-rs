# FHIRPath-rs Benchmark Results

Generated on: 2025-09-05 14:07:04 UTC

## Overview

This benchmark suite measures the performance of FHIRPath-rs library across three main operations:
- **Tokenization**: Converting FHIRPath expressions into tokens
- **Parsing**: Building AST from tokens  
- **Evaluation**: Executing expressions against FHIR data

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
  - `Patient.active`: 33.4K ops/sec
  - `Patient.name.family`: 31.1K ops/sec
  - `Patient.birthDate`: 33.8K ops/sec
  - `Patient.gender`: 33.7K ops/sec
  - `true`: 47.6K ops/sec
  - `false`: 45.2K ops/sec
  - `1 + 2`: 19.3K ops/sec
  - `Patient.name.count()`: 22.4K ops/sec
  - `Patient.name.where(use = 'official').family`: 11.9K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 14.0K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 13.7K ops/sec
  - `Patient.contact.name.family`: 27.2K ops/sec
  - `Patient.birthDate > @1980-01-01`: 21.0K ops/sec
  - `Patient.name.family.substring(0, 3)`: 8.9K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 14.6K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 13.5K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 9.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 9.6K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 8.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 7.9K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 12.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 7.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 6.2K ops/sec

## Parsing Benchmarks
  - `Patient.active`: 34.1K ops/sec
  - `Patient.name.family`: 30.8K ops/sec
  - `Patient.birthDate`: 33.6K ops/sec
  - `Patient.gender`: 33.6K ops/sec
  - `true`: 47.7K ops/sec
  - `false`: 46.0K ops/sec
  - `1 + 2`: 21.1K ops/sec
  - `Patient.name.count()`: 24.2K ops/sec
  - `Patient.name.where(use = 'official').family`: 14.1K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 14.2K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 13.7K ops/sec
  - `Patient.contact.name.family`: 28.4K ops/sec
  - `Patient.birthDate > @1980-01-01`: 21.0K ops/sec
  - `Patient.name.family.substring(0, 3)`: 11.0K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 14.8K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 13.5K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 9.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 9.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 6.9K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 8.0K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 12.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 7.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 6.2K ops/sec

## Evaluation Benchmarks
  - `Patient.active`: 23.3K ops/sec
  - `Patient.name.family`: 21.9K ops/sec
  - `Patient.birthDate`: 23.5K ops/sec
  - `Patient.gender`: 23.5K ops/sec
  - `true`: 30.3K ops/sec
  - `false`: 30.5K ops/sec
  - `1 + 2`: 25.6K ops/sec
  - `Patient.name.count()`: 19.1K ops/sec
  - `Patient.name.where(use = 'official').family`: 12.6K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 12.6K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 13.2K ops/sec
  - `Patient.contact.name.family`: 21.1K ops/sec
  - `Patient.birthDate > @1980-01-01`: 22.8K ops/sec
  - `Patient.name.family.substring(0, 3)`: 19.2K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 17.0K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 13.1K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 8.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 9.6K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 6.1K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 8.6K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 3.9K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 4.9K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 5.6K ops/sec
```

## Performance Summary

| Category | Operation | Avg Ops/sec | Notes |
|----------|-----------|-------------|--------|
| Simple   | Tokenize  | -           | Basic expressions |
| Simple   | Parse     | -           | Basic expressions |
| Simple   | Evaluate  | -           | Basic expressions |
| Medium   | Tokenize  | -           | Filtered queries |
| Medium   | Parse     | -           | Filtered queries |
| Medium   | Evaluate  | -           | Filtered queries |
| Complex  | Tokenize  | -           | Bundle operations |
| Complex  | Parse     | -           | Bundle operations |
| Complex  | Evaluate  | -           | Bundle operations |

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
