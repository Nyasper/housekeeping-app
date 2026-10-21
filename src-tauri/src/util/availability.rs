use chrono::NaiveDate;

use crate::util::dates::{is_date_within, parse_date, weekday_index};

/// Returns `true` when a seasonal staff member is within their configured
/// season on `date`. Non-seasonal staff or seasonal staff without dates are
/// always considered within season.
pub fn is_within_season(
    is_seasonal: bool,
    season_start_date: Option<&str>,
    season_end_date: Option<&str>,
    date: NaiveDate,
) -> bool {
    if !is_seasonal {
        return true;
    }
    let (Some(start_raw), Some(end_raw)) = (season_start_date, season_end_date) else {
        return true;
    };
    let (Ok(start), Ok(end)) = (
        parse_date(start_raw, "inicio de temporada"),
        parse_date(end_raw, "fin de temporada"),
    ) else {
        return true;
    };
    is_date_within(date, start, end)
}

/// Checks whether a staff member is available on `date` according to:
/// 1. `is_active`
/// 2. Season window (if seasonal with dates)
/// 3. Weekly rest days (`0 = Monday … 6 = Sunday`)
/// 4. Punctual absences (`[start_date, end_date]` inclusive)
pub fn is_staff_available_on(
    is_active: bool,
    is_seasonal: bool,
    season_start_date: Option<&str>,
    season_end_date: Option<&str>,
    rest_days: &[i64],
    absence_ranges: &[(NaiveDate, NaiveDate)],
    date: NaiveDate,
) -> bool {
    if !is_active {
        return false;
    }
    if !is_within_season(is_seasonal, season_start_date, season_end_date, date) {
        return false;
    }
    let weekday = weekday_index(date);
    if rest_days.contains(&weekday) {
        return false;
    }
    if absence_ranges
        .iter()
        .any(|(start, end)| is_date_within(date, *start, *end))
    {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seasonal_and_rest_days_and_absences_govern_availability() {
        let d_in = NaiveDate::from_ymd_opt(2026, 1, 12).expect("Monday"); // weekday 0
        let d_tue = NaiveDate::from_ymd_opt(2026, 1, 13).expect("Tuesday"); // weekday 1
        let d_out = NaiveDate::from_ymd_opt(2026, 3, 10).expect("Out of season");

        // Inactive -> false
        assert!(!is_staff_available_on(
            false,
            false,
            None,
            None,
            &[],
            &[],
            d_tue
        ));

        // Seasonal within vs outside season
        assert!(is_staff_available_on(
            true,
            true,
            Some("2026-01-01"),
            Some("2026-02-28"),
            &[],
            &[],
            d_tue
        ));
        assert!(!is_staff_available_on(
            true,
            true,
            Some("2026-01-01"),
            Some("2026-02-28"),
            &[],
            &[],
            d_out
        ));

        // Rest day on Monday (0)
        assert!(!is_staff_available_on(
            true,
            false,
            None,
            None,
            &[0],
            &[],
            d_in
        ));

        // Punctual absence covering Tuesday
        let abs_start = NaiveDate::from_ymd_opt(2026, 1, 13).expect("start");
        let abs_end = NaiveDate::from_ymd_opt(2026, 1, 15).expect("end");
        assert!(!is_staff_available_on(
            true,
            false,
            None,
            None,
            &[],
            &[(abs_start, abs_end)],
            d_tue
        ));
    }
}
