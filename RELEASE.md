# Release Process

This document describes the automated release process for OctoFHIR FHIRPath.

## Overview

The project uses GitHub Actions for automated cross-platform builds and releases. The release process includes:

- Building CLI binaries for multiple platforms
- Building Node.js native bindings for different OS/architecture combinations
- Building WebAssembly packages
- Publishing to crates.io, npm, and GitHub releases

## Workflows

### CI Workflow (`.github/workflows/ci.yml`)

Runs on every push and pull request to `main` and `develop` branches. Includes:

- **Rust Testing**: Tests all Rust components on Linux, macOS, and Windows
- **Node.js Testing**: Tests Node.js bindings on all platforms
- **WASM Testing**: Tests WebAssembly package builds
- **Documentation**: Validates documentation builds
- **Security Audit**: Runs security audits on all components

### Release Workflow (`.github/workflows/release.yml`)

Triggered by:
- Pushing a tag matching `v*` pattern (e.g., `v0.1.2`)
- Manual workflow dispatch with tag input

The release workflow includes these jobs:

#### 1. Build CLI Binaries (`build-cli`)
Builds the `octofhir-fhirpath` CLI binary for:
- **Linux**: x86_64, aarch64
- **macOS**: x86_64, aarch64 (Apple Silicon)
- **Windows**: x86_64

#### 2. Build Node.js Bindings (`build-node`)
Builds native Node.js modules using NAPI-RS for:
- **macOS**: x86_64-apple-darwin, aarch64-apple-darwin
- **Windows**: x86_64-pc-windows-msvc
- **Linux**: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu

#### 3. Build WASM Package (`build-wasm`)
Builds WebAssembly packages with multiple targets:
- Web target (for browsers)
- Bundler target (for webpack/rollup)
- Node.js target (for Node.js environments)

#### 4. Create GitHub Release (`release`)
- Downloads all build artifacts
- Creates compressed archives (tar.gz for Unix, zip for Windows)
- Creates a GitHub release with all binaries

#### 5. Publish to crates.io (`publish-crates`)
Publishes Rust crates in dependency order:
1. `fhirpath-core` (core library)
2. `fhirpath-cli` (CLI tool)

#### 6. Publish Node.js Package (`publish-node`)
Publishes `@octofhir/fhirpath-node` to npm with all platform binaries.

#### 7. Publish WASM Package (`publish-wasm`)
Publishes `@octofhir/fhirpath-wasm` to npm.

## Required Secrets

The following secrets must be configured in the GitHub repository settings:

### For crates.io Publishing
- `CARGO_REGISTRY_TOKEN`: API token from [crates.io](https://crates.io/me)
  - Go to crates.io → Account Settings → API Tokens
  - Create a new token with publish permissions

### For npm Publishing
- `NPM_TOKEN`: API token from [npmjs.com](https://www.npmjs.com/)
  - Go to npmjs.com → Access Tokens → Generate New Token
  - Choose "Automation" type for CI/CD usage
  - Ensure the token has publish permissions for `@octofhir` scope

## Release Process

### Automated Release (Recommended)

1. **Update Version**: Update the version in `Cargo.toml` (workspace version)
2. **Update Changelog**: Add changes to `CHANGELOG.md` under `[Unreleased]` section
3. **Sync Package Versions**: Ensure versions match in:
   - `fhirpath-node/package.json`
   - `fhirpath-wasm/package.json`
4. **Create and Push Tag**:
   ```bash
   git tag v0.1.2
   git push origin v0.1.2
   ```
5. **Monitor Release**: Check GitHub Actions for build progress

### Manual Release

You can also trigger a release manually:

1. Go to GitHub Actions → Release workflow
2. Click "Run workflow"
3. Enter the tag name (e.g., `v0.1.2`)
4. Click "Run workflow"

## Platform Support

### CLI Binaries
- **Linux**: x86_64, aarch64 (ARM64)
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x86_64

### Node.js Bindings
- **Linux**: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu
- **macOS**: x86_64-apple-darwin, aarch64-apple-darwin
- **Windows**: x86_64-pc-windows-msvc
- **Node.js**: Version 20+ required

### WebAssembly
- **Browsers**: All modern browsers with WebAssembly support
- **Node.js**: Version 20+ with WebAssembly support
- **Bundlers**: Webpack, Rollup, Vite, etc.

## Troubleshooting

### Build Failures

1. **Cross-compilation issues**: Ensure all required cross-compilation tools are installed
2. **Node.js binding failures**: Check NAPI-RS configuration and Rust toolchain
3. **WASM build failures**: Verify wasm-pack installation and wasm32-unknown-unknown target

### Publishing Failures

1. **crates.io**: Check token permissions and crate ownership
2. **npm**: Verify token has publish permissions for `@octofhir` scope
3. **Version conflicts**: Ensure version numbers are incremented properly

### Secret Configuration

1. Go to GitHub repository → Settings → Secrets and variables → Actions
2. Add the required secrets listed above
3. Ensure secret names match exactly (case-sensitive)

## Verification

After a successful release, verify:

1. **GitHub Release**: Check that binaries are attached and downloadable
2. **crates.io**: Verify packages are published and installable
3. **npm**: Check that packages are available and installable
4. **Functionality**: Test CLI and library functionality on different platforms

## Version Strategy

The project follows [Semantic Versioning (SemVer)](https://semver.org/):
- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible functionality additions
- **PATCH**: Backwards-compatible bug fixes

All packages (Rust crates, Node.js, WASM) maintain synchronized versions.

## Support

For release-related issues:
1. Check GitHub Actions logs for detailed error messages
2. Verify all required secrets are configured
3. Ensure version numbers are properly incremented
4. Open an issue if problems persist
