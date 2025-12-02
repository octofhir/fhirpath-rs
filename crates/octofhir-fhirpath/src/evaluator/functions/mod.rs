//! FHIRPath function implementations
//!
//! This module contains individual function implementations that are registered
//! with the FunctionRegistry for evaluation.

// Core collection functions
pub mod count_function;
pub mod empty_function;
pub mod exclude_function;
pub mod extension_function;
pub mod first_function;
pub mod last_function;
pub mod of_type_function;
pub mod select_function;
pub mod where_function;

// Advanced collection functions
pub mod combine_function;
pub mod distinct_function;
pub mod intersect_function;
pub mod single_function;
pub mod skip_function;
pub mod sort_function;
pub mod subset_of_function;
pub mod superset_of_function;
pub mod tail_function;
pub mod take_function;
pub mod union_function;

// Existence functions
pub mod all_function;
pub mod all_true_function;
pub mod any_true_function;
pub mod exists_function;

// Navigation functions (FHIRPath 3.0.0-ballot)
pub mod coalesce_function;
pub mod repeat_all_function;

// String manipulation functions
pub mod boundary_utils;
pub mod contains_function;
pub mod decode_function;
pub mod encode_function;
pub mod ends_with_function;
pub mod escape_function;
pub mod join_function;
pub mod lower_function;
pub mod replace_function;
pub mod replace_matches_function;
pub mod split_function;
pub mod starts_with_function;
pub mod to_chars_function;
pub mod trim_function;
pub mod unescape_function;
pub mod upper_function;

// Math functions
pub mod abs_function;
pub mod avg_function;
pub mod ceiling_function;
pub mod exp_function;
pub mod floor_function;
pub mod ln_function;
pub mod log_function;
pub mod max_function;
pub mod min_function;
pub mod power_function;
pub mod round_function;
pub mod sqrt_function;
pub mod sum_function;
pub mod truncate_function;

// Utility functions
pub mod define_variable_function;

// Temporal functions
pub mod day_of_function;
pub mod difference_function;
pub mod duration_function;
pub mod hour_of_function;
pub mod millisecond_function;
pub mod minute_of_function;
pub mod month_of_function;
pub mod now_function;
pub mod second_of_function;
pub mod timezone_offset_of_function;
pub mod today_function;
pub mod year_of_function;

// Validation functions
pub mod has_value_function;

// Logic functions
pub mod comparable_function;
pub mod not_function;

// Navigation functions
pub mod children_function;
pub mod descendants_function;
pub mod repeat_function;
pub mod resolve_function;

// Enhanced functions (FHIRPath 3.0.0-ballot)
pub mod high_boundary_function;
pub mod index_of_function;
pub mod is_distinct_function;
pub mod last_index_of_function;
pub mod low_boundary_function;
pub mod matches_full_function;
pub mod matches_function;
pub mod precision_function;

// Terminology functions (FHIRPath 3.0.0-ballot)
pub mod terminology;

// Conversion functions
pub mod conversion;

// Type checking functions
pub mod type_checking;

// Advanced functions (Phase 7)
pub mod aggregate_function;
pub mod iif_function;
pub mod length_function;
pub mod substring_function;

// Utility functions
pub mod trace_function;

// CDA functions
pub mod has_template_id_of_function;

// Re-export function evaluators explicitly

// Core collection functions
pub use count_function::CountFunctionEvaluator;
pub use empty_function::EmptyFunctionEvaluator;
pub use exclude_function::ExcludeFunctionEvaluator;
pub use extension_function::ExtensionFunctionEvaluator;
pub use first_function::FirstFunctionEvaluator;
pub use last_function::LastFunctionEvaluator;
pub use of_type_function::OfTypeFunctionEvaluator;
pub use select_function::SelectFunctionEvaluator;
pub use where_function::WhereFunctionEvaluator;

// Advanced collection functions
pub use combine_function::CombineFunctionEvaluator;
pub use distinct_function::DistinctFunctionEvaluator;
pub use intersect_function::IntersectFunctionEvaluator;
pub use single_function::SingleFunctionEvaluator;
pub use skip_function::SkipFunctionEvaluator;
pub use sort_function::SortFunctionEvaluator;
pub use subset_of_function::SubsetOfFunctionEvaluator;
pub use superset_of_function::SupersetOfFunctionEvaluator;
pub use tail_function::TailFunctionEvaluator;
pub use take_function::TakeFunctionEvaluator;
pub use union_function::UnionFunctionEvaluator;

// Existence functions
pub use all_function::AllFunctionEvaluator;
pub use all_true_function::AllTrueFunctionEvaluator;
pub use any_true_function::AnyTrueFunctionEvaluator;
pub use exists_function::ExistsFunctionEvaluator;

// Navigation functions
pub use coalesce_function::CoalesceFunctionEvaluator;
pub use repeat_all_function::RepeatAllFunctionEvaluator;

// String manipulation functions
pub use boundary_utils::NumericBoundaries;
pub use contains_function::ContainsFunctionEvaluator;
pub use decode_function::DecodeFunctionEvaluator;
pub use encode_function::EncodeFunctionEvaluator;
pub use ends_with_function::EndsWithFunctionEvaluator;
pub use escape_function::EscapeFunctionEvaluator;
pub use join_function::JoinFunctionEvaluator;
pub use lower_function::LowerFunctionEvaluator;
pub use replace_function::ReplaceFunctionEvaluator;
pub use replace_matches_function::ReplaceMatchesFunctionEvaluator;
pub use split_function::SplitFunctionEvaluator;
pub use starts_with_function::StartsWithFunctionEvaluator;
pub use to_chars_function::ToCharsFunctionEvaluator;
pub use trim_function::TrimFunctionEvaluator;
pub use unescape_function::UnescapeFunctionEvaluator;
pub use upper_function::UpperFunctionEvaluator;

// Math functions
pub use abs_function::AbsFunctionEvaluator;
pub use avg_function::AvgFunctionEvaluator;
pub use ceiling_function::CeilingFunctionEvaluator;
pub use exp_function::ExpFunctionEvaluator;
pub use floor_function::FloorFunctionEvaluator;
pub use ln_function::LnFunctionEvaluator;
pub use log_function::LogFunctionEvaluator;
pub use max_function::MaxFunctionEvaluator;
pub use min_function::MinFunctionEvaluator;
pub use power_function::PowerFunctionEvaluator;
pub use round_function::RoundFunctionEvaluator;
pub use sqrt_function::SqrtFunctionEvaluator;
pub use sum_function::SumFunctionEvaluator;
pub use truncate_function::TruncateFunctionEvaluator;

// Utility functions
pub use define_variable_function::DefineVariableFunctionEvaluator;

// Temporal functions
pub use day_of_function::DayOfFunctionEvaluator;
pub use difference_function::DifferenceFunctionEvaluator;
pub use duration_function::DurationFunctionEvaluator;
pub use hour_of_function::HourOfFunctionEvaluator;
pub use millisecond_function::MillisecondFunctionEvaluator;
pub use minute_of_function::MinuteOfFunctionEvaluator;
pub use month_of_function::MonthOfFunctionEvaluator;
pub use now_function::NowFunctionEvaluator;
pub use second_of_function::SecondOfFunctionEvaluator;
pub use timezone_offset_of_function::TimezoneOffsetOfFunctionEvaluator;
pub use today_function::TodayFunctionEvaluator;
pub use year_of_function::YearOfFunctionEvaluator;

// Validation functions
pub use has_value_function::HasValueFunctionEvaluator;

// Logic functions
pub use comparable_function::ComparableFunctionEvaluator;
pub use not_function::NotFunctionEvaluator;

// Navigation functions
pub use children_function::ChildrenFunctionEvaluator;
pub use descendants_function::DescendantsFunctionEvaluator;
pub use repeat_function::RepeatFunctionEvaluator;
pub use resolve_function::ResolveFunctionEvaluator;

// Enhanced functions
pub use high_boundary_function::HighBoundaryFunctionEvaluator;
pub use index_of_function::IndexOfFunctionEvaluator;
pub use is_distinct_function::IsDistinctFunctionEvaluator;
pub use last_index_of_function::LastIndexOfFunctionEvaluator;
pub use low_boundary_function::LowBoundaryFunctionEvaluator;
pub use matches_full_function::MatchesFullFunctionEvaluator;
pub use matches_function::MatchesFunctionEvaluator;
pub use precision_function::PrecisionFunctionEvaluator;

// Advanced functions
pub use aggregate_function::AggregateFunctionEvaluator;
pub use iif_function::IifFunctionEvaluator;
pub use length_function::LengthFunctionEvaluator;
pub use substring_function::SubstringFunctionEvaluator;

// Utility functions
pub use trace_function::TraceFunctionEvaluator;

// CDA functions
pub use has_template_id_of_function::HasTemplateIdOfFunctionEvaluator;

// Re-export from submodules
pub use conversion::{
    ConvertsToBooleanFunctionEvaluator, ConvertsToDateFunctionEvaluator,
    ConvertsToDateTimeFunctionEvaluator, ConvertsToDecimalFunctionEvaluator,
    ConvertsToIntegerFunctionEvaluator, ConvertsToQuantityFunctionEvaluator,
    ConvertsToStringFunctionEvaluator, ConvertsToTimeFunctionEvaluator, ToBooleanFunctionEvaluator,
    ToDateFunctionEvaluator, ToDateTimeFunctionEvaluator, ToDecimalFunctionEvaluator,
    ToIntegerFunctionEvaluator, ToQuantityFunctionEvaluator, ToStringFunctionEvaluator,
    ToTimeFunctionEvaluator,
};
pub use terminology::{
    ExpandFunctionEvaluator, LookupFunctionEvaluator, MemberOfFunctionEvaluator,
    SimpleExpandFunctionEvaluator, SubsumedByFunctionEvaluator, SubsumesFunctionEvaluator,
    TranslateFunctionEvaluator, ValidateCSFunctionEvaluator, ValidateVSFunctionEvaluator,
};
pub use type_checking::{
    AsFunctionEvaluator, ConformsToFunctionEvaluator, IsFunctionEvaluator, TypeFunctionEvaluator,
};
