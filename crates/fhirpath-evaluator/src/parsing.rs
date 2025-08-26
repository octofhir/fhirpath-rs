//! FHIRPath literal parsing utilities
//!
//! This module contains functions for parsing FHIRPath literal values including
//! dates, times, datetimes, and other temporal values with proper precision handling.

use chrono::TimeZone;
use octofhir_fhirpath_model::{PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision};

/// Parse FHIRPath date literal supporting partial dates
/// Supports: @YYYY, @YYYY-MM, @YYYY-MM-DD
pub fn parse_fhirpath_date(date_str: &str) -> Result<PrecisionDate, String> {
    // Remove the @ prefix if present
    let date_str = date_str.strip_prefix('@').unwrap_or(date_str);

    // Count the number of parts
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.is_empty() || parts[0].is_empty() {
        return Err(format!("Invalid date format: {date_str}"));
    }

    let year = parts[0]
        .parse::<i32>()
        .map_err(|_| format!("Invalid year in date: {}", parts[0]))?;

    match parts.len() {
        1 => {
            // Year only: @YYYY
            let date = chrono::NaiveDate::from_ymd_opt(year, 1, 1)
                .ok_or_else(|| format!("Invalid date: {year}-01-01"))?;
            Ok(PrecisionDate::new(date, TemporalPrecision::Year))
        }
        2 => {
            // Year-Month: @YYYY-MM
            let month = parts[1]
                .parse::<u32>()
                .map_err(|_| format!("Invalid month in date: {}", parts[1]))?;
            let date = chrono::NaiveDate::from_ymd_opt(year, month, 1)
                .ok_or_else(|| format!("Invalid date: {year}-{month:02}-01"))?;
            Ok(PrecisionDate::new(date, TemporalPrecision::Month))
        }
        3 => {
            // Year-Month-Day: @YYYY-MM-DD
            let month = parts[1]
                .parse::<u32>()
                .map_err(|_| format!("Invalid month in date: {}", parts[1]))?;
            let day = parts[2]
                .parse::<u32>()
                .map_err(|_| format!("Invalid day in date: {}", parts[2]))?;
            let date = chrono::NaiveDate::from_ymd_opt(year, month, day)
                .ok_or_else(|| format!("Invalid date: {year}-{month:02}-{day:02}"))?;
            Ok(PrecisionDate::new(date, TemporalPrecision::Day))
        }
        _ => Err(format!("Invalid date format: {date_str}")),
    }
}

/// Parse FHIRPath datetime literal supporting partial datetimes
/// Supports: @2015T, @2015-02T, @2015-02-04T14:34:28Z, etc.
pub fn parse_fhirpath_datetime(datetime_str: &str) -> Result<PrecisionDateTime, String> {
    // Remove the @ prefix if present
    let datetime_str = datetime_str.strip_prefix('@').unwrap_or(datetime_str);

    // Split date and time parts on 'T'
    let parts: Vec<&str> = datetime_str.splitn(2, 'T').collect();
    if parts.is_empty() {
        return Err(format!("Invalid datetime format: {datetime_str}"));
    }

    let date_str = parts[0];
    let time_str = if parts.len() > 1 {
        Some(parts[1])
    } else {
        None
    };

    // Parse date part
    let date_result = parse_fhirpath_date(date_str)?;

    match time_str {
        Some(time_part) if !time_part.is_empty() => {
            // Has time component
            let (time_part, tz_part) = split_time_timezone(time_part);

            if time_part.is_empty() {
                // Only T with no time (e.g., "@2015T")
                let naive_datetime = chrono::NaiveDateTime::new(
                    date_result.date,
                    chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                );
                let offset = parse_timezone_offset(tz_part)?;
                let datetime = offset
                    .from_local_datetime(&naive_datetime)
                    .single()
                    .ok_or_else(|| "Could not create datetime with timezone".to_string())?;
                let precision = match date_result.precision {
                    TemporalPrecision::Year => TemporalPrecision::Year,
                    TemporalPrecision::Month => TemporalPrecision::Month,
                    TemporalPrecision::Day => TemporalPrecision::Day,
                    _ => TemporalPrecision::Day,
                };
                Ok(PrecisionDateTime::new(datetime, precision))
            } else {
                // Parse the time component
                let time = parse_time_components(time_part)?;
                let naive_datetime = chrono::NaiveDateTime::new(date_result.date, time);
                let offset = parse_timezone_offset(tz_part)?;
                let datetime = offset
                    .from_local_datetime(&naive_datetime)
                    .single()
                    .ok_or_else(|| "Could not create datetime with timezone".to_string())?;
                let time_precision = determine_time_precision(time_part);

                // Combine date and time precision - use the more precise one
                let combined_precision = match (date_result.precision, time_precision) {
                    (_, TemporalPrecision::Second) => TemporalPrecision::Second,
                    (_, TemporalPrecision::Millisecond) => TemporalPrecision::Millisecond,
                    (TemporalPrecision::Day, _) => TemporalPrecision::Minute,
                    _ => date_result.precision,
                };

                Ok(PrecisionDateTime::new(datetime, combined_precision))
            }
        }
        _ => {
            // Date only with T but no time (e.g., "@2015T")
            let naive_datetime = chrono::NaiveDateTime::new(
                date_result.date,
                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            );
            let offset = chrono::FixedOffset::east_opt(0).unwrap(); // Default to UTC
            let datetime = offset
                .from_local_datetime(&naive_datetime)
                .single()
                .ok_or_else(|| "Could not create datetime with timezone".to_string())?;
            let precision = match date_result.precision {
                TemporalPrecision::Year => TemporalPrecision::Year,
                TemporalPrecision::Month => TemporalPrecision::Month,
                TemporalPrecision::Day => TemporalPrecision::Day,
                _ => TemporalPrecision::Day,
            };
            Ok(PrecisionDateTime::new(datetime, precision))
        }
    }
}

/// Parse FHIRPath time literal supporting partial times and timezones
/// Supports: @T14, @T14:34, @T14:34:28, @T14:34:28.123, @T14:34:28Z, @T14:34:28+10:00
pub fn parse_fhirpath_time(time_str: &str) -> Result<PrecisionTime, String> {
    // Remove the @T prefix if present
    let time_str = time_str
        .strip_prefix('@')
        .and_then(|s| s.strip_prefix('T'))
        .unwrap_or(time_str);
    // Remove timezone info for parsing time only
    let (time_part, _) = split_time_timezone(time_str);
    let time = parse_time_components(time_part)?;
    let precision = determine_time_precision(time_part);

    Ok(PrecisionTime::new(time, precision))
}

/// Split time string into time and timezone parts
pub fn split_time_timezone(time_str: &str) -> (&str, Option<&str>) {
    if let Some(pos) = time_str.find('Z') {
        (&time_str[..pos], Some("Z"))
    } else if let Some(pos) = time_str.find('+') {
        let parts = time_str.split_at(pos);
        (parts.0, Some(parts.1)) // Keep the '+' sign
    } else if let Some(pos) = time_str.rfind('-') {
        // Check if this '-' is part of timezone (not time separator)
        if pos > 0
            && time_str
                .chars()
                .nth(pos - 1)
                .is_some_and(|c| c.is_ascii_digit())
        {
            let parts = time_str.split_at(pos);
            (parts.0, Some(parts.1)) // Keep the '-' sign
        } else {
            (time_str, None)
        }
    } else {
        (time_str, None)
    }
}

/// Parse time components (hour, minute, second, millisecond)
pub fn parse_time_components(time_str: &str) -> Result<chrono::NaiveTime, String> {
    let parts: Vec<&str> = time_str.split(':').collect();

    let hour = parts[0]
        .parse::<u32>()
        .map_err(|_| format!("Invalid hour in time: {}", parts[0]))?;

    let minute = if parts.len() > 1 {
        parts[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid minute in time: {}", parts[1]))?
    } else {
        0
    };

    let (second, nano) = if parts.len() > 2 {
        let second_part = parts[2];
        if let Some(dot_pos) = second_part.find('.') {
            let (sec_str, frac_str) = second_part.split_at(dot_pos);
            let second = sec_str
                .parse::<u32>()
                .map_err(|_| format!("Invalid second in time: {sec_str}"))?;

            let frac_str = &frac_str[1..]; // Remove the dot
            // Pad or truncate to 9 digits (nanoseconds)
            let padded_frac = if frac_str.len() < 9 {
                format!("{frac_str:0<9}")
            } else {
                frac_str[..9].to_string()
            };
            let nano = padded_frac
                .parse::<u32>()
                .map_err(|_| format!("Invalid fractional seconds in time: {frac_str}"))?;
            (second, nano)
        } else {
            let second = second_part
                .parse::<u32>()
                .map_err(|_| format!("Invalid second in time: {second_part}"))?;
            (second, 0)
        }
    } else {
        (0, 0)
    };

    chrono::NaiveTime::from_hms_nano_opt(hour, minute, second, nano)
        .ok_or_else(|| format!("Invalid time: {hour:02}:{minute:02}:{second:02}.{nano:09}"))
}

/// Parse timezone offset
pub fn parse_timezone_offset(tz_str: Option<&str>) -> Result<chrono::FixedOffset, String> {
    match tz_str {
        None => {
            // No timezone specified, assume local/unspecified
            // For FHIRPath, this is valid - datetimes can be timezone-agnostic
            Ok(chrono::FixedOffset::east_opt(0).unwrap()) // Use UTC as default
        }
        Some("Z") => {
            // UTC
            Ok(chrono::FixedOffset::east_opt(0).unwrap())
        }
        Some(tz_str) => {
            // Parse +HH:MM or -HH:MM format
            let (sign, tz_str) = if let Some(stripped) = tz_str.strip_prefix('+') {
                (1, stripped)
            } else if let Some(stripped) = tz_str.strip_prefix('-') {
                (-1, stripped)
            } else {
                return Err(format!("Invalid timezone format: {tz_str}"));
            };

            let parts: Vec<&str> = tz_str.split(':').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid timezone format: {tz_str}"));
            }

            let hours = parts[0]
                .parse::<i32>()
                .map_err(|_| format!("Invalid timezone hours: {}", parts[0]))?;
            let minutes = parts[1]
                .parse::<i32>()
                .map_err(|_| format!("Invalid timezone minutes: {}", parts[1]))?;

            let offset_seconds = sign * (hours * 3600 + minutes * 60);
            chrono::FixedOffset::east_opt(offset_seconds)
                .ok_or_else(|| format!("Invalid timezone offset: {offset_seconds}"))
        }
    }
}

/// Determine time precision from time string format
pub fn determine_time_precision(time_str: &str) -> TemporalPrecision {
    let parts: Vec<&str> = time_str.split(':').collect();
    match parts.len() {
        1 => TemporalPrecision::Hour,   // @T14
        2 => TemporalPrecision::Minute, // @T14:30
        3 => {
            // Check for fractional seconds
            if parts[2].contains('.') {
                TemporalPrecision::Millisecond // @T14:30:15.123
            } else {
                TemporalPrecision::Second // @T14:30:15
            }
        }
        _ => TemporalPrecision::Hour, // Default fallback
    }
}
