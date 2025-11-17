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
pub mod hour_of_function;
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

// Re-export all functions for convenience
pub use abs_function::*;
pub use aggregate_function::*;
pub use all_function::*;
pub use all_true_function::*;
pub use any_true_function::*;
pub use avg_function::*;
pub use ceiling_function::*;
pub use children_function::*;
pub use coalesce_function::*;
pub use combine_function::*;
pub use comparable_function::*;
pub use contains_function::*;
pub use conversion::*;
pub use count_function::*;
pub use day_of_function::*;
pub use decode_function::*;
pub use define_variable_function::*;
pub use descendants_function::*;
pub use distinct_function::*;
pub use empty_function::*;
pub use encode_function::*;
pub use ends_with_function::*;
pub use escape_function::*;
pub use exclude_function::*;
pub use exists_function::*;
pub use exp_function::*;
pub use extension_function::*;
pub use first_function::*;
pub use floor_function::*;
pub use has_template_id_of_function::*;
pub use has_value_function::*;
pub use high_boundary_function::*;
pub use hour_of_function::*;
pub use iif_function::*;
pub use index_of_function::*;
pub use intersect_function::*;
pub use is_distinct_function::*;
pub use join_function::*;
pub use last_function::*;
pub use last_index_of_function::*;
pub use length_function::*;
pub use ln_function::*;
pub use log_function::*;
pub use low_boundary_function::*;
pub use lower_function::*;
pub use matches_full_function::*;
pub use matches_function::*;
pub use max_function::*;
pub use min_function::*;
pub use minute_of_function::*;
pub use month_of_function::*;
pub use not_function::*;
pub use now_function::*;
pub use of_type_function::*;
pub use power_function::*;
pub use precision_function::*;
pub use repeat_all_function::*;
pub use repeat_function::*;
pub use replace_function::*;
pub use replace_matches_function::*;
pub use resolve_function::*;
pub use round_function::*;
pub use second_of_function::*;
pub use select_function::*;
pub use single_function::*;
pub use skip_function::*;
pub use sort_function::*;
pub use split_function::*;
pub use sqrt_function::*;
pub use starts_with_function::*;
pub use subset_of_function::*;
pub use substring_function::*;
pub use sum_function::*;
pub use superset_of_function::*;
pub use tail_function::*;
pub use take_function::*;
pub use terminology::*;
pub use timezone_offset_of_function::*;
pub use to_chars_function::*;
pub use today_function::*;
pub use trace_function::*;
pub use trim_function::*;
pub use truncate_function::*;
pub use type_checking::*;
pub use unescape_function::*;
pub use union_function::*;
pub use upper_function::*;
pub use where_function::*;
pub use year_of_function::*;
