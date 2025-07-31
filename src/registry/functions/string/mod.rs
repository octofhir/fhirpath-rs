//! String manipulation functions for FHIRPath expressions

mod contains;
mod decode;
mod encode;
mod ends_with;
mod escape;
mod index_of;
mod join;
mod lower;
mod matches;
mod matches_full;
mod replace;
mod replace_matches;
mod split;
mod starts_with;
mod substring;
mod to_chars;
mod trim;
mod unescape;
mod upper;

pub use contains::ContainsFunction;
pub use decode::DecodeFunction;
pub use encode::EncodeFunction;
pub use ends_with::EndsWithFunction;
pub use escape::EscapeFunction;
pub use index_of::IndexOfFunction;
pub use join::JoinFunction;
pub use lower::LowerFunction;
pub use matches::MatchesFunction;
pub use matches_full::MatchesFullFunction;
pub use replace::ReplaceFunction;
pub use replace_matches::ReplaceMatchesFunction;
pub use split::SplitFunction;
pub use starts_with::StartsWithFunction;
pub use substring::SubstringFunction;
pub use to_chars::ToCharsFunction;
pub use trim::TrimFunction;
pub use unescape::UnescapeFunction;
pub use upper::UpperFunction;

use crate::registry::function::FunctionRegistry;

/// Register all string functions
pub fn register_string_functions(registry: &mut FunctionRegistry) {
    registry.register(SubstringFunction);
    registry.register(StartsWithFunction);
    registry.register(EndsWithFunction);
    registry.register(ContainsFunction);
    registry.register(MatchesFunction);
    registry.register(MatchesFullFunction);
    registry.register(ReplaceFunction);
    registry.register(ReplaceMatchesFunction);
    registry.register(SplitFunction);
    registry.register(JoinFunction);
    registry.register(TrimFunction);
    registry.register(ToCharsFunction);
    registry.register(IndexOfFunction);
    registry.register(UpperFunction);
    registry.register(LowerFunction);
    registry.register(EncodeFunction);
    registry.register(DecodeFunction);
    registry.register(EscapeFunction);
    registry.register(UnescapeFunction);
}
