#!/bin/bash
# scripts/restore-dev-dependencies.sh
#
# Restore development dependencies by removing version specifications from
# workspace dependencies in Cargo.toml files.

set -e

VERSION="0.4.0"

echo "🔄 Restoring development dependencies"

# Function to restore path-only dependencies
restore_cargo_toml() {
    local cargo_file="$1"
    echo "📝 Restoring $cargo_file"
    
    # Remove version specifications, keep only path
    # Transform: { version = "VERSION", path = "../crate-name" } -> { path = "../crate-name" }
    sed -i.bak 's|{ version = "'$VERSION'", path = "\.\./|{ path = "../|g' "$cargo_file"
    
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