//! Utility functions for quantity operations and unit conversions
//!
//! This module provides helper functions for working with quantities in FHIRPath,
//! including unit conversions using the UCUM library.

use crate::core::{FhirPathValue, types::CalendarUnit};
use octofhir_ucum::{EvalResult, evaluate, parse_expression};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::borrow::Cow;
use std::sync::Arc;

/// Parse a string into a Quantity according to FHIRPath rules used by convertsToQuantity()/toQuantity().
///
/// Supported forms:
/// - `"<number>"` → unitless quantity
/// - `"<number> '<ucum>'"` → UCUM unit in single quotes
/// - `"<number> <calendar-word>"` → calendar unit (e.g., day, week, month, year, hour, minute, second, millisecond)
///
/// Notably, unquoted UCUM abbreviations like "wk" or "mo" are NOT accepted as calendar words.
pub fn parse_string_to_quantity_value(s: &str) -> Option<FhirPathValue> {
    let trimmed = s.trim();

    // 1) Plain number → unitless quantity
    if let Some(dec) = crate::core::value::utils::decimal_from_text(trimmed) {
        return Some(FhirPathValue::quantity_with_components(
            dec,
            Some("1".to_string()),
            None,
            None,
        ));
    }

    // 2) Split into two parts (number and unit)
    let mut parts = trimmed.split_whitespace();
    let num = parts.next()?;
    let unit_part = parts.next()?;
    // There should be exactly 2 parts
    if parts.next().is_some() {
        return None;
    }

    // Parse number first
    let dec = crate::core::value::utils::decimal_from_text(num)?;

    // Quoted UCUM: '<unit>'
    if unit_part.len() >= 2 && unit_part.starts_with('\'') && unit_part.ends_with('\'') {
        let inner = &unit_part[1..unit_part.len() - 1];
        return Some(FhirPathValue::quoted_quantity(dec, Some(inner.to_string())));
    }

    // Calendar words (full words only; no UCUM abbreviations like wk, mo, a)
    let unit_lc = unit_part.to_ascii_lowercase();
    // Accept singular/plural full words only
    let accepted = [
        "millisecond",
        "milliseconds",
        "second",
        "seconds",
        "minute",
        "minutes",
        "hour",
        "hours",
        "day",
        "days",
        "week",
        "weeks",
        "month",
        "months",
        "year",
        "years",
    ];

    if accepted.contains(&unit_lc.as_str())
        && let Some(cal) = CalendarUnit::from_str(&unit_lc)
    {
        return Some(FhirPathValue::calendar_quantity(dec, cal));
    }

    None
}

fn normalize_ucum_unit(unit: &str) -> Cow<'_, str> {
    match unit.trim() {
        "°C" => Cow::Borrowed("Cel"),
        "°F" => Cow::Borrowed("[degF]"),
        _ => Cow::Borrowed(unit),
    }
}

/// Result of a quantity conversion operation
#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub value: Decimal,
    pub unit: Option<String>,
    pub ucum_unit: Option<Arc<octofhir_ucum::UnitRecord>>,
    pub calendar_unit: Option<CalendarUnit>,
}

/// Error type for quantity operations
#[derive(Debug, Clone)]
pub enum QuantityError {
    /// Units are not compatible for conversion
    IncompatibleUnits(String, String),
    /// Failed to parse unit expression
    ParseError(String),
    /// Failed to evaluate unit expression
    EvaluationError(String),
    /// Invalid quantity data
    InvalidQuantity(String),
}

impl std::fmt::Display for QuantityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuantityError::IncompatibleUnits(from, to) => {
                write!(
                    f,
                    "Cannot convert from '{from}' to '{to}': incompatible units"
                )
            }
            QuantityError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            QuantityError::EvaluationError(msg) => write!(f, "Evaluation error: {msg}"),
            QuantityError::InvalidQuantity(msg) => write!(f, "Invalid quantity: {msg}"),
        }
    }
}

impl std::error::Error for QuantityError {}

/// Check if two unit strings are equivalent
/// Handles common cases where unit and code fields might differ but represent the same unit
fn units_are_equivalent(unit1: &str, unit2: &str) -> bool {
    // Direct match (already checked above, but for completeness)
    if unit1 == unit2 {
        return true;
    }

    // Common pound equivalences in FHIR/UCUM
    let pound_variants = ["lb", "lbs", "[lb_av]", "lb_av"];
    let is_pound1 = pound_variants.contains(&unit1);
    let is_pound2 = pound_variants.contains(&unit2);

    if is_pound1 && is_pound2 {
        return true;
    }

    // Add other common equivalences as needed
    // kg/kilogram, g/gram, m/meter, etc.
    let kg_variants = ["kg", "kilogram"];
    let is_kg1 = kg_variants.contains(&unit1);
    let is_kg2 = kg_variants.contains(&unit2);

    if is_kg1 && is_kg2 {
        return true;
    }

    false
}

/// Map UCUM time units to FHIRPath calendar units for equivalence checking.
/// Per FHIRPath spec: `1 year` is equivalent to `1 'a'`, `1 month` is equivalent to `1 'mo'`, etc.
/// Note: These are equivalent (~) but not equal (=) because UCUM uses fixed durations
/// (e.g., 'a' = 365.25 days) while FHIRPath calendar units are variable length.
fn ucum_to_calendar_unit(ucum_unit: &str) -> Option<CalendarUnit> {
    match ucum_unit {
        "a" => Some(CalendarUnit::Year),
        "mo" => Some(CalendarUnit::Month),
        "wk" => Some(CalendarUnit::Week),
        "d" => Some(CalendarUnit::Day),
        "h" => Some(CalendarUnit::Hour),
        "min" => Some(CalendarUnit::Minute),
        "s" => Some(CalendarUnit::Second),
        "ms" => Some(CalendarUnit::Millisecond),
        _ => None,
    }
}

/// Map FHIRPath calendar unit to its equivalent UCUM unit
fn calendar_to_ucum_unit(cal_unit: CalendarUnit) -> &'static str {
    match cal_unit {
        CalendarUnit::Year => "a",
        CalendarUnit::Month => "mo",
        CalendarUnit::Week => "wk",
        CalendarUnit::Day => "d",
        CalendarUnit::Hour => "h",
        CalendarUnit::Minute => "min",
        CalendarUnit::Second => "s",
        CalendarUnit::Millisecond => "ms",
    }
}

/// A UCUM unit as atoms with exponents: `kg.m/s2` is `kg^1 m^1 s^-2`. `None` for what
/// this simple form cannot hold (annotations, parentheses).
fn unit_terms(unit: &str) -> Option<Vec<(String, i32)>> {
    if unit.contains(['{', '(', ')']) {
        return None;
    }
    let mut terms: Vec<(String, i32)> = Vec::new();
    let mut sign = 1;
    let mut rest = unit;
    if let Some(stripped) = rest.strip_prefix('/') {
        sign = -1;
        rest = stripped;
    }
    loop {
        let end = rest.find(['.', '/']).unwrap_or(rest.len());
        let term = &rest[..end];
        let digits = term
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .trim_end_matches(['-', '+']);
        let atom = digits;
        let exponent: i32 = if atom.len() == term.len() {
            1
        } else {
            term[atom.len()..].parse().ok()?
        };
        if atom.is_empty() {
            // A bare number: only `1` (unity) is meaningful.
            if term != "1" {
                return None;
            }
        } else {
            match terms.iter_mut().find(|(a, _)| a == atom) {
                Some((_, e)) => *e += sign * exponent,
                None => terms.push((atom.to_string(), sign * exponent)),
            }
        }
        if end == rest.len() {
            break;
        }
        sign = if rest.as_bytes()[end] == b'/' { -1 } else { 1 };
        rest = &rest[end + 1..];
    }
    Some(terms)
}

fn render_unit_terms(terms: &[(String, i32)]) -> String {
    let term = |atom: &str, exponent: i32| {
        if exponent == 1 {
            atom.to_string()
        } else {
            format!("{atom}{exponent}")
        }
    };
    let numerator: Vec<String> = terms
        .iter()
        .filter(|(_, e)| *e > 0)
        .map(|(a, e)| term(a, *e))
        .collect();
    let denominator: Vec<String> = terms
        .iter()
        .filter(|(_, e)| *e < 0)
        .map(|(a, e)| term(a, -*e))
        .collect();
    match (numerator.is_empty(), denominator.is_empty()) {
        (true, true) => "1".to_string(),
        (false, true) => numerator.join("."),
        (true, false) => format!("/{}", denominator.join("/")),
        (false, false) => format!("{}/{}", numerator.join("."), denominator.join("/")),
    }
}

/// The unit of `left * right` (or `left / right`), with equal atoms combined:
/// `cm * cm` is `cm2`, `m / m` is `1`, `m/s * s` is `m`, `1 / s` is `/s`.
pub fn combine_units(left: &str, right: &str, divide: bool) -> String {
    let fallback = || {
        if divide {
            format!("{left}/{right}")
        } else {
            format!("{left}.{right}")
        }
    };
    let (Some(mut terms), Some(right_terms)) = (unit_terms(left), unit_terms(right)) else {
        return fallback();
    };
    for (atom, exponent) in right_terms {
        let exponent = if divide { -exponent } else { exponent };
        match terms.iter_mut().find(|(a, _)| *a == atom) {
            Some((_, e)) => *e += exponent,
            None => terms.push((atom, exponent)),
        }
    }
    terms.retain(|(_, e)| *e != 0);
    render_unit_terms(&terms)
}

/// A UCUM definite duration (`'wk'`, `'d'`, `'h'`, `'min'`, `'s'`, `'ms'`) as the matching
/// calendar duration, for date/time arithmetic, where they behave the same (FHIRPath
/// §1.6.7.1: `@1973-12-25 + 1 'd'` is `@1973-12-26`). `'a'` and `'mo'` are not calendar
/// years and months, so they and every other value are returned unchanged.
pub fn definite_duration_as_calendar(value: &FhirPathValue) -> std::borrow::Cow<'_, FhirPathValue> {
    if let FhirPathValue::Quantity {
        value: amount,
        unit,
        code,
        calendar_unit: None,
        ..
    } = value
        && let Some(ucum) = code.as_deref().or(unit.as_deref())
        && let Some(calendar) = ucum_to_calendar_unit(ucum)
        && !matches!(calendar, CalendarUnit::Year | CalendarUnit::Month)
    {
        let keyword = match calendar {
            CalendarUnit::Week => "week",
            CalendarUnit::Day => "day",
            CalendarUnit::Hour => "hour",
            CalendarUnit::Minute => "minute",
            CalendarUnit::Second => "second",
            _ => "millisecond",
        };
        return std::borrow::Cow::Owned(FhirPathValue::quantity_with_components(
            *amount,
            Some(keyword.to_string()),
            None,
            None,
        ));
    }
    std::borrow::Cow::Borrowed(value)
}

/// The time-valued quantity to add to (or subtract from) `target` (FHIRPath §1.6.7): UCUM
/// definite durations as calendar durations, and a unit finer than a partial value's
/// precision converted to that precision and truncated (`@2016 + 365 days` adds 1 year,
/// `@2026-02 + 4 weeks` adds 0 months).
pub fn duration_for_temporal<'a>(
    target: &FhirPathValue,
    quantity: &'a FhirPathValue,
) -> Cow<'a, FhirPathValue> {
    use crate::core::temporal::TemporalPrecision;
    let quantity = definite_duration_as_calendar(quantity);
    let precision = match target {
        FhirPathValue::Date(date, _, _) => date.precision,
        FhirPathValue::DateTime(dt, _, _) => dt.precision,
        FhirPathValue::Time(time, _, _) => time.precision,
        _ => return quantity,
    };
    let FhirPathValue::Quantity {
        value,
        calendar_unit: Some(unit),
        ..
    } = quantity.as_ref()
    else {
        return quantity;
    };
    let target_unit = match precision {
        TemporalPrecision::Year => CalendarUnit::Year,
        TemporalPrecision::Month => CalendarUnit::Month,
        TemporalPrecision::Day => CalendarUnit::Day,
        TemporalPrecision::Hour => CalendarUnit::Hour,
        TemporalPrecision::Minute => CalendarUnit::Minute,
        TemporalPrecision::Second => CalendarUnit::Second,
        TemporalPrecision::Millisecond => return quantity,
    };
    if calendar_milliseconds(*unit) >= calendar_milliseconds(target_unit) {
        return quantity;
    }
    match convert_calendar_value(*value, *unit, target_unit) {
        Some(converted) => Cow::Owned(FhirPathValue::calendar_quantity(
            converted.trunc(),
            target_unit,
        )),
        None => quantity,
    }
}

/// The unit a quantity's arithmetic works in: its UCUM code, a calendar unit's UCUM
/// equivalent, or `'1'` without a unit. `None` for calendar years and months, which only
/// combine with the same unit.
fn arithmetic_unit(quantity: &FhirPathValue) -> Option<(String, bool)> {
    let FhirPathValue::Quantity {
        unit,
        code,
        calendar_unit,
        ..
    } = quantity
    else {
        return None;
    };
    match calendar_unit {
        Some(CalendarUnit::Year | CalendarUnit::Month) => None,
        Some(calendar) => Some((calendar_to_ucum_unit(*calendar).to_string(), true)),
        None => Some((
            code.clone()
                .or_else(|| unit.clone())
                .unwrap_or_else(|| "1".to_string()),
            false,
        )),
    }
}

/// A number as a quantity with the UCUM default unit `'1'` (FHIRPath implicit conversion);
/// any other value unchanged.
pub fn number_as_quantity(value: &FhirPathValue) -> Cow<'_, FhirPathValue> {
    let amount = match value {
        FhirPathValue::Integer(i, _, _) => Decimal::from(*i),
        FhirPathValue::Decimal(d, _, _) => *d,
        _ => return Cow::Borrowed(value),
    };
    Cow::Owned(FhirPathValue::quantity_with_components(
        amount,
        Some("1".to_string()),
        None,
        None,
    ))
}

/// Whether a quantity has a UCUM "special" unit on a non-ratio scale (`Cel`, `[degF]`,
/// `[pH]`, ...): arithmetic on those is empty (FHIRPath §1.6.6).
pub fn has_special_unit(quantity: &FhirPathValue) -> bool {
    let FhirPathValue::Quantity {
        unit,
        code,
        calendar_unit: None,
        ..
    } = quantity
    else {
        return false;
    };
    code.as_deref()
        .or(unit.as_deref())
        .and_then(|unit| evaluate_unit(unit).ok())
        .is_some_and(|eval| eval.special.is_some())
}

/// `left + right` (or `left - right`) for two quantities (FHIRPath §1.6.6.3, FHIR-56190):
/// units must be commensurable, and the result is in the more granular of the two units
/// (`3 'm' + 3 'cm'` is `303 'cm'`). Calendar years and months only combine with the same
/// unit. Empty (`None`) when the units cannot be combined.
pub fn add_quantities(
    left: &FhirPathValue,
    right: &FhirPathValue,
    subtract: bool,
) -> Option<FhirPathValue> {
    let (FhirPathValue::Quantity { value: lv, .. }, FhirPathValue::Quantity { value: rv, .. }) =
        (left, right)
    else {
        return None;
    };
    if has_special_unit(left) || has_special_unit(right) {
        return None;
    }
    let combine = |a: Decimal, b: Decimal| {
        if subtract {
            a.checked_sub(b)
        } else {
            a.checked_add(b)
        }
    };
    let same_calendar = matches!(
        (left, right),
        (
            FhirPathValue::Quantity { calendar_unit: Some(l), .. },
            FhirPathValue::Quantity { calendar_unit: Some(r), .. },
        ) if l == r
    );
    let (base, value) = if same_calendar {
        (left, combine(*lv, *rv)?)
    } else {
        let (left_unit, _) = arithmetic_unit(left)?;
        let (right_unit, _) = arithmetic_unit(right)?;
        if left_unit == right_unit {
            (left, combine(*lv, *rv)?)
        } else {
            // How many right units one left unit is: above 1, the right unit is finer.
            let factor = convert_ucum_quantity(Decimal::ONE, &left_unit, &right_unit)
                .ok()?
                .value;
            if factor >= Decimal::ONE {
                (right, combine(lv.checked_mul(factor)?, *rv)?)
            } else {
                let back = convert_ucum_quantity(Decimal::ONE, &right_unit, &left_unit)
                    .ok()?
                    .value;
                (left, combine(*lv, rv.checked_mul(back)?)?)
            }
        }
    };
    let FhirPathValue::Quantity {
        unit,
        code,
        system,
        ucum_unit,
        calendar_unit,
        ..
    } = base
    else {
        return None;
    };
    // Operands from different time-valued unit systems give a calendar duration
    // (`60 's' + 2 minutes` is `180 seconds`).
    let is_calendar = |q: &FhirPathValue| {
        matches!(
            q,
            FhirPathValue::Quantity {
                calendar_unit: Some(_),
                ..
            }
        )
    };
    if is_calendar(left) != is_calendar(right)
        && calendar_unit.is_none()
        && let Some(calendar) = code
            .as_deref()
            .or(unit.as_deref())
            .and_then(ucum_to_calendar_unit)
    {
        return Some(FhirPathValue::calendar_quantity(value, calendar));
    }
    Some(FhirPathValue::Quantity {
        value,
        unit: unit.clone(),
        code: code.clone(),
        system: system.clone(),
        ucum_unit: ucum_unit.clone(),
        calendar_unit: *calendar_unit,
        type_info: crate::core::types::quantity_type_info(),
        primitive_element: None,
    })
}

/// What two quantities are compared for. Equality and ordering keep calendar years and
/// months apart from UCUM `'a'` and `'mo'` (`1 year = 1 'a'` is empty); equivalence treats
/// them as the same (`1 year ~ 1 'a'`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantityComparison {
    Equality,
    Equivalence,
    Ordering,
}

/// A calendar duration in milliseconds with the FHIRPath conversion factors (§1.5.5.7.2.1):
/// a month is 30 days and a year 365 days.
fn calendar_milliseconds(unit: CalendarUnit) -> Decimal {
    Decimal::from(match unit {
        CalendarUnit::Millisecond => 1_i64,
        CalendarUnit::Second => 1_000,
        CalendarUnit::Minute => 60_000,
        CalendarUnit::Hour => 3_600_000,
        CalendarUnit::Day => 86_400_000,
        CalendarUnit::Week => 604_800_000,
        CalendarUnit::Month => 30 * 86_400_000,
        CalendarUnit::Year => 365 * 86_400_000,
    })
}

/// `value` calendar `from` units as `to` units, along the shortest chain: a year is 12
/// months or 365 days, a month 30 days (FHIRPath §1.5.5.7.2.1).
pub fn convert_calendar_value(
    value: Decimal,
    from: CalendarUnit,
    to: CalendarUnit,
) -> Option<Decimal> {
    match (from, to) {
        _ if from == to => Some(value),
        (CalendarUnit::Year, CalendarUnit::Month) => value.checked_mul(Decimal::from(12)),
        (CalendarUnit::Month, CalendarUnit::Year) => value.checked_div(Decimal::from(12)),
        _ => value
            .checked_mul(calendar_milliseconds(from))?
            .checked_div(calendar_milliseconds(to))
            .map(|v| v.normalize()),
    }
}

/// A UCUM time quantity as a calendar duration: `'a'`, `'mo'`, `'wk'`, `'d'`, `'h'`,
/// `'min'`, `'s'`, `'ms'` by name, any other time unit through seconds.
fn ucum_as_calendar(value: Decimal, unit: &str) -> Option<(Decimal, CalendarUnit)> {
    if let Some(calendar) = ucum_to_calendar_unit(unit) {
        return Some((value, calendar));
    }
    let seconds = convert_ucum_quantity(value, unit, "s").ok()?.value;
    Some((seconds, CalendarUnit::Second))
}

/// A converted value with the binary floating-point noise of a UCUM factor removed.
fn tidy(value: Decimal) -> Decimal {
    value.round_sf(15).unwrap_or(value).normalize()
}

/// Two calendar durations in the less granular unit (FHIRPath §1.6.1.1.1: "convert to the
/// least granular unit"): `7 days` and `1 week` are `1` and `1`.
fn align_calendar(
    left: Decimal,
    left_unit: CalendarUnit,
    right: Decimal,
    right_unit: CalendarUnit,
) -> Option<(Decimal, Decimal)> {
    if left_unit == right_unit {
        return Some((left, right));
    }
    if calendar_milliseconds(left_unit) >= calendar_milliseconds(right_unit) {
        Some((left, convert_calendar_value(right, right_unit, left_unit)?))
    } else {
        Some((convert_calendar_value(left, left_unit, right_unit)?, right))
    }
}

/// Two UCUM quantities in one unit: the less granular for equivalence (its rounding depends
/// on the unit, §1.6.1.2.1), the more granular otherwise (any commensurable unit gives the
/// same answer, and scaling up keeps the decimals exact). Special units (`Cel`, `[degF]`)
/// convert the right value into the left unit. `None` when not commensurable.
fn align_ucum(
    left: Decimal,
    left_unit: &str,
    right: Decimal,
    right_unit: &str,
    purpose: QuantityComparison,
) -> Option<(Decimal, Decimal)> {
    if left_unit == right_unit || units_are_equivalent(left_unit, right_unit) {
        return Some((left, right));
    }
    let left_eval = evaluate_unit(left_unit).ok()?;
    let right_eval = evaluate_unit(right_unit).ok()?;
    if left_eval.special.is_some() || right_eval.special.is_some() {
        let converted = convert_ucum_quantity(right, right_unit, left_unit).ok()?;
        return Some((left, tidy(converted.value)));
    }
    if !linear_commensurable(&left_eval, &right_eval) {
        return None;
    }
    // One left unit in right units: at least 1 means the left unit is the coarser.
    let factor = convert_ucum_quantity(Decimal::ONE, left_unit, right_unit)
        .ok()?
        .value;
    let left_is_coarser = factor >= Decimal::ONE;
    if left_is_coarser == (purpose == QuantityComparison::Equivalence) {
        let converted = convert_ucum_quantity(right, right_unit, left_unit).ok()?;
        Some((left, tidy(converted.value)))
    } else {
        let converted = convert_ucum_quantity(left, left_unit, right_unit).ok()?;
        Some((tidy(converted.value), right))
    }
}

/// Both quantities' values in one unit, ready to compare (FHIRPath §1.6.1.1.1, §1.6.1.2.1,
/// §1.6.2). A unit is the UCUM code (`'1'` when absent) or a calendar duration; a UCUM time
/// unit meets a calendar duration as its calendar counterpart (`1 hour = 3600 's'`), except
/// that UCUM years and months only match calendar ones for equivalence. `None` when the
/// units are not commensurable.
#[allow(clippy::too_many_arguments)]
pub fn align_quantities(
    left_value: Decimal,
    left_unit: Option<&str>,
    left_calendar: Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: Option<&str>,
    right_calendar: Option<CalendarUnit>,
    purpose: QuantityComparison,
) -> Option<(Decimal, Decimal)> {
    let year_or_month =
        |unit: CalendarUnit| matches!(unit, CalendarUnit::Year | CalendarUnit::Month);
    match (left_calendar, right_calendar) {
        (Some(l), Some(r)) => align_calendar(left_value, l, right_value, r),
        (None, None) => align_ucum(
            left_value,
            left_unit.unwrap_or("1"),
            right_value,
            right_unit.unwrap_or("1"),
            purpose,
        ),
        (Some(l), None) => {
            let (value, r) = ucum_as_calendar(right_value, right_unit?)?;
            if purpose != QuantityComparison::Equivalence && (year_or_month(l) || year_or_month(r))
            {
                return None;
            }
            align_calendar(left_value, l, value, r)
        }
        (None, Some(r)) => {
            let (value, l) = ucum_as_calendar(left_value, left_unit?)?;
            if purpose != QuantityComparison::Equivalence && (year_or_month(l) || year_or_month(r))
            {
                return None;
            }
            align_calendar(value, l, right_value, r)
        }
    }
}

/// Compare two quantities and return an Ordering (for <, >, <= comparisons); `None` when
/// they are not comparable.
pub fn compare_quantities(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    right_calendar_unit: &Option<CalendarUnit>,
) -> Result<Option<std::cmp::Ordering>, QuantityError> {
    Ok(align_quantities(
        left_value,
        left_unit.as_deref(),
        *left_calendar_unit,
        right_value,
        right_unit.as_deref(),
        *right_calendar_unit,
        QuantityComparison::Ordering,
    )
    .map(|(l, r)| l.cmp(&r)))
}

/// Check if two quantities are equivalent (FHIRPath §1.6.1.2.1): converted to the less
/// granular unit, rounded to the precision of the less precise value, then equal.
pub fn are_quantities_equivalent(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    right_calendar_unit: &Option<CalendarUnit>,
) -> Result<bool, QuantityError> {
    are_quantities_equivalent_with_codes(
        left_value,
        left_unit,
        None,
        left_calendar_unit,
        right_value,
        right_unit,
        None,
        right_calendar_unit,
    )
}

/// [`are_quantities_equivalent`] with the FHIR `code` (the UCUM code) preferred over the
/// display `unit`.
#[allow(clippy::too_many_arguments)]
pub fn are_quantities_equivalent_with_codes(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_code: Option<&str>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    right_code: Option<&str>,
    right_calendar_unit: &Option<CalendarUnit>,
) -> Result<bool, QuantityError> {
    let Some((left, right)) = align_quantities(
        left_value,
        left_code.or(left_unit.as_deref()),
        *left_calendar_unit,
        right_value,
        right_code.or(right_unit.as_deref()),
        *right_calendar_unit,
        QuantityComparison::Equivalence,
    ) else {
        return Ok(false);
    };
    let precision = left_value.scale().min(right_value.scale());
    let round = |value: Decimal| {
        value.round_dp_with_strategy(
            precision,
            rust_decimal::RoundingStrategy::MidpointAwayFromZero,
        )
    };
    Ok(round(left) == round(right))
}

/// Check if two quantities are equal (FHIRPath §1.6.1.1.1); `None` (empty) when the units
/// are not commensurable.
pub fn are_quantities_equal(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    right_calendar_unit: &Option<CalendarUnit>,
) -> Option<bool> {
    are_quantities_equal_with_codes(
        left_value,
        left_unit,
        None,
        left_calendar_unit,
        right_value,
        right_unit,
        None,
        right_calendar_unit,
    )
}

/// [`are_quantities_equal`] with the FHIR `code` (the UCUM code) preferred over the display
/// `unit`.
#[allow(clippy::too_many_arguments)]
pub fn are_quantities_equal_with_codes(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_code: Option<&str>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    right_code: Option<&str>,
    right_calendar_unit: &Option<CalendarUnit>,
) -> Option<bool> {
    align_quantities(
        left_value,
        left_code.or(left_unit.as_deref()),
        *left_calendar_unit,
        right_value,
        right_code.or(right_unit.as_deref()),
        *right_calendar_unit,
        QuantityComparison::Equality,
    )
    .map(|(left, right)| left == right)
}

/// `quantity` in `target`: a UCUM code or a calendar duration keyword (FHIRPath
/// §1.5.5.7.1). Between a calendar duration and a UCUM time unit the conversion happens in
/// the source's system and the unit is then renamed (`7 days.toQuantity('wk')` is
/// `1 'wk'`). `None` when the units are not commensurable.
pub fn quantity_to_unit(quantity: &FhirPathValue, target: &str) -> Option<FhirPathValue> {
    let FhirPathValue::Quantity {
        value,
        unit,
        code,
        calendar_unit,
        ..
    } = quantity
    else {
        return None;
    };
    let source_unit = code.as_deref().or(unit.as_deref()).unwrap_or("1");
    match (calendar_unit, CalendarUnit::from_keyword(target)) {
        (Some(from), Some(to)) => Some(FhirPathValue::calendar_quantity(
            convert_calendar_value(*value, *from, to)?,
            to,
        )),
        (Some(from), None) => {
            let to = ucum_to_calendar_unit(target)?;
            let converted = convert_calendar_value(*value, *from, to)?;
            Some(FhirPathValue::quoted_quantity(
                converted,
                Some(target.to_string()),
            ))
        }
        (None, Some(to)) => {
            let ucum = calendar_to_ucum_unit(to);
            let converted = if source_unit == ucum {
                *value
            } else {
                tidy(convert_ucum_quantity(*value, source_unit, ucum).ok()?.value)
            };
            Some(FhirPathValue::calendar_quantity(converted, to))
        }
        (None, None) => {
            let converted = if source_unit == target {
                *value
            } else {
                // Validate the target as UCUM before converting.
                evaluate_unit(target).ok()?;
                tidy(
                    convert_ucum_quantity(*value, source_unit, target)
                        .ok()?
                        .value,
                )
            };
            Some(FhirPathValue::quoted_quantity(
                converted,
                Some(target.to_string()),
            ))
        }
    }
}

/// Convert a quantity to a target unit
pub fn convert_quantity(
    value: Decimal,
    from_unit: &Option<String>,
    from_calendar_unit: &Option<CalendarUnit>,
    to_unit: &str,
) -> Result<ConversionResult, QuantityError> {
    // Handle calendar unit conversions
    if let Some(from_cal) = from_calendar_unit
        && let Some(to_cal) = CalendarUnit::from_str(to_unit)
    {
        let converted_value =
            convert_calendar_value(value, *from_cal, to_cal).ok_or_else(|| {
                QuantityError::EvaluationError("Calendar conversion overflow".to_string())
            })?;
        return Ok(ConversionResult {
            value: converted_value,
            unit: Some(to_unit.to_string()),
            ucum_unit: None,
            calendar_unit: Some(to_cal),
        });
    }

    // Handle UCUM conversions
    if let Some(from_unit_str) = from_unit {
        return convert_ucum_quantity(value, from_unit_str, to_unit);
    }

    Err(QuantityError::InvalidQuantity(
        "Cannot convert unitless quantity to a specific unit".to_string(),
    ))
}

/// Parse and evaluate a UCUM unit (after FHIRPath display-unit normalization).
fn evaluate_unit(unit: &str) -> Result<EvalResult, QuantityError> {
    let normalized = normalize_ucum_unit(unit);
    let expr = parse_expression(normalized.as_ref())
        .map_err(|e| QuantityError::ParseError(format!("Failed to parse '{unit}': {e}")))?;
    evaluate(&expr)
        .map_err(|e| QuantityError::EvaluationError(format!("Failed to evaluate '{unit}': {e}")))
}

/// Linear units (no special function, UCUM §21-23) are commensurable when they
/// have the same base-unit dimension and the same arbitrary atoms (§24-25:
/// `[IU]` is commensurable with no other unit).
fn linear_commensurable(left: &EvalResult, right: &EvalResult) -> bool {
    left.dim == right.dim && left.arbitrary == right.arbitrary
}

fn decimal_to_f64(value: Decimal) -> Result<f64, QuantityError> {
    value.to_f64().ok_or_else(|| {
        QuantityError::EvaluationError(format!("Quantity value {value} is out of range"))
    })
}

/// Check if two UCUM units are comparable (commensurable)
pub fn are_ucum_units_comparable(left_unit: &str, right_unit: &str) -> Result<bool, QuantityError> {
    // If units are identical, they're comparable
    if left_unit == right_unit {
        return Ok(true);
    }
    let left_normalized = normalize_ucum_unit(left_unit);
    let right_normalized = normalize_ucum_unit(right_unit);
    // Parse first so a malformed unit is a parse error, not "not comparable".
    evaluate_unit(left_unit)?;
    evaluate_unit(right_unit)?;
    octofhir_ucum::is_comparable(left_normalized.as_ref(), right_normalized.as_ref())
        .map_err(|e| QuantityError::EvaluationError(e.to_string()))
}

/// Convert a UCUM quantity to a target unit
fn convert_ucum_quantity(
    value: Decimal,
    from_unit: &str,
    to_unit: &str,
) -> Result<ConversionResult, QuantityError> {
    let from_eval = evaluate_unit(from_unit)?;
    let to_eval = evaluate_unit(to_unit)?;
    let to_unit_normalized = normalize_ucum_unit(to_unit);
    let to_unit_value = to_unit_normalized.as_ref();

    let incompatible =
        || QuantityError::IncompatibleUnits(from_unit.to_string(), to_unit.to_string());

    let converted_value = if from_eval.special.is_none() && to_eval.special.is_none() {
        if !linear_commensurable(&from_eval, &to_eval) {
            return Err(incompatible());
        }
        // Linear: scale by the ratio of the magnitudes, kept in Decimal so an
        // exact conversion (1 'km' -> 1000 'm') stays exact.
        let ratio = from_eval
            .factor
            .checked_div(to_eval.factor)
            .ok_or_else(incompatible)?
            .to_f64();
        value * Decimal::try_from(ratio).unwrap_or_default()
    } else {
        // Special units convert through their function, not a factor.
        let from_normalized = normalize_ucum_unit(from_unit);
        let converted = octofhir_ucum::convert(
            decimal_to_f64(value)?,
            from_normalized.as_ref(),
            to_unit_value,
        )
        .map_err(|_| incompatible())?;
        Decimal::try_from(converted).map_err(|_| {
            QuantityError::EvaluationError(format!("Converted value {converted} is out of range"))
        })?
    };

    // Try to find the target unit record
    let ucum_unit =
        crate::core::types::ucum_unit_record(to_unit_value).map(|u| Arc::new(u.clone()));

    Ok(ConversionResult {
        value: converted_value,
        unit: Some(to_unit_value.to_string()),
        ucum_unit,
        calendar_unit: CalendarUnit::from_str(to_unit_value),
    })
}

/// Multiply two quantities, handling unit combination
pub fn multiply_quantities(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    right_calendar_unit: &Option<CalendarUnit>,
) -> Result<FhirPathValue, QuantityError> {
    let result_value = left_value * right_value;

    // Handle unit combination
    let (result_unit, _result_ucum_unit, _result_calendar_unit) = match (left_unit, right_unit) {
        (None, None) => (None, None::<Arc<octofhir_ucum::UnitRecord>>, None),
        (Some(l), None) => (Some(l.clone()), None, *left_calendar_unit),
        (None, Some(r)) => (Some(r.clone()), None, *right_calendar_unit),
        (Some(l), Some(r)) => {
            // For now, simple concatenation - TODO: implement proper UCUM unit multiplication
            let combined_unit = format!("{l}.{r}");
            (Some(combined_unit), None, None)
        }
    };

    Ok(FhirPathValue::quantity_with_components(
        result_value,
        result_unit.clone(),
        result_unit,
        None,
    ))
}

/// Divide two quantities, handling unit combination
pub fn divide_quantities(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    _right_calendar_unit: &Option<CalendarUnit>,
) -> Result<FhirPathValue, QuantityError> {
    if right_value == Decimal::ZERO {
        return Err(QuantityError::InvalidQuantity(
            "Division by zero".to_string(),
        ));
    }

    let result_value = left_value / right_value;

    // Handle unit combination
    let (result_unit, _result_ucum_unit, _result_calendar_unit) = match (left_unit, right_unit) {
        (None, None) => (None, None::<Arc<octofhir_ucum::UnitRecord>>, None),
        (Some(l), None) => (Some(l.clone()), None, *left_calendar_unit),
        (None, Some(r)) => (Some(format!("1/{r}")), None, None),
        (Some(l), Some(r)) => {
            // Handle special cases
            if l == r {
                // Same units cancel out to dimensionless
                (Some("1".to_string()), None, None)
            } else {
                // For now, simple concatenation - TODO: implement proper UCUM unit division
                let combined_unit = format!("{l}/{r}");
                (Some(combined_unit), None, None)
            }
        }
    };

    Ok(FhirPathValue::quantity_with_components(
        result_value,
        result_unit.clone(),
        result_unit,
        None,
    ))
}

#[cfg(test)]
mod tests {

    #[test]
    fn ucum_linear_conversion_scales_by_the_magnitude_ratio() {
        let mg_to_g = convert_ucum_quantity(Decimal::new(1000, 0), "mg", "g");
        assert_eq!(mg_to_g.expect("mg -> g").value, Decimal::new(1, 0));
        let km_to_m = convert_ucum_quantity(Decimal::new(15, 1), "km", "m");
        assert_eq!(km_to_m.expect("km -> m").value, Decimal::new(1500, 0));
    }

    #[test]
    fn ucum_special_units_convert_through_their_function() {
        // UCUM §21-23: a temperature is not a factor of kelvin.
        let c_to_f = convert_ucum_quantity(Decimal::new(100, 0), "Cel", "[degF]");
        assert_eq!(c_to_f.expect("Cel -> [degF]").value, Decimal::new(212, 0));
    }

    #[test]
    fn ucum_arbitrary_units_are_not_commensurable() {
        // UCUM §24-25: [IU] converts to no other unit.
        assert!(convert_ucum_quantity(Decimal::new(1, 0), "[IU]", "1").is_err());
        assert!(!are_ucum_units_comparable("[IU]", "[arb'U]").unwrap());
    }

    use super::*;

    #[test]
    fn test_ucum_to_calendar_mapping() {
        // Test that UCUM time units map to calendar units
        assert_eq!(ucum_to_calendar_unit("a"), Some(CalendarUnit::Year));
        assert_eq!(ucum_to_calendar_unit("mo"), Some(CalendarUnit::Month));
        assert_eq!(ucum_to_calendar_unit("wk"), Some(CalendarUnit::Week));
        assert_eq!(ucum_to_calendar_unit("d"), Some(CalendarUnit::Day));
        assert_eq!(ucum_to_calendar_unit("h"), Some(CalendarUnit::Hour));
        assert_eq!(ucum_to_calendar_unit("min"), Some(CalendarUnit::Minute));
        assert_eq!(ucum_to_calendar_unit("s"), Some(CalendarUnit::Second));
        assert_eq!(ucum_to_calendar_unit("ms"), Some(CalendarUnit::Millisecond));
        // Non-time units should return None
        assert_eq!(ucum_to_calendar_unit("kg"), None);
        assert_eq!(ucum_to_calendar_unit("m"), None);
    }

    #[test]
    fn units_combine_by_atom() {
        assert_eq!(combine_units("cm", "cm", false), "cm2");
        assert_eq!(combine_units("cm", "cm2", false), "cm3");
        assert_eq!(combine_units("m/s", "s", false), "m");
        assert_eq!(combine_units("m", "m", true), "1");
        assert_eq!(combine_units("cm2", "cm", true), "cm");
        assert_eq!(combine_units("m", "s", true), "m/s");
        assert_eq!(combine_units("1", "s", true), "/s");
        assert_eq!(combine_units("kg.m", "s2", true), "kg.m/s2");
        assert_eq!(combine_units("mg", "1", false), "mg");
        assert_eq!(combine_units("{a}", "m", false), "{a}.m");
    }

    #[test]
    fn test_ucum_a_vs_calendar_year_equivalence() {
        // Per FHIRPath spec: 1 'a' ~ 1 year should be true
        let result = are_quantities_equivalent(
            Decimal::new(1, 0),
            &Some("a".to_string()),
            &None, // UCUM, not calendar
            Decimal::new(1, 0),
            &Some("year".to_string()),
            &Some(CalendarUnit::Year),
        );
        assert!(result.unwrap(), "1 'a' should be equivalent to 1 year");
    }

    #[test]
    fn test_calendar_year_vs_ucum_a_equivalence() {
        // Same test in reverse order
        let result = are_quantities_equivalent(
            Decimal::new(1, 0),
            &Some("year".to_string()),
            &Some(CalendarUnit::Year),
            Decimal::new(1, 0),
            &Some("a".to_string()),
            &None, // UCUM, not calendar
        );
        assert!(result.unwrap(), "1 year should be equivalent to 1 'a'");
    }

    #[test]
    fn test_same_unit_equivalence() {
        let result = are_quantities_equivalent(
            Decimal::new(1000, 0),
            &Some("g".to_string()),
            &None,
            Decimal::new(1000, 0),
            &Some("g".to_string()),
            &None,
        );
        assert!(result.unwrap());
    }

    #[test]
    fn test_ucum_conversion() {
        let result = are_quantities_equivalent(
            Decimal::new(1000, 0),
            &Some("mg".to_string()),
            &None,
            Decimal::new(1, 0),
            &Some("g".to_string()),
            &None,
        );
        assert!(result.unwrap());
    }

    #[test]
    fn test_calendar_unit_conversion() {
        let result = are_quantities_equivalent(
            Decimal::new(7, 0),
            &Some("days".to_string()),
            &Some(CalendarUnit::Day),
            Decimal::new(1, 0),
            &Some("week".to_string()),
            &Some(CalendarUnit::Week),
        );
        assert!(result.unwrap());
    }
}
