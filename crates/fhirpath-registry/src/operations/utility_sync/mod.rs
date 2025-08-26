//! Utility operations - sync implementations
//!
//! These operations handle encoding/decoding, string escaping, debugging, and data validation.
//! All operations are pure data processing and don't require I/O.

pub mod comparable;
pub mod decode;
pub mod define_variable;
pub mod encode;
pub mod escape;
pub mod has_value;
pub mod trace;
pub mod unescape;

pub use comparable::ComparableFunction;
pub use decode::DecodeFunction;
pub use define_variable::DefineVariableFunction;
pub use encode::EncodeFunction;
pub use escape::EscapeFunction;
pub use has_value::HasValueFunction;
pub use trace::TraceFunction;
pub use unescape::UnescapeFunction;
