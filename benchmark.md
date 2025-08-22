# FHIRPath-rs Benchmark Results

Generated on: 2025-08-22 12:16:29 UTC

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
  - `Patient.active`: 1.3M ops/sec
  - `Patient.name.family`: 1.0M ops/sec
  - `Patient.birthDate`: 2.0M ops/sec
  - `Patient.gender`: 1.5M ops/sec
  - `true`: 3.1M ops/sec
  - `false`: 3.0M ops/sec
  - `1 + 2`: 3.6M ops/sec
  - `Patient.name.count()`: 857.3K ops/sec
  - `Patient.name.where(use = 'official').family`: 469.2K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 453.9K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 531.4K ops/sec
  - `Patient.contact.name.family`: 760.3K ops/sec
  - `Patient.birthDate > @1980-01-01`: 1.3M ops/sec
  - `Patient.name.family.substring(0, 3)`: 631.7K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 572.2K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 532.7K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 319.2K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 318.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 243.2K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 273.3K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 356.2K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 231.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 181.1K ops/sec

## Parsing Benchmarks
  - `Patient.active`: 798.3K ops/sec
  - `Patient.name.family`: 554.3K ops/sec
  - `Patient.birthDate`: 965.8K ops/sec
  - `Patient.gender`: 887.8K ops/sec
  - `true`: 2.4M ops/sec
  - `false`: 2.4M ops/sec
  - `1 + 2`: 1.6M ops/sec
  - `Patient.name.count()`: 496.9K ops/sec
  - `Patient.name.where(use = 'official').family`: 256.8K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 245.3K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 235.9K ops/sec
  - `Patient.contact.name.family`: 372.9K ops/sec
  - `Patient.birthDate > @1980-01-01`: 672.6K ops/sec
  - `Patient.name.family.substring(0, 3)`: 331.8K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 311.2K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 242.6K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 156.1K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 156.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 117.6K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 139.5K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 206.6K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 111.7K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 91.8K ops/sec

## Evaluation Benchmarks
  - `Patient.active`: 99.8K ops/sec
  - `Patient.name.family`: 52.5K ops/sec
  - `Patient.birthDate`: 101.9K ops/sec
  - `Patient.gender`: 104.2K ops/sec
  - `true`: 183.8K ops/sec
  - `false`: 184.6K ops/sec
  - `1 + 2`: 135.0K ops/sec
  - `Patient.name.count()`: 70.3K ops/sec
  - `Patient.name.where(use = 'official').family`: 26.6K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 24.5K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 52.6K ops/sec
  - `Patient.contact.name.family`: 93.5K ops/sec
  - `Patient.birthDate > @1980-01-01`: 81.4K ops/sec
  - `Patient.name.family.substring(0, 3)`: 41.3K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 68.7K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 52.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 28 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 27 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 28 ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 1 ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 58 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 27 ops/sec
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
