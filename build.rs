//! Build script for FHIRPath project
//!
//! This script automatically builds the SolidJS web UI during cargo build
//! to embed it as static assets in the CLI binary.

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Skip UI build entirely when building for docs.rs or during documentation generation
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=Skipping UI build for docs.rs");
        return;
    }

    // Skip UI build when generating documentation
    if env::var("RUSTDOCFLAGS").is_ok() || env::args().any(|arg| arg.contains("doc")) {
        println!("cargo:warning=Skipping UI build during documentation generation");
        return;
    }

    // Get the workspace root directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = Path::new(&manifest_dir).parent().unwrap().parent().unwrap(); // Go from crates/octofhir-fhirpath to workspace root
    let ui_dir = workspace_root.join("ui");

    // Skip UI build when being built as a dependency (e.g., from crates.io)
    // Only skip if UI directory doesn't exist (which means we're building from crates.io)
    if !ui_dir.exists() {
        println!(
            "cargo:warning=Skipping UI build - UI directory not found (likely building as dependency from crates.io)"
        );
        return;
    }

    // Re-run if UI files change (only when not building docs)
    println!("cargo:rerun-if-changed=ui/src");
    println!("cargo:rerun-if-changed=ui/package.json");
    println!("cargo:rerun-if-changed=ui/vite.config.ts");
    println!("cargo:rerun-if-changed=ui/tsconfig.json");

    // Only build UI if we're in the workspace root and not in a build dependency context
    let pkg_name = env::var("CARGO_PKG_NAME").unwrap_or_default();
    println!("cargo:warning=Build script running for package: {pkg_name}");
    if pkg_name != "octofhir-fhirpath" {
        println!("cargo:warning=Skipping UI build for package: {pkg_name}");
        return;
    }

    let dist_dir = workspace_root.join("dist/ui");

    // Check if UI directory exists
    if !ui_dir.exists() {
        println!("cargo:warning=UI directory not found, skipping UI build");
        create_fallback_ui(&dist_dir);
        return;
    }

    println!("cargo:warning=Building web UI...");

    // Ensure pnpm is available
    if !command_exists("pnpm") {
        println!(
            "cargo:warning=pnpm not found, skipping UI build. Install with: npm install -g pnpm"
        );
        create_fallback_ui(&dist_dir);
        return;
    }

    // Create dist directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(workspace_root.join("dist")) {
        println!("cargo:warning=Failed to create dist directory: {}", e);
        create_fallback_ui(&dist_dir);
        return;
    }

    // Install dependencies
    let install_status = Command::new("pnpm")
        .args(&["install", "--frozen-lockfile"])
        .current_dir(&ui_dir)
        .status();

    match install_status {
        Ok(status) if status.success() => {
            println!("cargo:warning=UI dependencies installed successfully");
        }
        Ok(_) => {
            println!("cargo:warning=Failed to install UI dependencies, skipping UI build");
            create_fallback_ui(&dist_dir);
            return;
        }
        Err(e) => {
            println!(
                "cargo:warning=Failed to run pnpm install: {}, skipping UI build",
                e
            );
            create_fallback_ui(&dist_dir);
            return;
        }
    }

    // Build the UI
    let build_status = Command::new("pnpm")
        .args(&["build"])
        .current_dir(&ui_dir)
        .status();

    match build_status {
        Ok(status) if status.success() => {
            println!("cargo:warning=UI built successfully");

            // Verify that the build output exists
            if dist_dir.join("index.html").exists() {
                println!("cargo:warning=UI assets ready for embedding");
            } else {
                println!("cargo:warning=UI build completed but index.html not found");
            }
        }
        Ok(_) => {
            println!("cargo:warning=UI build failed, continuing without embedded UI");
            create_fallback_ui(&dist_dir);
        }
        Err(e) => {
            println!(
                "cargo:warning=Failed to run UI build: {}, continuing without embedded UI",
                e
            );
            create_fallback_ui(&dist_dir);
        }
    }
}

/// Check if a command exists in the system PATH
fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Create a fallback UI directory with a basic HTML file to prevent include_dir! from failing
fn create_fallback_ui(dist_dir: &Path) {
    if let Err(e) = fs::create_dir_all(dist_dir) {
        println!(
            "cargo:warning=Failed to create fallback dist/ui directory: {}",
            e
        );
        return;
    }

    let fallback_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>FHIRPath Server</title>
    <style>
        body { 
            font-family: system-ui, sans-serif; 
            margin: 0; 
            padding: 2rem; 
            background: #0a0a0f; 
            color: #f1f5f9; 
            text-align: center; 
        }
        h1 { color: #b347d9; }
        .container { max-width: 600px; margin: 0 auto; }
        .info { 
            background: #16213e; 
            padding: 1.5rem; 
            border-radius: 0.5rem; 
            margin-top: 2rem; 
            border: 1px solid #334155;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🐙 FHIRPath Server</h1>
        <p>The server is running, but the web UI is not available.</p>
        <div class="info">
            <h3>To enable the web UI:</h3>
            <p>1. Navigate to the project directory<br>
            2. Run: <code>cd ui && pnpm install && pnpm build</code><br>
            3. Restart the server</p>
        </div>
        <p><strong>API endpoints are still available at:</strong><br>
        <code>POST /r4/evaluate</code>, <code>POST /r4/analyze</code>, etc.</p>
    </div>
</body>
</html>"#;

    let index_path = dist_dir.join("index.html");
    if let Err(e) = fs::write(&index_path, fallback_html) {
        println!("cargo:warning=Failed to create fallback index.html: {}", e);
    }
}
