//! FHIR version handling for server endpoints

use crate::cli::server::error::{ServerError, ServerResult};
use octofhir_fhirpath_model::provider::FhirVersion;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Supported FHIR versions for the server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerFhirVersion {
    R4,
    R4B,
    R5,
    R6,
}

impl ServerFhirVersion {
    /// Get all supported FHIR versions
    pub fn all() -> &'static [ServerFhirVersion] {
        &[
            ServerFhirVersion::R4,
            ServerFhirVersion::R4B,
            ServerFhirVersion::R5,
            ServerFhirVersion::R6,
        ]
    }

    /// Get the version as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            ServerFhirVersion::R4 => "r4",
            ServerFhirVersion::R4B => "r4b",
            ServerFhirVersion::R5 => "r5",
            ServerFhirVersion::R6 => "r6",
        }
    }

    /// Convert to the model provider's FhirVersion enum
    pub fn to_model_version(&self) -> FhirVersion {
        match self {
            ServerFhirVersion::R4 => FhirVersion::R4,
            ServerFhirVersion::R4B => FhirVersion::R4B,
            ServerFhirVersion::R5 => FhirVersion::R5,
            ServerFhirVersion::R6 => FhirVersion::R5, // R6 uses R5 schema for now
        }
    }
}

impl FromStr for ServerFhirVersion {
    type Err = ServerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "r4" => Ok(ServerFhirVersion::R4),
            "r4b" => Ok(ServerFhirVersion::R4B),
            "r5" => Ok(ServerFhirVersion::R5),
            "r6" => Ok(ServerFhirVersion::R6),
            _ => Err(ServerError::InvalidFhirVersion {
                version: s.to_string(),
            }),
        }
    }
}

impl std::fmt::Display for ServerFhirVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Extract FHIR version from request path
pub fn extract_version_from_path(path: &str) -> ServerResult<ServerFhirVersion> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    if parts.is_empty() {
        return Err(ServerError::BadRequest {
            message: "Missing FHIR version in path".to_string(),
        });
    }

    ServerFhirVersion::from_str(parts[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_from_str() {
        assert_eq!(
            "r4".parse::<ServerFhirVersion>().unwrap(),
            ServerFhirVersion::R4
        );
        assert_eq!(
            "R4".parse::<ServerFhirVersion>().unwrap(),
            ServerFhirVersion::R4
        );
        assert_eq!(
            "r4b".parse::<ServerFhirVersion>().unwrap(),
            ServerFhirVersion::R4B
        );
        assert_eq!(
            "r5".parse::<ServerFhirVersion>().unwrap(),
            ServerFhirVersion::R5
        );
        assert_eq!(
            "r6".parse::<ServerFhirVersion>().unwrap(),
            ServerFhirVersion::R6
        );

        assert!("invalid".parse::<ServerFhirVersion>().is_err());
    }

    #[test]
    fn test_extract_version_from_path() {
        assert_eq!(
            extract_version_from_path("/r4/evaluate").unwrap(),
            ServerFhirVersion::R4
        );
        assert_eq!(
            extract_version_from_path("r4b/analyze").unwrap(),
            ServerFhirVersion::R4B
        );
        assert_eq!(
            extract_version_from_path("/r5/evaluate").unwrap(),
            ServerFhirVersion::R5
        );

        assert!(extract_version_from_path("").is_err());
        assert!(extract_version_from_path("/").is_err());
        assert!(extract_version_from_path("/invalid/evaluate").is_err());
    }
}
