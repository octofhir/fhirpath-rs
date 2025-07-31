//! Built-in extensions for FHIRPath

pub mod cda;
pub mod fhir;

pub use cda::CdaExtension;
pub use fhir::FhirExtension;

use super::{ExtensionManager, ExtensionResult};

/// Load all built-in extensions into the extension manager
pub fn load_builtin_extensions(manager: &mut ExtensionManager) -> ExtensionResult<()> {
    // Load CDA extension
    let cda_extension = Box::new(CdaExtension::new());
    manager.load_extension(cda_extension)?;

    // Load FHIR extension
    let fhir_extension = Box::new(FhirExtension::new());
    manager.load_extension(fhir_extension)?;

    Ok(())
}
