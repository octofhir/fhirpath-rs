#!/bin/bash
# scripts/prepare-release.sh
#
# Prepare the workspace for publishing by updating Cargo.toml files to include
# version specifications for workspace dependencies.

set -e

VERSION="0.4.0"

echo "🚀 Preparing workspace for release v$VERSION"

# Function to update a Cargo.toml file with version specifications
update_cargo_toml() {
    local cargo_file="$1"
    echo "📝 Updating $cargo_file"
    
    # Add version specifications to workspace dependencies
    # Transform: { path = "../crate-name" } -> { version = "VERSION", path = "../crate-name" }
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