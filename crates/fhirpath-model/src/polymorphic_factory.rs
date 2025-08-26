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

//! Factory utilities for creating PolymorphicPathResolver instances

use sonic_rs::Value;
use std::sync::Arc;

use crate::polymorphic_resolver::PolymorphicPathResolver;
use crate::provider::ModelProvider;
use crate::{ChoiceTypeMapper, FhirPathError};

/// Polymorphic resolver factory for easy creation
pub struct PolymorphicResolverFactory;

impl PolymorphicResolverFactory {
    /// Create a new resolver that dynamically discovers choice types from FHIRSchema
    pub fn create_dynamic_resolver(
        model_provider: Arc<dyn ModelProvider>,
    ) -> PolymorphicPathResolver {
        PolymorphicPathResolver::new(model_provider)
    }

    /// Create resolver with FHIRSchema discovery
    pub async fn create_with_schema(
        model_provider: Arc<dyn ModelProvider>,
        schema: &Value,
        choice_mapper: Option<Arc<ChoiceTypeMapper>>,
    ) -> Result<PolymorphicPathResolver, FhirPathError> {
        PolymorphicPathResolver::new_with_schema_discovery(model_provider, schema, choice_mapper)
            .await
    }

    /// Create resolver with default FHIR R4 patterns only
    pub fn create_default(model_provider: Arc<dyn ModelProvider>) -> PolymorphicPathResolver {
        let mapper = Arc::new(ChoiceTypeMapper::new_fhir_r4());
        PolymorphicPathResolver::new_with_mapper(model_provider, mapper)
    }
}