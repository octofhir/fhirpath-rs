# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.29] - 2025-01-XX

### Added

- **difference(precision) function** - FHIRPath v3.0.0 STU temporal function
  - Computes signed integer difference between two temporal values in specified units
  - Supports precision units: 'years', 'months', 'days', 'hours', 'minutes', 'seconds', 'milliseconds'
  - Returns signed integer (negative if input < comparand, positive otherwise)
  - Automatic truncation to whole units
  - Calendar arithmetic for years/months (variable month lengths, leap years)
  - Time-based arithmetic for hours/minutes/seconds/milliseconds
  - Examples:
    - `@2023-01-01.difference(@2024-01-01, 'years')` → `-1`
    - `@2024-01-01.difference(@2023-01-01, 'years')` → `1`
    - `@2024-01-01.difference(@2024-01-15, 'days')` → `-14`
    - `@2024-01-01.difference(@2024-01-01, 'days')` → `0`
    - `@2024-01-01T10:00:00.difference(@2024-01-01T12:00:00, 'hours')` → `-2`
  - Added 11 comprehensive tests covering all scenarios
  - Zero warnings, no test regression (484 tests pass)

- **duration() function** - FHIRPath v3.0.0 STU temporal function
  - Calculates absolute (always positive) duration between two temporal values
  - Requires exactly 2 temporal values of the same type
  - Unit selection based on temporal type:
    - Date - Date → Quantity in 'day' units
    - DateTime - DateTime → Quantity in 'ms' (millisecond) units
    - Time - Time → Quantity in 'ms' (millisecond) units
  - Type mismatch returns empty collection
  - Examples:
    - `{ @2024-01-01, @2024-01-15 }.duration()` → `14 'day'`
    - `{ @2024-01-15, @2024-01-01 }.duration()` → `14 'day'` (absolute value)
    - `{ @2024-01-01T00:00:00, @2024-01-01T01:00:00 }.duration()` → `3600000 'ms'`
    - `{ @T10:00:00, @T10:30:00 }.duration()` → `1800000 'ms'`
  - Added 10 comprehensive tests covering all scenarios
  - Zero warnings, no test regression (474 tests pass)

- **Quantity - Quantity subtraction with UCUM conversion** - FHIRPath v3.0.0 quantity arithmetic
  - Automatic unit conversion for compatible quantities using UCUM
  - Same units: direct subtraction
  - Compatible units: converts right to left's unit, then subtracts
  - Incompatible units: returns empty collection
  - Supports length (m, cm, mm), mass (kg, g, mg), time (h, min, s) and other UCUM units
  - Mirrors addition operator logic for consistency

- **Time - Time subtraction** - FHIRPath v3.0.0 time arithmetic
  - Returns Quantity in 'ms' (millisecond) units with UCUM code
  - Calculates difference in milliseconds between two times
  - No wrap-around behavior (10:00 - 23:00 = negative result)
  - Supports millisecond precision (sub-second differences)
  - Handles full time range (midnight to 23:59:59.999)
  - Comprehensive test coverage including edge cases and precision

- **DateTime - DateTime subtraction** - FHIRPath v3.0.0 datetime arithmetic
  - Returns Quantity in 'ms' (millisecond) units with UCUM code
  - Calculates difference in milliseconds between two datetimes
  - Automatic timezone conversion (datetimes stored with FixedOffset)
  - Handles same instant in different timezones correctly
  - Supports negative results when left datetime is earlier than right
  - Comprehensive test coverage including timezone handling and large differences

- **Date - Date subtraction** - FHIRPath v3.0.0 date arithmetic
  - Returns Quantity in 'day' units with UCUM code
  - Calculates difference in whole days between two dates
  - Handles leap years correctly (February 29, 366-day years)
  - Returns empty for partial dates (Year or Month precision)
  - Supports negative results when left date is earlier than right
  - Comprehensive test coverage including year boundaries and large differences

- **millisecond() function** - FHIRPath v3.0.0 temporal component extraction function
  - Extracts the millisecond component (0-999) from DateTime or Time values
  - Returns empty if the value doesn't have millisecond precision
  - Consistent with other temporal component functions (hourOf, minuteOf, secondOf)
  - Completes the temporal component extraction API

- **sum() function** - FHIRPath v3.0.0 aggregate function
  - Returns the sum of all numeric items in a collection
  - Supports Integer, Decimal, and Quantity types
  - Handles type promotion (Integer to Decimal when mixed)
  - UCUM unit conversion for compatible Quantity units
  - Skips empty values in collection
  - Returns empty for empty collections or type mismatches

- **min() function** - FHIRPath v3.0.0 aggregate function
  - Returns the smallest value in a collection
  - Supports Integer, Decimal, Quantity, Date, DateTime, Time, and String types
  - Uses standard comparison semantics for each type
  - Handles incompatible types/units gracefully
  - Skips empty values in collection
  - Returns empty for empty collections or type mismatches

- **max() function** - FHIRPath v3.0.0 aggregate function
  - Returns the largest value in a collection
  - Supports Integer, Decimal, Quantity, Date, DateTime, Time, and String types
  - Uses standard comparison semantics for each type
  - Handles incompatible types/units gracefully
  - Skips empty values in collection
  - Returns empty for empty collections or type mismatches

- **avg() function** - FHIRPath v3.0.0 aggregate function
  - Calculates the average of all numeric items in a collection
  - Supports Integer (returns Decimal), Decimal, and Quantity types
  - Always returns Decimal for numeric types (never Integer)
  - Leverages sum() function with division by count
  - UCUM unit conversion for compatible Quantity units
  - Skips empty values in collection
  - Returns empty for empty collections or type mismatches

- Architecture Decision Records (ADRs) for major architectural decisions
  - ADR-001: Model Context Protocol (MCP) Server Implementation
  - ADR-002: FHIRPath Analyzer Crate for static analysis and expression explanation
- Enhanced CLAUDE.md with improved workspace architecture documentation
- Simple and complete usage examples in README.md
- CLI quick start example with practical usage
- Comprehensive contributor guide updates
- Future development roadmap with MCP server and analyzer crate plans

### Changed

- **Multiplication operator (*) - Proper UCUM unit algebra for Quantity * Quantity**
  - Replaced simple string concatenation with proper UCUM unit multiplication
  - Uses `octofhir_ucum::unit_multiply()` for correct unit composition
  - Handles dimensionless units ("1") correctly - they act as identity in multiplication
  - Examples:
    - `2 'm' * 3 'm'` → `6 'm.m'` (correct UCUM expression)
    - `5 '1' * 3 'm'` → `15 'm'` (dimensionless acts as identity)
    - `10 'm' * 5 's'` → `50 'm.s'` (proper unit composition)
  - Gracefully falls back to string concatenation for non-UCUM units
  - Added 15 comprehensive tests covering all multiplication scenarios
  - Zero clippy warnings, no test regression

- Updated README.md with modular workspace architecture description
- Improved Quick Start section with both simple and complete examples
- Enhanced CLI documentation with practical examples
- Updated CONTRIBUTING.md to reflect workspace structure
- Updated specification compliance rate to 88.1%
- Enhanced development workflow documentation
- Updated architecture overview to show 11 specialized crates

### Fixed

- **Division operator (/) - Critical fixes for FHIRPath v3.0.0 compliance**
  - **Division by zero now returns empty collection (not error)** for all type combinations
    - Previously: Decimal/Integer mixed types and Quantities returned errors on division by zero
    - Now: All division by zero cases return empty collection per FHIRPath spec
    - Safer behavior, prevents exceptions during evaluation
  - **Quantity / Quantity with same units now returns Decimal (not Quantity)**
    - Previously: `6 'kg' / 2 'kg'` returned `3 '1'` (Quantity with unit "1")
    - Now: `6 'kg' / 2 'kg'` returns `3.0` (Decimal with no units)
    - Units properly cancel when identical, per FHIRPath specification
  - Added 23 comprehensive tests covering all division scenarios
  - Zero clippy warnings, no test regression

## [0.4.28] - 2025-01-XX

### Changed
- Updated ecosystem libraries and dependencies
- Improved support for new math functions from latest FHIRPath specification updates

### Fixed
- Resolved various warnings and clippy errors
- General maintenance improvements

## [0.4.0] - 2024-XX-XX

### Added

- Comprehensive CI/CD pipeline with GitHub Actions
- Canary releases on every main branch push
- Automated release workflow with manual trigger
- Automatic crates.io publishing on tag push
- GitHub issue templates for bug reports and feature requests
- Pull request template with comprehensive checklist
- Dependabot configuration for automated dependency updates
- Code coverage reporting with codecov
- Security audit workflow
- MSRV (Minimum Supported Rust Version) testing

### Changed

- Package name from `fhirpath-rs` to `octofhir-fhirpath` for crates.io compatibility
- Library name from `fhirpath_rs` to `octofhir_fhirpath` for consistency
- Updated package metadata with better description and categorization
- Added MSRV specification (Rust 1.87.0)

### Fixed

- Resolved deprecation warnings in benchmark files
- Fixed clippy warning about infinite loop in parser
- Fixed compilation errors in diagnostic reporter
- Fixed hanging cache tests with proper eviction logic
- Fixed union type compatibility and signature matching
- All tests now pass (122 passed, 0 failed)

## [0.2.0] - 2024-07-31

### Added

- Comprehensive FHIRPath implementation with 82.7% official spec compliance
- Modular architecture with separate components for parsing, evaluation, and registry
- High-performance tokenizer and parser using nom library
- Function registry with built-in FHIRPath functions
- Type system with comprehensive type checking
- Diagnostic system with detailed error reporting
- Caching infrastructure for improved performance
- Benchmark suite for performance monitoring
- CLI tool for FHIRPath evaluation
- Integration with official FHIRPath test suites

### Changed

- Consolidated from multi-crate workspace to single crate with modular structure
- Improved error handling and diagnostic reporting
- Enhanced type system with better compatibility checking

### Fixed

- Numerous parsing edge cases
- Type coercion and compatibility issues
- Performance bottlenecks in evaluation engine

## [0.1.0] - Initial Release

### Added

- Basic FHIRPath parser and evaluator
- Core functionality for FHIRPath expressions
- Initial test suite
- Basic documentation
