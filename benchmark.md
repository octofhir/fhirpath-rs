# FHIRPath-rs Benchmark Results

Generated on: 2025-08-18 15:31:07 UTC

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
  - `Patient.active`: 500375.28 ops/sec
  - `Patient.name.family`: 406359.53 ops/sec
  - `Patient.birthDate`: 822424.56 ops/sec
  - `Patient.gender`: 698547.44 ops/sec
  - `true`: 2830519.80 ops/sec
  - `false`: 2752924.98 ops/sec
  - `1 + 2`: 2699055.33 ops/sec
  - `Patient.name.count()`: 433980.69 ops/sec
  - `Patient.name.where(use = 'official').family`: 233622.10 ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 265213.93 ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 255446.83 ops/sec
  - `Patient.contact.name.family`: 313750.10 ops/sec
  - `Patient.birthDate > @1980-01-01`: 663184.95 ops/sec
  - `Patient.name.family.substring(0, 3)`: 278090.01 ops/sec
  - `Patient.telecom.exists(system = 'email')`: 280314.94 ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 274568.08 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 193756.21 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 185836.20 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 140202.49 ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 163595.84 ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 221663.97 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 131962.40 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 102210.30 ops/sec

## Parsing Benchmarks
  - `Patient.active`: 444164.72 ops/sec
  - `Patient.name.family`: 325648.99 ops/sec
  - `Patient.birthDate`: 641402.67 ops/sec
  - `Patient.gender`: 566906.92 ops/sec
  - `true`: 2277038.12 ops/sec
  - `false`: 2262228.48 ops/sec
  - `1 + 2`: 1701282.09 ops/sec
  - `Patient.name.count()`: 311243.65 ops/sec
  - `Patient.name.where(use = 'official').family`: 170143.98 ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 183087.31 ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 166478.21 ops/sec
  - `Patient.contact.name.family`: 253876.95 ops/sec
  - `Patient.birthDate > @1980-01-01`: 481395.97 ops/sec
  - `Patient.name.family.substring(0, 3)`: 223228.84 ops/sec
  - `Patient.telecom.exists(system = 'email')`: 203446.71 ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 180285.75 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 129313.83 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 127565.92 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 94023.62 ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 109509.53 ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 162124.93 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 88108.96 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 73782.81 ops/sec

## Evaluation Benchmarks
  - `Patient.active`: 89629.11 ops/sec
  - `Patient.name.family`: 51907.60 ops/sec
  - `Patient.birthDate`: 94040.20 ops/sec
  - `Patient.gender`: 98789.82 ops/sec
  - `true`: 182982.62 ops/sec
  - `false`: 176860.62 ops/sec
  - `1 + 2`: 146439.69 ops/sec
  - `Patient.name.count()`: 65580.91 ops/sec
  - `Patient.name.where(use = 'official').family`: 25178.34 ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 25981.62 ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 48009.59 ops/sec
  - `Patient.contact.name.family`: 83134.16 ops/sec
  - `Patient.birthDate > @1980-01-01`: 77140.67 ops/sec
  - `Patient.name.family.substring(0, 3)`: 38424.59 ops/sec
  - `Patient.telecom.exists(system = 'email')`: 60777.93 ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 48397.84 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 28.54 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 28.61 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 28.04 ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 0.91 ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 61.96 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 28.89 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 14.83 ops/sec
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
