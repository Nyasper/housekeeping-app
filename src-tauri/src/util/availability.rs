use chrono::NaiveDate;

use crate::settings::AppSettings;
use crate::util::dates::{is_date_within, parse_date, weekday_index};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceTaskRecommendation {
    pub effective_status: String,
    pub recommended_service_type: Option<String>,
}

/// Computes `effective_status` and `recommended_service_type` for a place on
/// `reference_date` following the priority rules in `PLAN.md` §3.2.1.
pub fn compute_recommended_task(
    place_type: &str,
    manual_status: Option<&str>,
    check_in_date: Option<&str>,
    check_out_date: Option<&str>,
    reference_date: NaiveDate,
    settings: AppSettings,
) -> PlaceTaskRecommendation {
    if place_type != "apartment" {
        return PlaceTaskRecommendation {
            effective_status: "vacant".to_string(),
            recommended_service_type: None,
        };
    }

    if manual_status == Some("cancelled") {
        return PlaceTaskRecommendation {
            effective_status: "cancelled".to_string(),
            recommended_service_type: None,
        };
    }

    let check_in = check_in_date.and_then(|d| parse_date(d, "ingreso").ok());
    let check_out = check_out_date.and_then(|d| parse_date(d, "egreso").ok());

    let is_occupied = match (check_in, check_out) {
        (Some(ci), Some(co)) => ci <= reference_date && reference_date <= co,
        _ => false,
    };

    let (derived_status, recommended_service_type) = if let Some(co) = check_out
        && (0..=settings.recent_vacated_days).contains(&(reference_date - co).num_days())
    {
        ("recently_vacated", Some("deep_cleaning".to_string()))
    } else if let Some(ci) = check_in
        && !is_occupied
        && (0..=settings.upcoming_occupancy_days).contains(&(ci - reference_date).num_days())
    {
        ("upcoming_occupancy", Some("touch_up".to_string()))
    } else if is_occupied {
        ("occupied", Some("daily_cleaning".to_string()))
    } else {
        ("vacant", None)
    };

    let effective_status = manual_status.unwrap_or(derived_status).to_string();

    PlaceTaskRecommendation {
        effective_status,
        recommended_service_type,
    }
}

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

    #[test]
    fn recommended_task_follows_priority_rules() {
        let ref_date = NaiveDate::from_ymd_opt(2026, 10, 21).expect("ref date");
        let settings = AppSettings::default(); // 2 days each

        // 1. Cancelled manual override -> no recommendation
        let rec = compute_recommended_task(
            "apartment",
            Some("cancelled"),
            Some("2026-10-18"),
            Some("2026-10-21"),
            ref_date,
            settings,
        );
        assert_eq!(rec.effective_status, "cancelled");
        assert_eq!(rec.recommended_service_type, None);

        // 2. Checkout today -> recently_vacated + deep_cleaning
        let rec = compute_recommended_task(
            "apartment",
            None,
            Some("2026-10-18"),
            Some("2026-10-21"),
            ref_date,
            settings,
        );
        assert_eq!(rec.effective_status, "recently_vacated");
        assert_eq!(
            rec.recommended_service_type.as_deref(),
            Some("deep_cleaning")
        );

        // 3. Check-in in 2 days -> upcoming_occupancy + touch_up
        let rec = compute_recommended_task(
            "apartment",
            None,
            Some("2026-10-23"),
            Some("2026-10-28"),
            ref_date,
            settings,
        );
        assert_eq!(rec.effective_status, "upcoming_occupancy");
        assert_eq!(rec.recommended_service_type.as_deref(), Some("touch_up"));

        // 4. Occupied -> occupied + daily_cleaning
        let rec = compute_recommended_task(
            "apartment",
            None,
            Some("2026-10-19"),
            Some("2026-10-25"),
            ref_date,
            settings,
        );
        assert_eq!(rec.effective_status, "occupied");
        assert_eq!(
            rec.recommended_service_type.as_deref(),
            Some("daily_cleaning")
        );

        // 5. Partial dates (only check_out_date within window, only check_in_date within window, no dates)
        let rec_out_only = compute_recommended_task(
            "apartment",
            None,
            None,
            Some("2026-10-20"),
            ref_date,
            settings,
        );
        assert_eq!(rec_out_only.effective_status, "recently_vacated");
        assert_eq!(
            rec_out_only.recommended_service_type.as_deref(),
            Some("deep_cleaning")
        );

        let rec_in_only = compute_recommended_task(
            "apartment",
            None,
            Some("2026-10-22"),
            None,
            ref_date,
            settings,
        );
        assert_eq!(rec_in_only.effective_status, "upcoming_occupancy");
        assert_eq!(
            rec_in_only.recommended_service_type.as_deref(),
            Some("touch_up")
        );

        let rec_none = compute_recommended_task("apartment", None, None, None, ref_date, settings);
        assert_eq!(rec_none.effective_status, "vacant");
        assert_eq!(rec_none.recommended_service_type, None);

        // Manual override other than cancelled keeps date-based recommendation
        let rec_override = compute_recommended_task(
            "apartment",
            Some("vacant"),
            Some("2026-10-19"),
            Some("2026-10-25"),
            ref_date,
            settings,
        );
        assert_eq!(rec_override.effective_status, "vacant");
        assert_eq!(
            rec_override.recommended_service_type.as_deref(),
            Some("daily_cleaning")
        );
    }
}
