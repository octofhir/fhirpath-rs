#!/bin/bash
# scripts/restore-dev-dependencies.sh
#
# Restore development dependencies by removing version specifications from
# workspace dependencies in Cargo.toml files and restoring original versions.

set -e

# Extract the base version from any existing canary version in the workspace
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if [[ "$CURRENT_VERSION" =~ ^([0-9]+\.[0-9]+\.[0-9]+)-canary\. ]]; then
    DEV_VERSION="${BASH_REMATCH[1]}"
    echo "🔍 Detected canary version $CURRENT_VERSION, extracting base version: $DEV_VERSION"
else
    # Fallback to current version if it's not a canary version
    DEV_VERSION="$CURRENT_VERSION"
    echo "🔍 Current version is not canary, using as-is: $DEV_VERSION"
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
    # Use a general pattern to match any canary version for this major.minor.patch
    local base_version_escaped=$(echo "$DEV_VERSION" | sed 's/\./\\./g')
    sed -i.bak 's|version = "'$base_version_escaped'-canary[^"]*", ||g' "$cargo_file"
    sed -i.bak2 's|version = "'$DEV_VERSION'", ||g' "$cargo_file"
    
    echo "✅ Restored $cargo_file"
    rm "$cargo_file.bak" 2>/dev/null || true
    rm "$cargo_file.bak2" 2>/dev/null || true
}

# Function to restore root workspace version
restore_root_version() {
    local cargo_file="Cargo.toml"
    echo "📝 Restoring root workspace version in $cargo_file"
    
    # Restore workspace version from canary back to dev version
    # Use a general pattern to match any canary version for this major.minor.patch
    local base_version_escaped=$(echo "$DEV_VERSION" | sed 's/\./\\./g')
    sed -i.bak 's|version = "'$base_version_escaped'-canary[^"]*"|version = "'$DEV_VERSION'"|g' "$cargo_file"
    
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