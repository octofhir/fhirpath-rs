# FHIRPath-rs Benchmark Results

Generated on: 2025-08-16 12:14:02 UTC

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

## Benchmark Results

```
## Tokenization Benchmarks
  - `Patient.active`: 471818.74 ops/sec
  - `Patient.name.family`: 405056.56 ops/sec
  - `Patient.birthDate`: 814719.04 ops/sec
  - `Patient.gender`: 669362.41 ops/sec
  - `true`: 2564648.37 ops/sec
  - `false`: 1955190.93 ops/sec
  - `1 + 2`: 2428900.02 ops/sec
  - `Patient.name.count()`: 449101.73 ops/sec
  - `Patient.name.where(use = 'official').family`: 244429.03 ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 264576.51 ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 257743.05 ops/sec
  - `Patient.contact.name.family`: 306388.23 ops/sec
  - `Patient.birthDate > @1980-01-01`: 672268.91 ops/sec
  - `Patient.name.family.substring(0, 3)`: 291516.89 ops/sec
  - `Patient.telecom.exists(system = 'email')`: 280311.64 ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 270051.31 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 187536.62 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 193340.99 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 138649.80 ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 142048.80 ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 226073.87 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 131045.15 ops/sec

## Parsing Benchmarks
  - `Patient.active`: 447777.90 ops/sec
  - `Patient.name.family`: 325724.09 ops/sec
  - `Patient.birthDate`: 654807.23 ops/sec
  - `Patient.gender`: 556534.75 ops/sec
  - `true`: 2303701.59 ops/sec
  - `false`: 2289639.38 ops/sec
  - `1 + 2`: 1741401.83 ops/sec
  - `Patient.name.count()`: 343888.89 ops/sec
  - `Patient.name.where(use = 'official').family`: 166144.01 ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 180879.52 ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 176362.95 ops/sec
  - `Patient.contact.name.family`: 242772.48 ops/sec
  - `Patient.birthDate > @1980-01-01`: 489416.37 ops/sec
  - `Patient.name.family.substring(0, 3)`: 222179.02 ops/sec
  - `Patient.telecom.exists(system = 'email')`: 206768.23 ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 173211.40 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 129168.38 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 130343.39 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 93375.41 ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 109271.70 ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 166178.52 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 88165.93 ops/sec

## Evaluation Benchmarks
  - `Patient.active`: 62201.97 ops/sec
  - `Patient.name.family`: 41943.37 ops/sec
  - `Patient.birthDate`: 63013.64 ops/sec
  - `Patient.gender`: 55441.32 ops/sec
  - `true`: 96875.76 ops/sec
  - `false`: 97288.09 ops/sec
  - `1 + 2`: 66644.45 ops/sec
  - `Patient.name.count()`: 48390.02 ops/sec
  - `Patient.name.where(use = 'official').family`: 38595.14 ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 39908.21 ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 45555.49 ops/sec
  - `Patient.contact.name.family`: 52283.02 ops/sec
  - `Patient.birthDate > @1980-01-01`: 48492.68 ops/sec
  - `Patient.name.family.substring(0, 3)`: 29805.52 ops/sec
  - `Patient.telecom.exists(system = 'email')`: 43427.11 ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 47327.96 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 9.22 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 9.16 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 9.51 ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 9.49 ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 9.47 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 9.52 ops/sec
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
