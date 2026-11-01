//! Format-code parsing for `toDate([format])` / `toDateTime([format])`
//! (and the matching `convertsTo*` functions), per FHIRPath 3.1 CI
//! "Date/DateTime String Format Codes".
//!
//! Supported codes (case-sensitive): `yyyy`, `yy` (00-49 -> 20xx, 50-99 -> 19xx),
//! `MMMM`/`MMM` (English month names), `MM`, `M`, `dd`, `d`, `hh`, `h`, `HH`, `H`,
//! `mm`, `m`, `ss`, `s`, `S+` (fraction of a second, one digit per `S`),
//! `a` (AM/PM, English) and `Z` (`Z` or `+hhmm`/`-hhmm`). Every other character is a
//! literal that must match exactly. Unsupported: `z` (treated as a literal).

use crate::core::temporal::{PrecisionDate, PrecisionDateTime, TemporalPrecision};
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Literal(char),
    Year4,
    Year2,
    /// Numeric fields carry their minimum digit count (1 or 2; max is always 2)
    Month(usize),
    MonthName {
        full: bool,
    },
    Day(usize),
    Hour12(usize),
    Hour24(usize),
    Minute(usize),
    Second(usize),
    Fraction(usize),
    AmPm,
    Offset,
}

fn tokenize(format: &str) -> Option<Vec<Token>> {
    let chars: Vec<char> = format.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let mut run = 1;
        while i + run < chars.len() && chars[i + run] == c {
            run += 1;
        }
        let token = match (c, run) {
            ('y', 4) => Token::Year4,
            ('y', 2) => Token::Year2,
            ('M', 1 | 2) => Token::Month(run),
            ('M', 3) => Token::MonthName { full: false },
            ('M', 4) => Token::MonthName { full: true },
            ('d', 1 | 2) => Token::Day(run),
            ('h', 1 | 2) => Token::Hour12(run),
            ('H', 1 | 2) => Token::Hour24(run),
            ('m', 1 | 2) => Token::Minute(run),
            ('s', 1 | 2) => Token::Second(run),
            ('S', n) => Token::Fraction(n),
            ('a', 1 | 2) => Token::AmPm,
            ('Z', 1) => Token::Offset,
            ('y' | 'M' | 'd' | 'h' | 'H' | 'm' | 's' | 'a' | 'Z', _) => return None,
            _ => {
                tokens.extend(std::iter::repeat_n(Token::Literal(c), run));
                i += run;
                continue;
            }
        };
        tokens.push(token);
        i += run;
    }
    Some(tokens)
}

#[derive(Debug, Default)]
struct Parsed {
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    hour: Option<u32>,
    hour12: bool,
    pm: Option<bool>,
    minute: Option<u32>,
    second: Option<u32>,
    /// Nanoseconds of the fractional second
    fraction: Option<u32>,
    /// Offset in seconds east of UTC
    offset: Option<i32>,
}

fn take_digits(s: &[char], pos: &mut usize, min: usize, max: usize) -> Option<u32> {
    let mut n = 0;
    let mut value: u32 = 0;
    while n < max && *pos + n < s.len() {
        let Some(d) = s[*pos + n].to_digit(10) else {
            break;
        };
        value = value.checked_mul(10)?.checked_add(d)?;
        n += 1;
    }
    if n < min {
        return None;
    }
    *pos += n;
    Some(value)
}

/// Store a field, failing if it was already set (duplicate code in the format).
fn set<T>(slot: &mut Option<T>, value: T) -> Option<()> {
    if slot.is_some() {
        return None;
    }
    *slot = Some(value);
    Some(())
}

fn parse_fields(input: &str, format: &str) -> Option<Parsed> {
    let tokens = tokenize(format)?;
    let s: Vec<char> = input.chars().collect();
    let mut pos = 0;
    let mut p = Parsed::default();

    for token in &tokens {
        match token {
            Token::Literal(c) => {
                if s.get(pos) != Some(c) {
                    return None;
                }
                pos += 1;
            }
            Token::Year4 => set(&mut p.year, take_digits(&s, &mut pos, 4, 4)? as i32)?,
            Token::Year2 => {
                let yy = take_digits(&s, &mut pos, 2, 2)? as i32;
                set(&mut p.year, if yy < 50 { 2000 + yy } else { 1900 + yy })?
            }
            Token::Month(n) => set(&mut p.month, take_digits(&s, &mut pos, *n, 2)?)?,
            Token::MonthName { full } => {
                let rest: String = s[pos..].iter().collect::<String>().to_lowercase();
                let (idx, len) = MONTHS.iter().enumerate().find_map(|(i, name)| {
                    let name = if *full { *name } else { &name[..3] };
                    rest.starts_with(&name.to_lowercase())
                        .then(|| (i, name.chars().count()))
                })?;
                pos += len;
                set(&mut p.month, idx as u32 + 1)?
            }
            Token::Day(n) => set(&mut p.day, take_digits(&s, &mut pos, *n, 2)?)?,
            Token::Hour12(n) => {
                p.hour12 = true;
                set(&mut p.hour, take_digits(&s, &mut pos, *n, 2)?)?
            }
            Token::Hour24(n) => set(&mut p.hour, take_digits(&s, &mut pos, *n, 2)?)?,
            Token::Minute(n) => set(&mut p.minute, take_digits(&s, &mut pos, *n, 2)?)?,
            Token::Second(n) => set(&mut p.second, take_digits(&s, &mut pos, *n, 2)?)?,
            Token::Fraction(n) => {
                let mut nanos: u64 = 0;
                for k in 0..*n {
                    let d = s.get(pos + k)?.to_digit(10)?;
                    if k < 9 {
                        nanos += u64::from(d) * 10u64.pow(8 - k as u32);
                    }
                }
                pos += n;
                set(&mut p.fraction, nanos as u32)?
            }
            Token::AmPm => {
                let pm = match s.get(pos)?.to_ascii_uppercase() {
                    'A' => false,
                    'P' => true,
                    _ => return None,
                };
                pos += 1;
                if let Some(m) = s.get(pos)
                    && m.eq_ignore_ascii_case(&'M')
                {
                    pos += 1;
                }
                set(&mut p.pm, pm)?
            }
            Token::Offset => {
                let off = match s.get(pos)? {
                    'Z' => {
                        pos += 1;
                        0
                    }
                    sign @ ('+' | '-') => {
                        pos += 1;
                        let hh = take_digits(&s, &mut pos, 2, 2)?;
                        if s.get(pos) == Some(&':') {
                            pos += 1;
                        }
                        let mm = take_digits(&s, &mut pos, 2, 2)?;
                        if hh > 23 || mm > 59 {
                            return None;
                        }
                        let secs = (hh * 3600 + mm * 60) as i32;
                        if *sign == '-' { -secs } else { secs }
                    }
                    _ => return None,
                };
                set(&mut p.offset, off)?
            }
        }
    }
    (pos == s.len()).then_some(p)
}

impl Parsed {
    /// Resolve the 12-hour clock; `Some(None)` means no hour field.
    fn hour24(&self) -> Option<Option<u32>> {
        match (self.hour, self.hour12) {
            (None, _) => Some(None),
            (Some(h), false) => (h < 24).then_some(Some(h)),
            (Some(h), true) => {
                if !(1..=12).contains(&h) {
                    return None;
                }
                let pm = self.pm?;
                Some(Some(h % 12 + if pm { 12 } else { 0 }))
            }
        }
    }

    fn date_precision(&self) -> Option<TemporalPrecision> {
        self.year?;
        match (self.month, self.day) {
            (None, None) => Some(TemporalPrecision::Year),
            (Some(_), None) => Some(TemporalPrecision::Month),
            (Some(_), Some(_)) => Some(TemporalPrecision::Day),
            (None, Some(_)) => None,
        }
    }

    fn naive_date(&self) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(self.year?, self.month.unwrap_or(1), self.day.unwrap_or(1))
    }
}

/// Parse `input` with `format` into a `Date`. Precision follows the date
/// components the format provides. Returns `None` if the input does not match.
pub fn parse_date_with_format(input: &str, format: &str) -> Option<PrecisionDate> {
    let p = parse_fields(input, format)?;
    let precision = p.date_precision()?;
    p.hour24()?; // validate any time fields present
    Some(PrecisionDate::new(p.naive_date()?, precision))
}

/// Parse `input` with `format` into a `DateTime`. Precision is the finest
/// contiguous component the format provides.
pub fn parse_datetime_with_format(input: &str, format: &str) -> Option<PrecisionDateTime> {
    let p = parse_fields(input, format)?;
    let mut precision = p.date_precision()?;
    let date = p.naive_date()?;
    let hour = p.hour24()?;

    let (mut h, mut mi, mut sec, mut nanos) = (0, 0, 0, 0);
    let has_time =
        hour.is_some() || p.minute.is_some() || p.second.is_some() || p.fraction.is_some();
    if has_time {
        // time components must be contiguous and need a full date
        if precision != TemporalPrecision::Day {
            return None;
        }
        h = hour?;
        precision = TemporalPrecision::Hour;
        if p.minute.is_some() {
            mi = p.minute?;
            precision = TemporalPrecision::Minute;
        }
        if p.second.is_some() {
            if precision != TemporalPrecision::Minute {
                return None;
            }
            sec = p.second?;
            precision = TemporalPrecision::Second;
        }
        if p.fraction.is_some() {
            if precision != TemporalPrecision::Second {
                return None;
            }
            nanos = p.fraction?;
            precision = TemporalPrecision::Millisecond;
        }
    }

    let time = NaiveTime::from_hms_nano_opt(h, mi, sec, nanos)?;
    let offset = FixedOffset::east_opt(p.offset.unwrap_or(0))?;
    let utc = date.and_time(time) - chrono::Duration::seconds(i64::from(offset.local_minus_utc()));
    let dt: DateTime<FixedOffset> = DateTime::from_naive_utc_and_offset(utc, offset);
    Some(PrecisionDateTime::new_with_tz(
        dt,
        precision,
        p.offset.is_some(),
    ))
}

/// Extract the optional `format` argument of the conversion functions.
/// An absent or empty argument means "use the default format".
pub fn format_arg(args: &[Collection], function: &str) -> Result<Option<String>> {
    let err = |msg: String| FhirPathError::evaluation_error(crate::core::error_code::FP0053, msg);
    if args.len() > 1 {
        return Err(err(format!(
            "{function} function takes at most one argument"
        )));
    }
    match args.first() {
        None => Ok(None),
        Some(c) if c.is_empty() => Ok(None),
        Some(c) if c.len() == 1 => match &c[0] {
            FhirPathValue::String(s, _, _) => Ok(Some(s.to_string())),
            _ => Err(err(format!("{function} format argument must be a string"))),
        },
        Some(_) => Err(err(format!(
            "{function} format argument must be a single string"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn date_spec_examples() {
        let d = parse_date_with_format("150124", "ddMMyy").unwrap();
        assert_eq!(d.date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        assert_eq!(d.precision, TemporalPrecision::Day);
        let d = parse_date_with_format("15-01-2024", "dd-MM-yyyy").unwrap();
        assert_eq!(d.date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        let d = parse_date_with_format("12-27", "MM-yy").unwrap();
        assert_eq!((d.date.year(), d.date.month()), (2027, 12));
        assert_eq!(d.precision, TemporalPrecision::Month);
    }

    #[test]
    fn two_digit_year_century() {
        assert_eq!(
            parse_date_with_format("49", "yy").unwrap().date.year(),
            2049
        );
        assert_eq!(
            parse_date_with_format("50", "yy").unwrap().date.year(),
            1950
        );
        assert_eq!(
            parse_date_with_format("50", "yy").unwrap().precision,
            TemporalPrecision::Year
        );
    }

    #[test]
    fn variable_width_and_names() {
        let d = parse_date_with_format("5/3/2024", "d/M/yyyy").unwrap();
        assert_eq!((d.date.day(), d.date.month()), (5, 3));
        let d = parse_date_with_format("15 Jun 2024", "d MMM yyyy").unwrap();
        assert_eq!(d.date.month(), 6);
        let d = parse_date_with_format("15 june 2024", "dd MMMM yyyy").unwrap();
        assert_eq!(d.date.month(), 6);
    }

    #[test]
    fn non_matching_is_none() {
        assert!(parse_date_with_format("2024-01-15", "dd-MM-yyyy").is_none());
        assert!(parse_date_with_format("15-01-2024x", "dd-MM-yyyy").is_none());
        assert!(parse_date_with_format("32-01-2024", "dd-MM-yyyy").is_none());
        assert!(parse_date_with_format("1-01-2024", "dd-MM-yyyy").is_none());
        assert!(parse_date_with_format("15", "dd").is_none()); // no year
        assert!(parse_date_with_format("2024", "yyy").is_none()); // bad code
        assert!(parse_date_with_format("2024 2024", "yyyy yyyy").is_none());
    }

    #[test]
    fn datetime_full() {
        let dt = parse_datetime_with_format(
            "15/01/2024 11:30:45.123 PM +0200",
            "dd/MM/yyyy hh:mm:ss.SSS a Z",
        )
        .unwrap();
        assert_eq!(dt.precision, TemporalPrecision::Millisecond);
        assert!(dt.tz_specified);
        assert_eq!(dt.datetime.hour(), 23);
        assert_eq!(dt.datetime.nanosecond(), 123_000_000);
        assert_eq!(dt.datetime.offset().local_minus_utc(), 7200);
    }

    #[test]
    fn datetime_precision() {
        let dt = parse_datetime_with_format("2024-01-15 07", "yyyy-MM-dd HH").unwrap();
        assert_eq!(dt.precision, TemporalPrecision::Hour);
        assert!(!dt.tz_specified);
        let dt = parse_datetime_with_format("2024-01", "yyyy-MM").unwrap();
        assert_eq!(dt.precision, TemporalPrecision::Month);
        let dt = parse_datetime_with_format("2024-01-15T10:00Z", "yyyy-MM-ddTHH:mmZ").unwrap();
        assert_eq!(dt.precision, TemporalPrecision::Minute);
        assert!(dt.tz_specified);
    }

    #[test]
    fn datetime_rejects_gaps_and_bad_clock() {
        assert!(parse_datetime_with_format("10:00", "HH:mm").is_none());
        assert!(parse_datetime_with_format("2024-01-15 10", "yyyy-MM-dd mm").is_none());
        assert!(parse_datetime_with_format("2024-01-15 13 PM", "yyyy-MM-dd hh a").is_none());
        assert!(parse_datetime_with_format("2024-01-15 10", "yyyy-MM-dd hh").is_none());
        let dt = parse_datetime_with_format("2024-01-15 12 AM", "yyyy-MM-dd hh a").unwrap();
        assert_eq!(dt.datetime.hour(), 0);
    }

    #[test]
    fn format_arg_handling() {
        assert_eq!(format_arg(&[], "f").unwrap(), None);
        let s = Collection::from(vec![FhirPathValue::string("yy")]);
        assert_eq!(format_arg(&[s], "f").unwrap(), Some("yy".to_string()));
        let i = Collection::from(vec![FhirPathValue::integer(1)]);
        assert!(format_arg(&[i], "f").is_err());
    }
}
