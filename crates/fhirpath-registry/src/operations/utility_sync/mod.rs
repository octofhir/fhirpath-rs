//! Utility operations - sync implementations
//! 
//! These operations handle encoding/decoding, string escaping, debugging, and data validation.
//! All operations are pure data processing and don't require I/O.

pub mod has_value;
pub mod comparable;
pub mod encode;
pub mod decode;
pub mod escape;
pub mod unescape;
pub mod trace;
pub mod define_variable;

pub use has_value::HasValueFunction;
pub use comparable::ComparableFunction;
pub use encode::EncodeFunction;
pub use decode::DecodeFunction;
pub use escape::EscapeFunction;
pub use unescape::UnescapeFunction;
pub use trace::TraceFunction;
pub use define_variable::DefineVariableFunction;