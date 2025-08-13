#!/bin/bash
# scripts/restore-dev-dependencies.sh
#
# Restore development dependencies by removing version specifications from
# workspace dependencies in Cargo.toml files and restoring original versions.

set -e

# Extract the base version from current workspace version
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

# Handle different version patterns:
# 1. Canary versions: X.Y.Z-canary.timestamp.hash -> X.Y.Z
# 2. Regular semver: X.Y.Z -> X.Y.Z (use as-is)
if [[ "$CURRENT_VERSION" =~ ^([0-9]+\.[0-9]+\.[0-9]+)-canary\. ]]; then
    DEV_VERSION="${BASH_REMATCH[1]}"
    echo "🔍 Detected canary version $CURRENT_VERSION, extracting base version: $DEV_VERSION"
elif [[ "$CURRENT_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    DEV_VERSION="$CURRENT_VERSION"
    echo "🔍 Detected regular semver version: $DEV_VERSION"
else
    # Fallback for any other version format
    DEV_VERSION="$CURRENT_VERSION"
    echo "🔍 Using current version as-is: $DEV_VERSION"
fi

echo "🔄 Restoring development dependencies"

# Function to restore path-only dependencies and clean up canary versions
restore_cargo_toml() {
    local cargo_file="$1"
    echo "📝 Restoring $cargo_file"
    
    # Remove version specifications from workspace dependencies, keep only path and other properties
    # Handle both simple and complex dependency specifications:
    # { version = "VERSION", path = "../crate-name" } -> { path = "../crate-name" }
    # { version = "VERSION", path = "../crate-name", features = [...] } -> { path = "../crate-name", features = [...] }
    
    # Remove version specifications while preserving all other properties
    # This pattern finds and removes 'version = "VERSION", ' part
    local base_version_escaped=$(echo "$DEV_VERSION" | sed 's/\./\\./g')
    
    # Remove canary versions (if any exist)
    sed -i.bak 's|version = "'$base_version_escaped'-canary[^"]*", ||g' "$cargo_file"
    
    # Remove regular semver versions matching the current version
    sed -i.bak2 's|version = "'$DEV_VERSION'", ||g' "$cargo_file"
    
    echo "✅ Restored $cargo_file"
    rm "$cargo_file.bak" 2>/dev/null || true
    rm "$cargo_file.bak2" 2>/dev/null || true
}

# Function to restore root workspace version
restore_root_version() {
    local cargo_file="Cargo.toml"
    echo "📝 Restoring root workspace version in $cargo_file"
    
    # Restore workspace version - handle both canary and regular versions
    local base_version_escaped=$(echo "$DEV_VERSION" | sed 's/\./\\./g')
    
    # For canary versions, restore to base version
    sed -i.bak 's|version = "'$base_version_escaped'-canary[^"]*"|version = "'$DEV_VERSION'"|g' "$cargo_file"
    
    # For regular versions, no change needed (already correct)
    # This ensures the script is idempotent
    
    echo "✅ Restored root workspace version"
    rm "$cargo_file.bak" 2>/dev/null || true
}

# Restore root workspace version
restore_root_version

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