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

//! Bridge support types for fhirpath-model integration
//!
//! This module defines additional bridge types needed for choice type resolution
//! and resource information that extend the base bridge_support types.

use serde::{Deserialize, Serialize};

/// Choice type information for bridge support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeChoiceInfo {
    /// The original choice type path (e.g., "value[x]")
    pub original_path: String,

    /// The specific property name (e.g., "valueString")
    pub resolved_property: String,

    /// The resolved type name (e.g., "string")
    pub resolved_type: String,

    /// Whether this choice resolution is valid
    pub is_valid: bool,

    /// List of all possible choice variants for this path
    pub possible_variants: Vec<String>,

    /// Cardinality information for the resolved choice
    pub cardinality: octofhir_fhirschema::types::BridgeCardinality,

    /// Additional metadata about the choice type
    pub metadata: Option<BridgeChoiceMetadata>,
}

/// Resource information for bridge support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeResourceInfo {
    /// Resource type name
    pub resource_type: String,

    /// Base resource type (e.g., "DomainResource", "Resource")
    pub base_type: Option<String>,

    /// Whether this is an abstract resource type
    pub is_abstract: bool,

    /// List of all properties available on this resource
    pub properties: Vec<String>,

    /// Profile URLs that this resource conforms to
    pub profiles: Vec<String>,

    /// Namespace (typically "FHIR")
    pub namespace: String,
}

/// Additional metadata for choice type resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeChoiceMetadata {
    /// Human-readable description of the choice type
    pub description: Option<String>,

    /// Whether this choice type is required
    pub is_required: bool,

    /// Default type for the choice (if any)
    pub default_type: Option<String>,

    /// Constraints specific to this choice type
    pub constraints: Vec<String>,
}

impl BridgeChoiceInfo {
    /// Create a new BridgeChoiceInfo for a valid choice resolution
    pub fn valid(
        original_path: String,
        resolved_property: String,
        resolved_type: String,
        cardinality: octofhir_fhirschema::types::BridgeCardinality,
    ) -> Self {
        Self {
            original_path,
            resolved_property,
            resolved_type,
            is_valid: true,
            possible_variants: Vec::new(),
            cardinality,
            metadata: None,
        }
    }

    /// Create a new BridgeChoiceInfo for an invalid choice resolution
    pub fn invalid(original_path: String, resolved_property: String) -> Self {
        Self {
            original_path,
            resolved_property,
            resolved_type: String::new(),
            is_valid: false,
            possible_variants: Vec::new(),
            cardinality: octofhir_fhirschema::types::BridgeCardinality::new(0, Some(1)),
            metadata: None,
        }
    }

    /// Add possible variants to the choice info
    pub fn with_variants(mut self, variants: Vec<String>) -> Self {
        self.possible_variants = variants;
        self
    }

    /// Add metadata to the choice info
    pub fn with_metadata(mut self, metadata: BridgeChoiceMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl BridgeResourceInfo {
    /// Create a new BridgeResourceInfo
    pub fn new(resource_type: String, namespace: String) -> Self {
        Self {
            resource_type,
            base_type: None,
            is_abstract: false,
            properties: Vec::new(),
            profiles: Vec::new(),
            namespace,
        }
    }

    /// Set the base type
    pub fn with_base_type(mut self, base_type: String) -> Self {
        self.base_type = Some(base_type);
        self
    }

    /// Set whether this is an abstract type
    pub fn with_abstract(mut self, is_abstract: bool) -> Self {
        self.is_abstract = is_abstract;
        self
    }

    /// Add properties to the resource info
    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.properties = properties;
        self
    }

    /// Add profiles to the resource info
    pub fn with_profiles(mut self, profiles: Vec<String>) -> Self {
        self.profiles = profiles;
        self
    }
}

impl Default for BridgeChoiceMetadata {
    fn default() -> Self {
        Self {
            description: None,
            is_required: false,
            default_type: None,
            constraints: Vec::new(),
        }
    }
}
