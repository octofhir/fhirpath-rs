# Versioning Strategy

## Overview
OctoFHIR FHIRPath follows [Semantic Versioning (SemVer)](https://semver.org/) for all packages and releases.

## Version Format
All versions follow the format: `MAJOR.MINOR.PATCH`

- **MAJOR**: Incremented for incompatible API changes
- **MINOR**: Incremented for backwards-compatible functionality additions
- **PATCH**: Incremented for backwards-compatible bug fixes

## Pre-release Versions
Pre-release versions may use suffixes:
- `alpha`: Early development versions (e.g., `0.2.0-alpha.1`)
- `beta`: Feature-complete but potentially unstable (e.g., `0.2.0-beta.1`)
- `rc`: Release candidates (e.g., `0.2.0-rc.1`)

## Package Synchronization
All packages in the workspace maintain synchronized versions:
- `fhirpath-core`
- `fhirpath-cli`
- `@octofhir/fhirpath-node`
- `@octofhir/fhirpath-wasm`

## Release Process

### 1. Version Bumping
Update versions in:
- Root `Cargo.toml` workspace configuration
- Individual package `Cargo.toml` files (if they override a workspace version)
- `fhirpath-node/package.json`
- `fhirpath-wasm/package.json`

### 2. Automatic Tagging
Tags are created automatically when:
- Version is updated in root `Cargo.toml`
- Changes are pushed to the `main` branch
- Unreleased changes exist in `CHANGELOG.md`

The auto-tag workflow:
- Detects version changes in `Cargo.toml`
- Validates that the version has actually changed
- Checks for unreleased changes in `CHANGELOG.md`
- Creates and pushes a git tag in the format `v{VERSION}` (e.g., `v0.2.0`)
- Can be manually triggered with force option if needed

### 3. Automated Release
Once a tag is created, the release workflow automatically:
- Builds cross-platform binaries
- Publishes to crates.io
- Publishes to npm
- Creates GitHub releases with changelogs
- Uploads release artifacts

## Breaking Changes
When introducing breaking changes:
1. Document the change in `CHANGELOG.md`
2. Update migration guides if necessary
3. Increment the MAJOR version
4. Consider deprecation warnings in the previous minor version

## API Stability
- **0.x.x versions**: No API stability guarantees
- **1.x.x versions**: Full semantic versioning applies
- Public APIs are considered stable once version 1.0.0 is released

## Release Cadence
- **Patch releases**: As needed for bug fixes
- **Minor releases**: Monthly or when significant features are ready
- **Major releases**: When breaking changes are necessary

## Version Lifecycle
- **Current**: Latest stable version
- **Maintenance**: Previous major version (security fixes only)
- **End of Life**: Versions older than the maintenance version
