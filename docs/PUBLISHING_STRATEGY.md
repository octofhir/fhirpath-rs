# Publishing Strategy for octofhir-fhirpath Workspace

This document outlines the complete strategy for publishing all crates in the octofhir-fhirpath workspace to crates.io.

## Problem

The workspace uses local path dependencies for development, which crates.io doesn't accept. We need to:

1. Convert path dependencies to version dependencies for publishing
2. Publish crates in the correct dependency order  
3. Restore path dependencies for continued development

## Solution: cargo-workspaces Tool

After investigation, the best approach is to use `cargo-workspaces`, which automates workspace publishing and handles dependency ordering automatically.

### Publishing Order (Verified by cargo-workspaces)

```bash
cargo workspaces plan
```

Output:
```
fhirpath-core
fhirpath-ast  
fhirpath-diagnostics
fhirpath-model
fhirpath-parser
fhirpath-registry
fhirpath-compiler
octofhir-fhirpath
```

Note: `fhirpath-evaluator`, `fhirpath-tools`, and `fhirpath-benchmarks` are missing from the current workspace setup.

## Implementation

### Step 1: Install cargo-workspaces

```bash
cargo install cargo-workspaces
```

### Step 2: Fix Dependency Specifications

Currently workspace dependencies look like:
```toml
[dependencies]
fhirpath-core = { path = "../fhirpath-core" }
```

They need to include version specifications:
```toml
[dependencies]  
fhirpath-core = { version = "0.4.0", path = "../fhirpath-core" }
```

### Step 3: Automated Preparation Script

Created `scripts/prepare-release.sh` to automatically add version specifications:

```bash
#!/bin/bash
# scripts/prepare-release.sh

set -e

VERSION="0.4.0"

echo "🚀 Preparing workspace for release v$VERSION"

# Function to update a Cargo.toml file with version specifications
update_cargo_toml() {
    local cargo_file="$1"
    echo "📝 Updating $cargo_file"
    
    # Add version specifications to workspace dependencies
    # Transform: { path = "../crate-name" } -> { version = "VERSION", path = "../crate-name" }
    sed -i.bak 's/{ path = "\.\.\//{ version = "'$VERSION'", path = "..//g' "$cargo_file"
    
    # Verify the change worked
    if grep -q 'version = "'$VERSION'", path = "' "$cargo_file"; then
        echo "✅ Updated $cargo_file successfully"
        rm "$cargo_file.bak" 2>/dev/null || true
    else
        echo "❌ No workspace dependencies found in $cargo_file (this is OK)"
        rm "$cargo_file.bak" 2>/dev/null || true
    fi
}

# Update all workspace Cargo.toml files
for crate in crates/*/Cargo.toml; do
    if [ -f "$crate" ]; then
        update_cargo_toml "$crate"
    fi
done

echo "✅ All Cargo.toml files updated successfully"
echo "🔍 Verifying publishing plan..."
cargo workspaces plan

echo ""
echo "🎯 Ready for publishing!"
echo "   Next steps:"
echo "   1. Test: cargo workspaces publish --dry-run"
echo "   2. Publish: cargo workspaces publish"
echo "   3. Restore: ./scripts/restore-dev-dependencies.sh"
```

### Step 4: Restoration Script  

Created `scripts/restore-dev-dependencies.sh` to restore development dependencies:

```bash
#!/bin/bash
# scripts/restore-dev-dependencies.sh

set -e

VERSION="0.4.0"

echo "🔄 Restoring development dependencies"

# Function to restore path-only dependencies
restore_cargo_toml() {
    local cargo_file="$1"
    echo "📝 Restoring $cargo_file"
    
    # Remove version specifications, keep only path
    # Transform: { version = "VERSION", path = "../crate-name" } -> { path = "../crate-name" }
    sed -i.bak 's/{ version = "'$VERSION'", path = "\.\.\//{ path = "..//g' "$cargo_file"
    
    echo "✅ Restored $cargo_file"
    rm "$cargo_file.bak" 2>/dev/null || true
}

# Restore all workspace Cargo.toml files
for crate in crates/*/Cargo.toml; do
    if [ -f "$crate" ]; then
        restore_cargo_toml "$crate"
    fi
done

echo "✅ All Cargo.toml files restored to development mode"
echo "🧪 Verifying workspace still builds..."
cargo check --workspace

echo "🎉 Development dependencies restored successfully!"
```

## Complete Publishing Workflow

### Standard Release Process:

```bash
# 1. Ensure clean working directory
git status

# 2. Prepare for publishing (adds version specs)
./scripts/prepare-release.sh

# 3. Test publishing (dry run)
cargo workspaces publish --dry-run

# 4. Publish to crates.io
cargo workspaces publish

# 5. Restore development dependencies
./scripts/restore-dev-dependencies.sh

# 6. Commit the release
git add -A
git commit -m "chore: release v0.4.0

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
git tag v0.4.0
git push origin main --tags
```

### One-liner for Quick Releases:

```bash
./scripts/prepare-release.sh && cargo workspaces publish && ./scripts/restore-dev-dependencies.sh
```

## Advantages

1. ✅ **Automated dependency order**: cargo-workspaces handles correct publishing sequence
2. ✅ **Single command**: One command publishes entire workspace  
3. ✅ **Development workflow preserved**: Path dependencies remain for fast builds
4. ✅ **Reliable**: Industry-standard tool used by major Rust projects
5. ✅ **Error handling**: Built-in validation and rollback capabilities
6. ✅ **Dry run support**: Test before actually publishing

## Troubleshooting

### Common Issues:

1. **"dependency does not specify a version"**: Run `./scripts/prepare-release.sh` first
2. **Git configuration errors**: `cargo-workspaces` needs git config setup
3. **Authentication**: Ensure `cargo login` is configured
4. **Network issues**: Use `--retry-count` flag for unreliable connections

### Manual Fallback:

If `cargo-workspaces` fails, individual crate publishing is possible:

```bash
# Publish in dependency order
cd crates/fhirpath-core && cargo publish
cd ../fhirpath-ast && cargo publish  
cd ../fhirpath-diagnostics && cargo publish
# ... continue in order
```

## Integration with CI/CD

This approach integrates well with GitHub Actions:

### Regular Releases (`publish.yml`)

```yaml
- name: Publish to crates.io
  run: |
    ./scripts/prepare-release.sh
    cargo workspaces publish --token ${{ secrets.CARGO_REGISTRY_TOKEN }}
    ./scripts/restore-dev-dependencies.sh
```

### Canary Releases (`canary-release.yml`)

Canary releases are automatic builds from the main branch with timestamped versions:

```yaml
- name: Prepare canary release
  run: |
    ./scripts/prepare-canary-release.sh "${{ steps.version.outputs.version }}"

- name: Publish canary
  run: |
    cargo workspaces publish --allow-dirty --token ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

**Canary Version Format:** `0.4.0-canary.20250812123456.abc1234`

## Release Types

1. **Regular Releases** (triggered by git tags like `v0.4.0`)
   - Stable versions: `0.4.0`, `0.5.0`, etc.
   - Full publishing workflow with clean commits
   
2. **Canary Releases** (triggered by pushes to main)
   - Preview versions: `0.4.0-canary.20250812123456.abc1234`
   - Automated testing and publishing
   - Includes binaries for CLI tools

## Required Secrets

Both workflows require a GitHub secret:
- `CARGO_REGISTRY_TOKEN`: crates.io API token for publishing