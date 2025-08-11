// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
