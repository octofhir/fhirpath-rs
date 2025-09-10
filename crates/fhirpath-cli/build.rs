//! Build script for FHIRPath CLI
//!
//! Simplified build script for the API-only FHIRPath server.
//! UI functionality has been removed.

fn main() {
    // No build steps required for API-only server
    println!(
        "cargo:warning=fhirpath-cli@{}: Building API-only server",
        env!("CARGO_PKG_VERSION")
    );
}
