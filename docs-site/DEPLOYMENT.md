# Documentation Site Deployment

This document describes the automated deployment process for the OctoFHIR FHIRPath documentation site to GitHub Pages.

## Overview

The documentation site is built using [Astro](https://astro.build/) with the [Starlight](https://starlight.astro.build/) theme and automatically deployed to GitHub Pages using GitHub Actions.

## Deployment Workflow

The deployment is handled by the `.github/workflows/docs.yml` workflow, which:

1. **Builds WASM Package**: Compiles the Rust code to WebAssembly for the interactive playground
2. **Copies WASM Files**: Places WASM files in the documentation site's public directory
3. **Copies Comparison Data**: Copies test results and benchmark data from `fhirpath-comparison/results`
4. **Builds Documentation**: Compiles the Astro site with proper base URL for GitHub Pages
5. **Deploys to GitHub Pages**: Publishes the built site to the `gh-pages` branch

## Trigger Conditions

The workflow runs on:
- **Push to main branch**: Builds and deploys the site
- **Pull requests to main**: Builds the site for testing (no deployment)
- **Manual trigger**: Can be triggered manually from GitHub Actions tab

## Prerequisites

### Repository Settings

1. **GitHub Pages Configuration**:
   - Go to repository Settings → Pages
   - Set Source to "GitHub Actions"
   - The site will be available at `https://octofhir.github.io/fhirpath-rs`

2. **Required Permissions**:
   The workflow uses the following permissions:
   - `contents: read` - To checkout the repository
   - `pages: write` - To deploy to GitHub Pages
   - `id-token: write` - For GitHub Pages deployment authentication

### Dependencies

The build process requires:
- **Rust toolchain** with `wasm32-unknown-unknown` target
- **wasm-pack** for building WebAssembly packages
- **Node.js 20+** for building the documentation site
- **Comparison data** from `fhirpath-comparison/results` directory

## Build Process Details

### 1. WASM Package Build

```bash
cd fhirpath-wasm
npm run build
```

This creates WebAssembly files in `fhirpath-wasm/pkg/` including:
- `fhirpath_wasm.js` - JavaScript bindings
- `fhirpath_wasm_bg.wasm` - WebAssembly binary
- `fhirpath_wasm.d.ts` - TypeScript definitions

### 2. File Copying

**WASM Files**:
```bash
mkdir -p docs-site/public/pkg
cp fhirpath-wasm/pkg/* docs-site/public/pkg/
```

**Comparison Data**:
```bash
node docs-site/scripts/copy-comparison-data.js
```

This copies JSON files from `fhirpath-comparison/results/`:
- `comparison_report.json`
- `*_test_results.json` (for each language)
- `*_benchmark_results.json` (for each language)

### 3. Documentation Build

```bash
cd docs-site
npm run build
```

This runs Astro build with:
- Base URL: `/fhirpath-rs` (for GitHub Pages)
- Environment variable: `PUBLIC_BASE_URL=/fhirpath-rs`

## Local Development

To run the documentation site locally:

```bash
# Build WASM package
cd fhirpath-wasm
npm run build

# Install dependencies and run dev server
cd ../docs-site
npm ci
npm run prebuild  # Copies WASM and comparison data
npm run dev
```

The site will be available at `http://localhost:4321`

## Interactive Features

The documentation site includes:

1. **FHIRPath Playground** (`/playground`):
   - Interactive FHIRPath expression evaluation
   - Powered by WebAssembly for client-side execution
   - Requires WASM package to be built and copied

2. **Performance Comparison** (`/comparison`):
   - Charts and tables comparing different FHIRPath implementations
   - Uses data from `fhirpath-comparison/results`
   - Requires comparison data to be copied

3. **Test Compliance** (`/reference/test-compliance`):
   - Shows compliance with official FHIRPath test suite
   - Uses test result data from comparison directory

## Troubleshooting

### Common Issues

1. **WASM Build Failures**:
   - Ensure Rust toolchain is installed with `wasm32-unknown-unknown` target
   - Check that `wasm-pack` is properly installed
   - Verify `fhirpath-wasm/Cargo.toml` configuration

2. **Missing Comparison Data**:
   - Ensure `fhirpath-comparison/results` directory exists
   - Check that comparison data files are present
   - Verify `copy-comparison-data.js` script runs without errors

3. **Deployment Failures**:
   - Check GitHub Pages is enabled in repository settings
   - Verify workflow permissions are correctly set
   - Ensure the workflow has access to required secrets

4. **Base URL Issues**:
   - The site uses `/fhirpath-rs` as base URL for GitHub Pages
   - Local development uses root URL (`/`)
   - Check `astro.config.mjs` for base URL configuration

### Debugging

To debug build issues:

1. Check GitHub Actions logs for detailed error messages
2. Run the build process locally to reproduce issues
3. Verify all dependencies are properly installed
4. Check file permissions and directory structure

## Site Structure

```
docs-site/
├── src/
│   ├── content/docs/          # Documentation content
│   ├── components/            # Astro/Svelte components
│   └── pages/                 # Special pages (playground, comparison)
├── public/
│   ├── pkg/                   # WASM files (copied during build)
│   └── *.json                 # Comparison data (copied during build)
├── scripts/
│   └── copy-comparison-data.js # Data copying script
└── dist/                      # Built site (generated)
```

## Maintenance

- **Update Dependencies**: Regularly update Astro, Starlight, and other dependencies
- **Monitor Build Times**: WASM compilation can be slow; consider caching strategies
- **Check Links**: Verify internal and external links work correctly
- **Test Interactive Features**: Ensure playground and comparison tools function properly

For more information about the documentation framework, see the [Starlight documentation](https://starlight.astro.build/).
