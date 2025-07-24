//! Error types for the model crate

use thiserror::Error;

/// Result type alias for model operations
pub type Result<T> = std::result::Result<T, ModelError>;

/// Model-specific error types
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ModelError {
    /// Invalid type conversion
    #[error("Cannot convert {from} to {to}")]
    ConversionError {
        /// Source type
        from: String,
        /// Target type
        to: String,
    },

    /// Invalid quantity unit
    #[error("Invalid quantity unit: {unit}")]
    InvalidUnit {
        /// The invalid unit
        unit: String,
    },

    /// Incompatible units for operation
    #[error("Incompatible units: '{left}' and '{right}'")]
    IncompatibleUnits {
        /// Left unit
        left: String,
        /// Right unit
        right: String,
    },

    /// Invalid date/time format
    #[error("Invalid date/time format: {value}")]
    InvalidDateTime {
        /// The invalid value
        value: String,
    },

    /// Schema loading error
    #[cfg(feature = "async-schema")]
    #[error("Failed to load FHIR Schema: {message}")]
    SchemaLoadError {
        /// Error message
        message: String,
    },

    /// Type not found in schema
    #[error("Type '{type_name}' not found in FHIR Schema")]
    TypeNotFound {
        /// Type name
        type_name: String,
    },

    /// Property not found
    #[error("Property '{property}' not found on type '{type_name}'")]
    PropertyNotFound {
        /// Type name
        type_name: String,
        /// Property name
        property: String,
    },
}

impl ModelError {
    /// Create a conversion error
    pub fn conversion_error(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::ConversionError {
            from: from.into(),
            to: to.into(),
        }
    }

    /// Create an invalid unit error
    pub fn invalid_unit(unit: impl Into<String>) -> Self {
        Self::InvalidUnit {
            unit: unit.into(),
        }
    }

    /// Create an incompatible units error
    pub fn incompatible_units(left: impl Into<String>, right: impl Into<String>) -> Self {
        Self::IncompatibleUnits {
            left: left.into(),
            right: right.into(),
        }
    }

    /// Create an invalid date/time error
    pub fn invalid_datetime(value: impl Into<String>) -> Self {
        Self::InvalidDateTime {
            value: value.into(),
        }
    }

    /// Create a schema load error
    #[cfg(feature = "async-schema")]
    pub fn schema_load_error(message: impl Into<String>) -> Self {
        Self::SchemaLoadError {
            message: message.into(),
        }
    }

    /// Create a type not found error
    pub fn type_not_found(type_name: impl Into<String>) -> Self {
        Self::TypeNotFound {
            type_name: type_name.into(),
        }
    }

    /// Create a property not found error
    pub fn property_not_found(type_name: impl Into<String>, property: impl Into<String>) -> Self {
        Self::PropertyNotFound {
            type_name: type_name.into(),
            property: property.into(),
        }
    }
}