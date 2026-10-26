use chrono::{Datelike, Local, NaiveDate, NaiveTime};

use crate::error::CommandError;

/// Returns today's local date in ISO format (`YYYY-MM-DD`).
pub fn today_iso() -> String {
    Local::now().date_naive().format("%Y-%m-%d").to_string()
}

/// Parses an ISO date string (`YYYY-MM-DD`).
pub fn parse_date(value: &str, field_label: &str) -> Result<NaiveDate, CommandError> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").map_err(|_| {
        CommandError::new(
            "validation",
            format!("La fecha {field_label} no es válida (formato esperado: AAAA-MM-DD)."),
        )
    })
}

/// Normalizes an optional ISO date string (`YYYY-MM-DD`), returning `None` when
/// the input is `None` or blank.
pub fn normalize_optional_date(
    value: Option<&str>,
    field_label: &str,
) -> Result<Option<String>, CommandError> {
    let Some(raw) = value.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let parsed = parse_date(raw, field_label)?;
    Ok(Some(parsed.format("%Y-%m-%d").to_string()))
}

/// Parses a 24-hour time string (`HH:MM`).
pub fn parse_time(value: &str, field_label: &str) -> Result<NaiveTime, CommandError> {
    NaiveTime::parse_from_str(value.trim(), "%H:%M").map_err(|_| {
        CommandError::new(
            "validation",
            format!("La hora de {field_label} no es válida (formato esperado: HH:MM)."),
        )
    })
}

/// Normalizes an optional `HH:MM` time string, returning `None` when blank.
pub fn normalize_optional_time(
    value: Option<&str>,
    field_label: &str,
) -> Result<Option<String>, CommandError> {
    let Some(raw) = value.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let parsed = parse_time(raw, field_label)?;
    Ok(Some(parsed.format("%H:%M").to_string()))
}

/// Ensures `start <= end` when both dates are present.
pub fn validate_date_range(
    start: &str,
    end: &str,
    range_label: &str,
) -> Result<(NaiveDate, NaiveDate), CommandError> {
    let start_date = parse_date(start, &format!("de inicio {range_label}"))?;
    let end_date = parse_date(end, &format!("de fin {range_label}"))?;
    if start_date > end_date {
        return Err(CommandError::new(
            "validation",
            format!("La fecha de inicio {range_label} no puede ser posterior a la fecha de fin."),
        ));
    }
    Ok((start_date, end_date))
}

/// Returns the ISO weekday index (`0 = Monday … 6 = Sunday`) for `date`.
pub fn weekday_index(date: NaiveDate) -> i64 {
    i64::from(date.weekday().num_days_from_monday())
}

/// Checks whether two inclusive date ranges `[start_a, end_a]` and `[start_b, end_b]` overlap.
pub fn ranges_overlap(
    start_a: NaiveDate,
    end_a: NaiveDate,
    start_b: NaiveDate,
    end_b: NaiveDate,
) -> bool {
    start_a <= end_b && end_a >= start_b
}

/// Checks whether `date` lies within `[start, end]` inclusive.
pub fn is_date_within(date: NaiveDate, start: NaiveDate, end: NaiveDate) -> bool {
    date >= start && date <= end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_dates_and_rejects_invalid_ones() {
        let date = parse_date("2026-10-21", "de prueba").expect("valid date");
        assert_eq!(date.year(), 2026);
        assert_eq!(date.month(), 10);
        assert_eq!(date.day(), 21);
        // 2026-10-21 is a Wednesday -> 2
        assert_eq!(weekday_index(date), 2);

        let err = parse_date("21/10/2026", "de prueba").expect_err("invalid format");
        assert_eq!(err.code, "validation");
    }

    #[test]
    fn validates_date_ranges_and_overlaps() {
        let (s1, e1) =
            validate_date_range("2026-01-10", "2026-01-15", "de la licencia").expect("range");
        let (s2, e2) =
            validate_date_range("2026-01-15", "2026-01-20", "de la licencia").expect("range");
        let (s3, e3) =
            validate_date_range("2026-01-16", "2026-01-20", "de la licencia").expect("range");

        assert!(ranges_overlap(s1, e1, s2, e2));
        assert!(!ranges_overlap(s1, e1, s3, e3));

        let err = validate_date_range("2026-01-20", "2026-01-10", "de la licencia")
            .expect_err("inverted range");
        assert_eq!(err.code, "validation");
    }
}
