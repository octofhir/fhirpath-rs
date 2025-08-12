#!/bin/bash
# scripts/restore-dev-dependencies.sh
#
# Restore development dependencies by removing version specifications from
# workspace dependencies in Cargo.toml files and restoring original versions.

set -e

DEV_VERSION="0.4.0"
CANARY_VERSION_PATTERN="0\.4\.0-canary\.[0-9]\{14\}\.[a-f0-9]\{7\}"

echo "🔄 Restoring development dependencies"

# Function to restore path-only dependencies and clean up canary versions
restore_cargo_toml() {
    local cargo_file="$1"
    echo "📝 Restoring $cargo_file"
    
    # Remove version specifications from workspace dependencies, keep only path
    # Transform: { version = "VERSION", path = "../crate-name" } -> { path = "../crate-name" }
    sed -i.bak 's|{ version = "'$CANARY_VERSION_PATTERN'", path = "\.\./|{ path = "../|g' "$cargo_file"
    sed -i.bak2 's|{ version = "'$DEV_VERSION'", path = "\.\./|{ path = "../|g' "$cargo_file"
    
    echo "✅ Restored $cargo_file"
    rm "$cargo_file.bak" 2>/dev/null || true
    rm "$cargo_file.bak2" 2>/dev/null || true
}

# Function to restore root workspace version
restore_root_version() {
    local cargo_file="Cargo.toml"
    echo "📝 Restoring root workspace version in $cargo_file"
    
    # Restore workspace version from canary back to dev version
    sed -i.bak 's|version = "'$CANARY_VERSION_PATTERN'"|version = "'$DEV_VERSION'"|g' "$cargo_file"
    
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