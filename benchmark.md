# FHIRPath-rs Benchmark Results

Generated on: 2025-08-26 08:17:47 UTC

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
  - `Patient.active`: 1.2M ops/sec
  - `Patient.name.family`: 986.4K ops/sec
  - `Patient.birthDate`: 1.9M ops/sec
  - `Patient.gender`: 1.5M ops/sec
  - `true`: 2.9M ops/sec
  - `false`: 2.9M ops/sec
  - `1 + 2`: 3.6M ops/sec
  - `Patient.name.count()`: 864.5K ops/sec
  - `Patient.name.where(use = 'official').family`: 495.8K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 499.6K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 532.5K ops/sec
  - `Patient.contact.name.family`: 750.6K ops/sec
  - `Patient.birthDate > @1980-01-01`: 1.3M ops/sec
  - `Patient.name.family.substring(0, 3)`: 617.5K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 573.4K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 438.5K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 316.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 320.7K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 247.3K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 282.3K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 362.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 231.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 181.5K ops/sec

## Parsing Benchmarks
  - `Patient.active`: 837.3K ops/sec
  - `Patient.name.family`: 573.2K ops/sec
  - `Patient.birthDate`: 1.1M ops/sec
  - `Patient.gender`: 791.8K ops/sec
  - `true`: 2.4M ops/sec
  - `false`: 2.3M ops/sec
  - `1 + 2`: 1.7M ops/sec
  - `Patient.name.count()`: 514.0K ops/sec
  - `Patient.name.where(use = 'official').family`: 262.6K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 263.8K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 263.0K ops/sec
  - `Patient.contact.name.family`: 395.7K ops/sec
  - `Patient.birthDate > @1980-01-01`: 677.4K ops/sec
  - `Patient.name.family.substring(0, 3)`: 349.7K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 319.6K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 255.9K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 162.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 165.4K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 128.1K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 140.0K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 212.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 115.4K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 95.2K ops/sec

## Evaluation Benchmarks
  - `Patient.active`: 97.8K ops/sec
  - `Patient.name.family`: 65.2K ops/sec
  - `Patient.birthDate`: 96.2K ops/sec
  - `Patient.gender`: 95.6K ops/sec
  - `true`: 179.2K ops/sec
  - `false`: 182.2K ops/sec
  - `1 + 2`: 144.4K ops/sec
  - `Patient.name.count()`: 65.1K ops/sec
  - `Patient.name.where(use = 'official').family`: 29.7K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 30.9K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 52.1K ops/sec
  - `Patient.contact.name.family`: 95.3K ops/sec
  - `Patient.birthDate > @1980-01-01`: 78.6K ops/sec
  - `Patient.name.family.substring(0, 3)`: 42.7K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 34.2K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 53.3K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 5 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 5 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 25 ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 1 ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 3 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 28 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 14 ops/sec
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
