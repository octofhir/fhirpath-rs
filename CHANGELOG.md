# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Phase 6 implementation: Complete packaging and distribution setup
- Comprehensive release automation with GitHub Actions
- Cross-platform binary builds for all major platforms
- npm packaging for Node.js bindings (@octofhir/fhirpath-node)
- WASM packaging for browsers and Cloudflare Workers (@octofhir/fhirpath-wasm)
- crates.io publishing configuration
- Versioning strategy documentation
- Automated changelog generation

### Changed
- Enhanced CI/CD pipeline with release automation
- Improved documentation deployment workflow

### Fixed
- Various bug fixes and improvements

## [0.1.0] - 2024-07-18

### Added
- Initial release of OctoFHIR FHIRPath
- Core FHIRPath engine implementation
- Command-line interface (CLI)
- Node.js bindings via NAPI-RS
- WebAssembly bindings
- Basic documentation and examples
- Conformance testing infrastructure

### Features
- FHIRPath expression parsing and evaluation
- Support for FHIR R4 data structures
- Cross-platform compatibility
- Multiple language bindings (Rust, Node.js, WASM)

---

## Release Template

When creating a new release, copy this template and fill in the details:

```markdown
## [VERSION] - YYYY-MM-DD

### Added
- New features and functionality

### Changed
- Changes to existing functionality

### Deprecated
- Features that will be removed in future versions

### Removed
- Features that have been removed

### Fixed
- Bug fixes

### Security
- Security improvements and fixes
```

## Guidelines

### Categories
- **Added**: New features
- **Changed**: Changes in existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security improvements

### Writing Style
- Use present tense ("Add feature" not "Added feature")
- Be concise but descriptive
- Include issue/PR numbers when relevant
- Group related changes together
- Order entries by importance (most important first)

### Version Links
All version numbers should link to the corresponding GitHub release or tag.
