# FHIRPath-rs Benchmark Results

Generated on: 2025-08-18 21:39:18 UTC

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
  - `Patient.active`: 478306.87 ops/sec
  - `Patient.name.family`: 385096.76 ops/sec
  - `Patient.birthDate`: 789993.62 ops/sec
  - `Patient.gender`: 665612.78 ops/sec
  - `true`: 2209129.45 ops/sec
  - `false`: 2626685.02 ops/sec
  - `1 + 2`: 2558094.32 ops/sec
  - `Patient.name.count()`: 440165.06 ops/sec
  - `Patient.name.where(use = 'official').family`: 226733.82 ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 249311.77 ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 237569.64 ops/sec
  - `Patient.contact.name.family`: 300868.79 ops/sec
  - `Patient.birthDate > @1980-01-01`: 639641.80 ops/sec
  - `Patient.name.family.substring(0, 3)`: 276838.99 ops/sec
  - `Patient.telecom.exists(system = 'email')`: 267725.72 ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 261392.40 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 183566.23 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 181063.74 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 134158.41 ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 153879.68 ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 218652.93 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 125218.47 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 99330.76 ops/sec

## Parsing Benchmarks
  - `Patient.active`: 446686.21 ops/sec
  - `Patient.name.family`: 323978.46 ops/sec
  - `Patient.birthDate`: 633428.96 ops/sec
  - `Patient.gender`: 570382.98 ops/sec
  - `true`: 2213613.72 ops/sec
  - `false`: 2135798.33 ops/sec
  - `1 + 2`: 1630212.40 ops/sec
  - `Patient.name.count()`: 336983.99 ops/sec
  - `Patient.name.where(use = 'official').family`: 172928.11 ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 189612.39 ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 168048.39 ops/sec
  - `Patient.contact.name.family`: 254073.11 ops/sec
  - `Patient.birthDate > @1980-01-01`: 496472.81 ops/sec
  - `Patient.name.family.substring(0, 3)`: 225980.22 ops/sec
  - `Patient.telecom.exists(system = 'email')`: 211437.01 ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 184518.87 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 134207.92 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 134276.26 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 97225.03 ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 114144.38 ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 169583.73 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 91248.85 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 72323.36 ops/sec

## Evaluation Benchmarks
  - `Patient.active`: 93341.57 ops/sec
  - `Patient.name.family`: 52025.75 ops/sec
  - `Patient.birthDate`: 93603.77 ops/sec
  - `Patient.gender`: 98023.26 ops/sec
  - `true`: 172006.02 ops/sec
  - `false`: 180018.00 ops/sec
  - `1 + 2`: 141668.14 ops/sec
  - `Patient.name.count()`: 51825.80 ops/sec
  - `Patient.name.where(use = 'official').family`: 25363.01 ops/sec
  - `Patient.telecom.where(system = 'phone').value`: 26166.02 ops/sec
  - `Patient.extension.where(url = 'http://example.org').value`: 47151.28 ops/sec
  - `Patient.contact.name.family`: 78290.63 ops/sec
  - `Patient.birthDate > @1980-01-01`: 70237.05 ops/sec
  - `Patient.name.family.substring(0, 3)`: 36373.56 ops/sec
  - `Patient.telecom.exists(system = 'email')`: 59594.76 ops/sec
  - `Patient.identifier.where(system = 'http://example.org/mrn').value`: 49079.75 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().count()`: 26.20 ops/sec
  - `Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().first()`: 29.64 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').name.where(use='official').family.first()`: 30.45 ops/sec
  - `Bundle.entry.resource.where(resourceType='Observation').value.as(Quantity).value > 100`: 1.00 ops/sec
  - `Bundle.entry.resource.descendants().where($this is Reference).reference`: 62.54 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient').telecom.where(system='phone' and use='mobile').value`: 30.63 ops/sec
  - `Bundle.entry.resource.where(resourceType='Patient' and telecom.exists() and telecom.system = 'phone' and telecom.user = 'mobile').value`: 15.30 ops/sec
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
