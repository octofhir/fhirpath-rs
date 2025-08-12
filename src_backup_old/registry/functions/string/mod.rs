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
    registry.register_async(SubstringFunction);
    registry.register_async(StartsWithFunction);
    registry.register_async(EndsWithFunction);
    registry.register_async(ContainsFunction);
    registry.register_async(MatchesFunction);
    registry.register_async(MatchesFullFunction);
    registry.register_async(ReplaceFunction);
    registry.register_async(ReplaceMatchesFunction);
    registry.register_async(SplitFunction);
    registry.register_async(JoinFunction);
    registry.register_async(TrimFunction);
    registry.register_async(ToCharsFunction);
    registry.register_async(IndexOfFunction);
    registry.register_async(UpperFunction);
    registry.register_async(LowerFunction);
    registry.register_async(EncodeFunction);
    registry.register_async(DecodeFunction);
    registry.register_async(EscapeFunction);
    registry.register_async(UnescapeFunction);
}
