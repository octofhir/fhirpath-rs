# FHIRPath-rs Benchmark Results

Generated on: 2025-08-19 07:42:13 UTC

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
  - `Patient.active`: 1.4M ops/sec
  - `Patient.name.family`: 988.1K ops/sec
  - `Patient.birthDate`: 1.9M ops/sec
  - `Patient.gender`: 1.4M ops/sec
  - `true`: 3.0M ops/sec
  - `false`: 2.9M ops/sec
  - `1 + 2`: 3.6M ops/sec
  - `Patient.name.count()`: 856.7K ops/sec
  - `Patient.name.where(use = 'official').family`: 441.2K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 474.1K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 530.3K ops/sec
  - `Patient.contact.name.family`: 740.2K ops/sec
  - `Patient.birthDate > @1980-01-01`: 1.3M ops/sec
  - `Patient.name.family.substring(0, 3)`: 633.6K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 588.5K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 477.2K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 316.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 318.6K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 244.3K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 278.8K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 369.6K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 229.6K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 181.0K ops/sec

## Parsing Benchmarks
  - `Patient.active`: 820.9K ops/sec
  - `Patient.name.family`: 576.7K ops/sec
  - `Patient.birthDate`: 922.0K ops/sec
  - `Patient.gender`: 890.1K ops/sec
  - `true`: 2.3M ops/sec
  - `false`: 2.3M ops/sec
  - `1 + 2`: 1.6M ops/sec
  - `Patient.name.count()`: 498.3K ops/sec
  - `Patient.name.where(use = 'official').family`: 259.4K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 256.4K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 257.6K ops/sec
  - `Patient.contact.name.family`: 389.3K ops/sec
  - `Patient.birthDate > @1980-01-01`: 671.1K ops/sec
  - `Patient.name.family.substring(0, 3)`: 345.9K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 312.5K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 252.2K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 156.0K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 160.8K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 121.2K ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 143.0K ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 209.4K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 112.3K ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 94.1K ops/sec

## Evaluation Benchmarks
  - `Patient.active`: 99.2K ops/sec
  - `Patient.name.family`: 53.6K ops/sec
  - `Patient.birthDate`: 101.3K ops/sec
  - `Patient.gender`: 104.9K ops/sec
  - `true`: 183.6K ops/sec
  - `false`: 184.2K ops/sec
  - `1 + 2`: 132.3K ops/sec
  - `Patient.name.count()`: 73.7K ops/sec
  - `Patient.name.where(use = 'official').family`: 28.5K ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 27.9K ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 53.0K ops/sec
  - `Patient.contact.name.family`: 84.0K ops/sec
  - `Patient.birthDate > @1980-01-01`: 77.3K ops/sec
  - `Patient.name.family.substring(0, 3)`: 41.5K ops/sec
  - `Patient.telecom.exists(system = 'email')`: 68.5K ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 53.3K ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 29 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 28 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 28 ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 1 ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 60 ops/sec
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
