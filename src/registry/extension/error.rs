//! Error types for the extension system

use thiserror::Error;

/// Result type for extension operations
pub type ExtensionResult<T> = Result<T, ExtensionError>;

/// Errors that can occur in the extension system
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ExtensionError {
    /// Extension already registered
    #[error("Extension '{namespace}' is already registered")]
    AlreadyRegistered {
        /// The namespace that is already registered
        namespace: String,
    },

    /// Extension not found
    #[error("Extension '{namespace}' not found")]
    NotFound {
        /// The namespace that was not found
        namespace: String,
    },

    /// Invalid extension namespace
    #[error("Invalid extension namespace '{namespace}': {reason}")]
    InvalidNamespace {
        /// The invalid namespace
        namespace: String,
        /// Reason why the namespace is invalid
        reason: String,
    },

    /// Function name conflict
    #[error("Function '{function}' conflicts with existing function in namespace '{namespace}'")]
    FunctionConflict {
        /// The namespace where the conflict occurred
        namespace: String,
        /// The conflicting function name
        function: String,
    },

    /// Variable name conflict
    #[error("Variable '{variable}' conflicts with existing variable in namespace '{namespace}'")]
    VariableConflict {
        /// The namespace where the conflict occurred
        namespace: String,
        /// The conflicting variable name
        variable: String,
    },

    /// Extension initialization failed
    #[error("Extension '{namespace}' initialization failed: {reason}")]
    InitializationFailed {
        /// The namespace that failed to initialize
        namespace: String,
        /// Reason for initialization failure
        reason: String,
    },

    /// Extension cleanup failed
    #[error("Extension '{namespace}' cleanup failed: {reason}")]
    CleanupFailed {
        /// The namespace that failed to cleanup
        namespace: String,
        /// Reason for cleanup failure
        reason: String,
    },

    /// Invalid extension metadata
    #[error("Invalid extension metadata: {reason}")]
    InvalidMetadata {
        /// Reason why metadata is invalid
        reason: String,
    },

    /// Namespace conflict between extensions
    #[error("Namespace '{namespace}' is already used by extension '{existing_extension}'")]
    NamespaceConflict {
        /// The conflicting namespace
        namespace: String,
        /// The existing extension using this namespace
        existing_extension: String,
    },

    /// Extension dependency not found
    #[error("Extension '{namespace}' depends on '{dependency}' which is not loaded")]
    DependencyNotFound {
        /// The namespace with missing dependency
        namespace: String,
        /// The missing dependency name
        dependency: String,
    },

    /// Version compatibility error
    #[error(
        "Extension '{namespace}' version '{version}' is not compatible with core version '{core_version}'"
    )]
    VersionIncompatible {
        /// The namespace with version incompatibility
        namespace: String,
        /// The extension version
        version: String,
        /// The core version
        core_version: String,
    },
}

impl ExtensionError {
    /// Create an already registered error
    pub fn already_registered(namespace: impl Into<String>) -> Self {
        Self::AlreadyRegistered {
            namespace: namespace.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(namespace: impl Into<String>) -> Self {
        Self::NotFound {
            namespace: namespace.into(),
        }
    }

    /// Create an invalid namespace error
    pub fn invalid_namespace(namespace: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidNamespace {
            namespace: namespace.into(),
            reason: reason.into(),
        }
    }

    /// Create a function conflict error
    pub fn function_conflict(namespace: impl Into<String>, function: impl Into<String>) -> Self {
        Self::FunctionConflict {
            namespace: namespace.into(),
            function: function.into(),
        }
    }

    /// Create a variable conflict error
    pub fn variable_conflict(namespace: impl Into<String>, variable: impl Into<String>) -> Self {
        Self::VariableConflict {
            namespace: namespace.into(),
            variable: variable.into(),
        }
    }

    /// Create an initialization failed error
    pub fn initialization_failed(namespace: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InitializationFailed {
            namespace: namespace.into(),
            reason: reason.into(),
        }
    }

    /// Create a cleanup failed error
    pub fn cleanup_failed(namespace: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::CleanupFailed {
            namespace: namespace.into(),
            reason: reason.into(),
        }
    }

    /// Create an invalid metadata error
    pub fn invalid_metadata(reason: impl Into<String>) -> Self {
        Self::InvalidMetadata {
            reason: reason.into(),
        }
    }
}
