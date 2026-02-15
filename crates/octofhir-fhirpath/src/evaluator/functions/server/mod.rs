//! Server functions for FHIRPath %server variable
//!
//! This module contains server functions that enable FHIR RESTful API operations
//! through the %server system variable.

pub mod apply_function;
pub mod capabilities_function;
pub mod create_function;
pub mod delete_function;
pub mod everything_function;
pub mod patch_function;
pub mod read_function;
pub mod search_function;
pub mod server_at_function;
pub mod transform_function;
pub mod update_function;
pub mod validate_function;

pub use apply_function::ServerApplyFunctionEvaluator;
pub use capabilities_function::ServerCapabilitiesFunctionEvaluator;
pub use create_function::ServerCreateFunctionEvaluator;
pub use delete_function::ServerDeleteFunctionEvaluator;
pub use everything_function::ServerEverythingFunctionEvaluator;
pub use patch_function::ServerPatchFunctionEvaluator;
pub use read_function::ServerReadFunctionEvaluator;
pub use search_function::ServerSearchFunctionEvaluator;
pub use server_at_function::ServerAtFunctionEvaluator;
pub use transform_function::ServerTransformFunctionEvaluator;
pub use update_function::ServerUpdateFunctionEvaluator;
pub use validate_function::ServerValidateFunctionEvaluator;
