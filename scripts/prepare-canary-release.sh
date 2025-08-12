#!/bin/bash
# scripts/prepare-canary-release.sh
#
# Prepare the workspace for canary publishing by updating Cargo.toml files
# with canary version specifications for workspace dependencies.

set -e

if [ -z "$1" ]; then
    echo "❌ Error: Canary version not provided"
    echo "Usage: $0 <canary-version>"
    echo "Example: $0 0.4.0-canary.20250812123456.abc1234"
    exit 1
fi

VERSION="$1"

echo "🚀 Preparing workspace for canary release v$VERSION"

# Function to update a Cargo.toml file with canary version specifications
update_cargo_toml_canary() {
    local cargo_file="$1"
    echo "📝 Updating $cargo_file for canary"
    
    # Add version specifications to workspace dependencies
    # Transform: { path = "../crate-name" } -> { version = "CANARY_VERSION", path = "../crate-name" }
    sed -i.bak 's|{ path = "\.\./|{ version = "'$VERSION'", path = "../|g' "$cargo_file"
    
    # Verify the change worked
    if grep -q 'version = "'$VERSION'", path = "' "$cargo_file"; then
        echo "✅ Updated $cargo_file successfully"
        rm "$cargo_file.bak" 2>/dev/null || true
    else
        echo "❌ No workspace dependencies found in $cargo_file (this is OK)"
        rm "$cargo_file.bak" 2>/dev/null || true
    fi
}

# Update workspace version first
echo "📝 Updating workspace version in root Cargo.toml"
sed -i.bak 's/^version = ".*"/version = "'$VERSION'"/' Cargo.toml
rm Cargo.toml.bak 2>/dev/null || true

# Update all workspace Cargo.toml files
for crate in crates/*/Cargo.toml; do
    if [ -f "$crate" ]; then
        update_cargo_toml_canary "$crate"
    fi
done

echo "✅ All Cargo.toml files updated for canary release v$VERSION"
echo "🔍 Verifying publishing plan..."

# Check if cargo-workspaces is available and show plan
if command -v cargo-workspaces &> /dev/null; then
    cargo workspaces plan
else
    echo "⚠️ cargo-workspaces not found, skipping plan verification"
fi

echo ""
echo "🎯 Ready for canary publishing!"
echo "   Next steps:"
echo "   1. Test: cargo workspaces publish --dry-run --allow-dirty"
echo "   2. Publish: cargo workspaces publish --allow-dirty"
echo "   3. Restore: ./scripts/restore-dev-dependencies.sh"