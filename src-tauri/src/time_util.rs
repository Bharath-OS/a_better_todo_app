//! Local-day boundary helpers.
//!
//! Day IDs are always computed in *local* civil time (not UTC), with an optional
//! `day_starts_at` offset (minutes after local midnight) for night-owl schedules.

use chrono::{DateTime, Duration, Local, NaiveDate, Timelike};

/// Parse `HH:MM` or `H:MM` into minutes after midnight. Defaults to 0 on error.
pub fn parse_day_starts_at(s: &str) -> i64 {
    let parts: Vec<_> = s.trim().split(':').collect();
    if parts.len() != 2 {
        return 0;
    }
    let Ok(h) = parts[0].parse::<i64>() else {
        return 0;
    };
    let Ok(m) = parts[1].parse::<i64>() else {
        return 0;
    };
    if !(0..24).contains(&h) || !(0..60).contains(&m) {
        return 0;
    }
    h * 60 + m
}

/// Local "logical day" for `now` given day-starts-at minutes after midnight.
pub fn local_day_id(now: DateTime<Local>, day_starts_at_minutes: i64) -> NaiveDate {
    let local_date = now.date_naive();
    let minutes = now.time().num_seconds_from_midnight() as i64 / 60;
    if minutes < day_starts_at_minutes {
        local_date - Duration::days(1)
    } else {
        local_date
    }
}

/// Format a local day id as `YYYY-MM-DD`.
pub fn format_day_id(day: NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}

/// Today's local day id using the process local timezone.
pub fn today_local_day_id(day_starts_at: &str) -> String {
    let mins = parse_day_starts_at(day_starts_at);
    format_day_id(local_day_id(Local::now(), mins))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn parse_day_starts_at_ok() {
        assert_eq!(parse_day_starts_at("00:00"), 0);
        assert_eq!(parse_day_starts_at("4:00"), 240);
        assert_eq!(parse_day_starts_at("04:30"), 270);
        assert_eq!(parse_day_starts_at("bad"), 0);
    }

    #[test]
    fn midnight_boundary_default() {
        let now = Local
            .with_ymd_and_hms(2026, 11, 12, 0, 30, 0)
            .single()
            .expect("local");
        assert_eq!(
            local_day_id(now, 0),
            NaiveDate::from_ymd_opt(2026, 11, 12).unwrap()
        );
    }

    #[test]
    fn night_owl_before_4am_belongs_to_previous_day() {
        let now = Local
            .with_ymd_and_hms(2026, 11, 12, 3, 15, 0)
            .single()
            .expect("local");
        assert_eq!(
            local_day_id(now, 4 * 60),
            NaiveDate::from_ymd_opt(2026, 11, 11).unwrap()
        );
        let after = Local
            .with_ymd_and_hms(2026, 11, 12, 4, 0, 0)
            .single()
            .expect("local");
        assert_eq!(
            local_day_id(after, 4 * 60),
            NaiveDate::from_ymd_opt(2026, 11, 12).unwrap()
        );
    }

    #[test]
    fn format_day_id_iso() {
        let d = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
        assert_eq!(format_day_id(d), "2026-01-05");
    }

    #[test]
    fn uses_local_civil_date_not_utc_api() {
        let local_now = Local::now();
        let id = local_day_id(local_now, 0);
        assert_eq!(id, local_now.date_naive());
    }

    #[test]
    fn parse_rejects_out_of_range() {
        assert_eq!(parse_day_starts_at("24:00"), 0);
        assert_eq!(parse_day_starts_at("12:60"), 0);
        assert_eq!(parse_day_starts_at("-1:00"), 0);
        assert_eq!(parse_day_starts_at(""), 0);
        assert_eq!(parse_day_starts_at("noon"), 0);
    }

    #[test]
    fn exactly_at_day_start_belongs_to_new_day() {
        let at_boundary = Local
            .with_ymd_and_hms(2026, 11, 12, 4, 0, 0)
            .single()
            .expect("local");
        assert_eq!(
            local_day_id(at_boundary, 240),
            NaiveDate::from_ymd_opt(2026, 11, 12).unwrap()
        );
        let just_before = Local
            .with_ymd_and_hms(2026, 11, 12, 3, 59, 0)
            .single()
            .expect("local");
        assert_eq!(
            local_day_id(just_before, 240),
            NaiveDate::from_ymd_opt(2026, 11, 11).unwrap()
        );
    }

    #[test]
    fn today_local_day_id_format() {
        let id = today_local_day_id("00:00");
        assert_eq!(id.len(), 10);
        assert_eq!(&id[4..5], "-");
        assert_eq!(&id[7..8], "-");
    }
}
