// ~/src/tests.rs
//
//  * Copyright (C) ParsiCore (parsidate) 2024-2025 <parsicore.dev@gmail.com>
//  * Package : parsidate
//  * License : Apache-2.0
//  * Version : 1.7.1
//  * URL     : https://github.com/parsicore/parsidate
//  * Sign: parsidate-20250607-fea13e856dcd-459c6e73c83e49e10162ee28b26ac7cd
//
//! Unit tests for the parsidate library.

// Add a new module for DateTime tests
#[cfg(test)]
mod datetime_tests {
    // Import necessary items, including those from the outer scope if needed
    use crate::{DateError, ParseErrorKind, ParsiDate, ParsiDateTime};
    use chrono::{Duration, NaiveDate};

    // Helper function for creating ParsiDateTime, panicking on failure
    fn pdt(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> ParsiDateTime {
        ParsiDateTime::new(year, month, day, hour, minute, second).unwrap_or_else(|e| {
            panic!(
                "Invalid test datetime {}-{}-{} {}:{}:{}: {:?}",
                year, month, day, hour, minute, second, e
            )
        })
    }

    // Helper for ParsiDate
    fn pd(year: i32, month: u32, day: u32) -> ParsiDate {
        ParsiDate::new(year, month, day).unwrap()
    }

    // --- Constructor & Validation Tests ---
    #[test]
    fn test_new_datetime() {
        assert!(ParsiDateTime::new(1403, 5, 2, 10, 30, 0).is_ok());
        assert_eq!(
            ParsiDateTime::new(1403, 5, 2, 24, 0, 0), // Invalid hour
            Err(DateError::InvalidTime)
        );
        assert_eq!(
            ParsiDateTime::new(1403, 5, 2, 10, 60, 0), // Invalid minute
            Err(DateError::InvalidTime)
        );
        assert_eq!(
            ParsiDateTime::new(1403, 5, 2, 10, 0, 60), // Invalid second
            Err(DateError::InvalidTime)
        );
        assert_eq!(
            ParsiDateTime::new(1404, 12, 30, 10, 0, 0), // Invalid date part
            Err(DateError::InvalidDate)
        );
    }

    #[test]
    fn test_from_date_and_time() {
        let date = pd(1403, 1, 1);
        assert!(ParsiDateTime::from_date_and_time(date, 0, 0, 0).is_ok());
        assert!(ParsiDateTime::from_date_and_time(date, 23, 59, 59).is_ok());
        assert_eq!(
            ParsiDateTime::from_date_and_time(date, 24, 0, 0),
            Err(DateError::InvalidTime)
        );
    }

    #[test]
    fn test_is_valid_datetime() {
        assert!(pdt(1403, 12, 30, 23, 59, 59).is_valid()); // Leap year end, valid time
        assert!(!unsafe { ParsiDateTime::new_unchecked(1404, 12, 30, 10, 0, 0) }.is_valid()); // Invalid date part
        assert!(!unsafe { ParsiDateTime::new_unchecked(1403, 12, 30, 24, 0, 0) }.is_valid()); // Invalid time part
        assert!(!unsafe { ParsiDateTime::new_unchecked(1404, 12, 30, 24, 0, 0) }.is_valid());
        // Both invalid
    }

    // --- Conversion Tests ---
    #[test]
    fn test_gregorian_to_persian_datetime() {
        let g_dt = NaiveDate::from_ymd_opt(2024, 7, 23)
            .unwrap()
            .and_hms_opt(15, 30, 45)
            .unwrap();
        let expected_pdt = pdt(1403, 5, 2, 15, 30, 45);
        assert_eq!(ParsiDateTime::from_gregorian(g_dt), Ok(expected_pdt));

        // Test epoch start with time
        let g_epoch_dt = NaiveDate::from_ymd_opt(622, 3, 21)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let expected_p_epoch_dt = pdt(1, 1, 1, 0, 0, 0);
        assert_eq!(
            ParsiDateTime::from_gregorian(g_epoch_dt),
            Ok(expected_p_epoch_dt)
        );

        // Test before epoch (should fail)
        let g_before_epoch = NaiveDate::from_ymd_opt(622, 3, 20)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        assert_eq!(
            ParsiDateTime::from_gregorian(g_before_epoch),
            Err(DateError::GregorianConversionError)
        );
    }

    #[test]
    fn test_persian_to_gregorian_datetime() {
        let p_dt = pdt(1403, 5, 2, 15, 30, 45);
        let expected_g_dt = NaiveDate::from_ymd_opt(2024, 7, 23)
            .unwrap()
            .and_hms_opt(15, 30, 45)
            .unwrap();
        assert_eq!(p_dt.to_gregorian(), Ok(expected_g_dt));

        // Test invalid datetime conversion attempt
        let invalid_dt_time = unsafe { ParsiDateTime::new_unchecked(1403, 5, 2, 24, 0, 0) };
        assert!(!invalid_dt_time.is_valid());
        assert_eq!(invalid_dt_time.to_gregorian(), Err(DateError::InvalidTime)); // Fails validation

        let invalid_dt_date = unsafe { ParsiDateTime::new_unchecked(1404, 12, 30, 10, 0, 0) };
        assert!(!invalid_dt_date.is_valid());
        assert_eq!(invalid_dt_date.to_gregorian(), Err(DateError::InvalidDate));
        // Fails validation
    }

    #[test]
    fn test_now_function() {
        match ParsiDateTime::now() {
            Ok(now) => {
                println!("Current Persian DateTime (captured by test): {}", now);
                assert!(
                    now.is_valid(),
                    "ParsiDateTime::now() returned an invalid object"
                );
                // Check against chrono::Local::now() loosely
                let chrono_now_naive = chrono::Local::now().naive_local();
                let converted_back = now.to_gregorian().unwrap();
                // Allow a small difference (e.g., 1 second) due to potential clock tick between calls
                let diff = (chrono_now_naive - converted_back).num_seconds().abs();
                assert!(
                    diff <= 1,
                    "Difference between now() and chrono local time is too large: {}s",
                    diff
                );
            }
            Err(e) => panic!("ParsiDateTime::now() failed: {}", e),
        }
    }

    #[test]
    fn test_week_of_year() {
        // --- Year 1403 (Leap Year, starts on Wednesday - weekday 4) ---
        // First day is week 1
        assert_eq!(pd(1403, 1, 1).week_of_year(), Ok(1)); // Wed
        assert_eq!(pd(1403, 1, 2).week_of_year(), Ok(1)); // Thu
        assert_eq!(pd(1403, 1, 3).week_of_year(), Ok(1)); // Fri
                                                          // Start of week 2
        assert_eq!(pd(1403, 1, 4).week_of_year(), Ok(2)); // Sat
        assert_eq!(pd(1403, 1, 10).week_of_year(), Ok(2)); // Fri
                                                           // Start of week 3
        assert_eq!(pd(1403, 1, 11).week_of_year(), Ok(3)); // Sat
                                                           // Mid-year
        assert_eq!(pd(1403, 5, 2).week_of_year(), Ok(19)); // Ordinal 126 -> Effective 130 -> Week 19
                                                           // End of year
        assert_eq!(pd(1403, 12, 29).week_of_year(), Ok(53)); // Ordinal 365 -> Effective 369 -> Week 53
        assert_eq!(pd(1403, 12, 30).week_of_year(), Ok(53)); // Ordinal 366 -> Effective 370 -> Week 53

        // --- Year 1404 (Common Year, starts on Friday - weekday 6) ---
        // First day is week 1
        assert_eq!(pd(1404, 1, 1).week_of_year(), Ok(1)); // Fri
                                                          // Start of week 2
        assert_eq!(pd(1404, 1, 2).week_of_year(), Ok(2)); // Sat
        assert_eq!(pd(1404, 1, 8).week_of_year(), Ok(2)); // Fri
                                                          // Start of week 3
        assert_eq!(pd(1404, 1, 9).week_of_year(), Ok(3)); // Sat
                                                          // End of year
        assert_eq!(pd(1404, 12, 28).week_of_year(), Ok(53)); // Ordinal 364 -> Effective 370 -> Week 53
        assert_eq!(pd(1404, 12, 29).week_of_year(), Ok(53)); // Ordinal 365 -> Effective 371 -> Week 53

        // --- Year 1 (Common Year, starts on Friday? Check conversion) ---
        // 1/1/1 Parsi = 622-03-21 Gregorian (Friday) -> weekday 6
        assert_eq!(pd(1, 1, 1).week_of_year(), Ok(1)); // Fri
        assert_eq!(pd(1, 1, 2).week_of_year(), Ok(1)); // Sat (Start of Week 2)

        // Test ParsiDateTime delegation
        let dt = crate::datetime::ParsiDateTime::new(1403, 1, 4, 10, 0, 0).unwrap(); // Week 2
        assert_eq!(dt.week_of_year(), Ok(2));

        // Test Error Case (invalid date)
        let invalid_date = unsafe { ParsiDate::new_unchecked(1400, 13, 1) };
        assert!(matches!(
            invalid_date.week_of_year(),
            Err(DateError::InvalidDate)
        ));
    }

    // --- Formatting Tests ---
    #[test]
    fn test_format_datetime() {
        let dt = pdt(1403, 5, 2, 8, 5, 3);
        let dt_pm = pdt(1403, 10, 15, 22, 59, 59);

        // Default Display
        assert_eq!(dt.to_string(), "1403/05/02 08:05:03");
        assert_eq!(dt_pm.to_string(), "1403/10/15 22:59:59");

        // Custom formats including time
        assert_eq!(dt.format("%Y-%m-%d %H:%M:%S"), "1403-05-02 08:05:03");
        assert_eq!(dt_pm.format("%Y-%m-%d %H:%M:%S"), "1403-10-15 22:59:59");
        assert_eq!(dt.format("%d %B %Y ساعت %H:%M"), "02 مرداد 1403 ساعت 08:05"); // Padded H, M
        assert_eq!(dt.format("%T"), "08:05:03");
        assert_eq!(dt_pm.format("%T"), "22:59:59");
        assert_eq!(dt.format("%Y%m%dT%H%M%S"), "14030502T080503");

        // Combining date and time specifiers
        assert_eq!(
            dt.format("%A %d %B - %H hours"),
            "سه‌شنبه 02 مرداد - 08 hours"
        );
    }

    // --- Parsing Tests ---
    #[test]
    fn test_parse_datetime() {
        let s1 = "1403/05/02 15:30:45";
        let fmt1 = "%Y/%m/%d %H:%M:%S";
        assert_eq!(
            ParsiDateTime::parse(s1, fmt1),
            Ok(pdt(1403, 5, 2, 15, 30, 45))
        );

        let s2 = "1399-12-30T23:59:01"; // Leap year end
        let fmt2 = "%Y-%m-%dT%T";
        assert_eq!(
            ParsiDateTime::parse(s2, fmt2),
            Ok(pdt(1399, 12, 30, 23, 59, 1))
        );

        let s3 = "01 فروردین 1400 00:00:00";
        let fmt3 = "%d %B %Y %H:%M:%S";
        assert_eq!(ParsiDateTime::parse(s3, fmt3), Ok(pdt(1400, 1, 1, 0, 0, 0)));

        // Error cases
        assert_eq!(
            ParsiDateTime::parse("1403/05/02 24:00:00", fmt1),
            Err(DateError::ParseError(ParseErrorKind::InvalidTimeValue))
        ); // Invalid hour
        assert_eq!(
            ParsiDateTime::parse("1403/05/02 15:60:00", fmt1),
            Err(DateError::ParseError(ParseErrorKind::InvalidTimeValue))
        ); // Invalid minute
        assert_eq!(
            ParsiDateTime::parse("1403/05/02 15:00:60", fmt1),
            Err(DateError::ParseError(ParseErrorKind::InvalidTimeValue))
        ); // Invalid second
        assert_eq!(
            ParsiDateTime::parse("1403/05/02 15:30", fmt1),
            Err(DateError::ParseError(ParseErrorKind::FormatMismatch))
        ); // Incomplete time
        assert_eq!(
            ParsiDateTime::parse("1403/05/02 15-30-45", fmt1),
            Err(DateError::ParseError(ParseErrorKind::FormatMismatch))
        ); // Wrong time separator
        assert_eq!(
            ParsiDateTime::parse("1404/12/30 10:00:00", fmt1),
            Err(DateError::ParseError(ParseErrorKind::InvalidDateValue))
        ); // Invalid date part
    }

    // --- Arithmetic Tests ---
    #[test]
    fn test_add_sub_duration() {
        let dt = pdt(1403, 5, 2, 10, 30, 15);

        // Add seconds
        assert_eq!(
            dt.add_duration(Duration::seconds(50)).unwrap(),
            pdt(1403, 5, 2, 10, 31, 5)
        ); // Cross minute
           // Add minutes
        assert_eq!(
            dt.add_duration(Duration::minutes(35)).unwrap(),
            pdt(1403, 5, 2, 11, 5, 15)
        ); // Cross hour
           // Add hours
        assert_eq!(
            dt.add_duration(Duration::hours(14)).unwrap(),
            pdt(1403, 5, 3, 0, 30, 15)
        ); // Cross day

        // Add days (via duration)
        let dt_next_day = dt.add_duration(Duration::days(1)).unwrap();
        assert_eq!(dt_next_day, pdt(1403, 5, 3, 10, 30, 15));

        // Subtract duration
        let dt_prev_sec = dt.sub_duration(Duration::seconds(20)).unwrap();
        assert_eq!(dt_prev_sec, pdt(1403, 5, 2, 10, 29, 55)); // Cross minute backward

        let dt_prev_hour = dt.sub_duration(Duration::hours(11)).unwrap();
        assert_eq!(dt_prev_hour, pdt(1403, 5, 1, 23, 30, 15)); // Cross day backward

        // Test boundary case: end of leap year
        let dt_leap_end = pdt(1403, 12, 30, 23, 59, 58);
        assert_eq!(
            dt_leap_end.add_duration(Duration::seconds(3)).unwrap(),
            pdt(1404, 1, 1, 0, 0, 1)
        );

        // Test boundary case: start of common year
        let dt_common_start = pdt(1404, 1, 1, 0, 0, 1);
        assert_eq!(
            dt_common_start.sub_duration(Duration::seconds(3)).unwrap(),
            pdt(1403, 12, 30, 23, 59, 58)
        );

        // Test Add/Sub trait impl
        assert_eq!(
            (dt + Duration::hours(1)).unwrap(),
            pdt(1403, 5, 2, 11, 30, 15)
        );
        assert_eq!(
            (dt - Duration::days(1)).unwrap(),
            pdt(1403, 5, 1, 10, 30, 15)
        );

        // Test Sub between ParsiDateTime
        let dt2 = pdt(1403, 5, 2, 11, 30, 15);
        assert_eq!((dt2 - dt).unwrap(), Duration::hours(1));
        assert_eq!((dt - dt2).unwrap(), Duration::hours(-1));
    }

    #[test]
    fn test_add_sub_days_months_years_datetime() {
        let dt = pdt(1403, 1, 31, 12, 0, 0); // End of Farvardin

        // Add days (preserves time)
        assert_eq!(dt.add_days(1).unwrap(), pdt(1403, 2, 1, 12, 0, 0));
        // Sub days
        assert_eq!(dt.sub_days(31).unwrap(), pdt(1402, 12, 29, 12, 0, 0)); // 1402 common

        // Add months (clamps day, preserves time)
        assert_eq!(dt.add_months(6).unwrap(), pdt(1403, 7, 30, 12, 0, 0)); // To Mehr (30d), clamped
                                                                           // Sub months
        assert_eq!(dt.sub_months(1).unwrap(), pdt(1402, 12, 29, 12, 0, 0)); // To Esfand (common), clamped

        // Add years (adjusts leap day, preserves time)
        let dt_leap = pdt(1403, 12, 30, 10, 0, 0);
        assert_eq!(dt_leap.add_years(1).unwrap(), pdt(1404, 12, 29, 10, 0, 0)); // Clamp day
                                                                                // Sub years
        assert_eq!(dt_leap.sub_years(4).unwrap(), pdt(1399, 12, 30, 10, 0, 0)); // To leap year

        // Test preservation of time precisely
        let dt_precise = pdt(1400, 6, 15, 1, 2, 3);
        assert_eq!(dt_precise.add_days(10).unwrap().time(), (1, 2, 3));
        assert_eq!(dt_precise.add_months(2).unwrap().time(), (1, 2, 3));
        assert_eq!(dt_precise.add_years(1).unwrap().time(), (1, 2, 3));
    }

    // --- Helper Method Tests ---
    #[test]
    fn test_with_time_components() {
        let dt = pdt(1403, 5, 2, 10, 20, 30);

        assert_eq!(dt.with_hour(11).unwrap(), pdt(1403, 5, 2, 11, 20, 30));
        assert_eq!(dt.with_minute(0).unwrap(), pdt(1403, 5, 2, 10, 0, 30));
        assert_eq!(dt.with_second(59).unwrap(), pdt(1403, 5, 2, 10, 20, 59));
        assert_eq!(dt.with_time(23, 0, 0).unwrap(), pdt(1403, 5, 2, 23, 0, 0));

        // Invalid values
        assert_eq!(dt.with_hour(24), Err(DateError::InvalidTime));
        assert_eq!(dt.with_minute(60), Err(DateError::InvalidTime));
        assert_eq!(dt.with_second(60), Err(DateError::InvalidTime));
        assert_eq!(dt.with_time(10, 60, 0), Err(DateError::InvalidTime));
    }

    #[test]
    fn test_with_date_components_datetime() {
        let dt = pdt(1403, 12, 30, 12, 34, 56); // Leap end

        // with_year clamping
        assert_eq!(dt.with_year(1404).unwrap(), pdt(1404, 12, 29, 12, 34, 56));
        // with_month clamping
        let dt2 = pdt(1403, 1, 31, 1, 2, 3);
        assert_eq!(dt2.with_month(7).unwrap(), pdt(1403, 7, 30, 1, 2, 3));
        // with_day validation
        assert_eq!(dt.with_day(1).unwrap(), pdt(1403, 12, 1, 12, 34, 56));
        assert_eq!(dt.with_day(31), Err(DateError::InvalidDate)); // Esfand never has 31 days
    }

    // --- Serde Tests (conditional on 'serde' feature) ---
    #[cfg(feature = "serde")]
    mod serde_tests_dt {
        use super::*; // Import items from outer scope

        #[test]
        fn test_datetime_serialization_deserialization() {
            let dt = pdt(1403, 5, 2, 10, 20, 30);
            let expected_json =
                r#"{"date":{"year":1403,"month":5,"day":2},"hour":10,"minute":20,"second":30}"#;

            let json = serde_json::to_string(&dt).expect("Serialization failed");
            assert_eq!(json, expected_json);

            let deserialized: ParsiDateTime =
                serde_json::from_str(&json).expect("Deserialization failed");
            assert_eq!(deserialized, dt);
            assert!(deserialized.is_valid());
        }

        #[test]
        fn test_datetime_deserialize_invalid() {
            // Invalid time component
            let json_invalid_time =
                r#"{"date":{"year":1403,"month":5,"day":2},"hour":25,"minute":20,"second":30}"#;
            let deser_invalid_time: ParsiDateTime =
                serde_json::from_str(json_invalid_time).unwrap();
            assert!(!deser_invalid_time.is_valid());
            assert_eq!(deser_invalid_time.hour(), 25); // Field populated directly

            // Invalid date component
            let json_invalid_date =
                r#"{"date":{"year":1404,"month":12,"day":30},"hour":10,"minute":20,"second":30}"#;
            let deser_invalid_date: ParsiDateTime =
                serde_json::from_str(json_invalid_date).unwrap();
            assert!(!deser_invalid_date.is_valid());
            assert_eq!(deser_invalid_date.day(), 30);

            // Structurally invalid (missing field)
            let json_missing_field =
                r#"{"date":{"year":1403,"month":5,"day":2},"hour":10,"minute":20}"#; // Missing second
            assert!(serde_json::from_str::<ParsiDateTime>(json_missing_field).is_err());
        }
    }
} // end mod datetime_tests

// Import necessary items from the library crate root and chrono
use crate::{DateError, ParseErrorKind, ParsiDate, MAX_PARSI_DATE, MIN_PARSI_DATE};
use chrono::NaiveDate;

// Helper function to create a ParsiDate for tests, panicking on failure.
fn pd(year: i32, month: u32, day: u32) -> ParsiDate {
    ParsiDate::new(year, month, day)
        .unwrap_or_else(|e| panic!("Invalid test date {}-{}-{}: {:?}", year, month, day, e))
}

// --- Constructor & Validation Tests ---
#[test]
fn test_new_constructor() {
    assert_eq!(ParsiDate::new(1403, 5, 2), Ok(pd(1403, 5, 2)));
    assert_eq!(ParsiDate::new(1403, 12, 30), Ok(pd(1403, 12, 30))); // Leap year valid end
    assert_eq!(ParsiDate::new(1404, 12, 29), Ok(pd(1404, 12, 29))); // Common year valid end
    assert_eq!(
        ParsiDate::new(1404, 12, 30),
        Err(DateError::InvalidDate),
        "Esfand 30 invalid in common year 1404"
    );
    assert_eq!(
        ParsiDate::new(1403, 13, 1),
        Err(DateError::InvalidDate),
        "Month 13 invalid"
    );
    assert_eq!(
        ParsiDate::new(1403, 0, 1),
        Err(DateError::InvalidDate),
        "Month 0 invalid"
    );
    assert_eq!(
        ParsiDate::new(1403, 1, 0),
        Err(DateError::InvalidDate),
        "Day 0 invalid"
    );
    assert_eq!(
        ParsiDate::new(1403, 7, 31),
        Err(DateError::InvalidDate),
        "Day 31 invalid for Mehr (Month 7)"
    );
    // Test year bounds defined by MIN/MAX constants
    assert_eq!(
        ParsiDate::new(MIN_PARSI_DATE.year() - 1, 1, 1),
        Err(DateError::InvalidDate),
        "Year 0 invalid"
    );
    assert_eq!(
        ParsiDate::new(MAX_PARSI_DATE.year() + 1, 1, 1),
        Err(DateError::InvalidDate),
        "Year 10000 invalid"
    );
    assert!(ParsiDate::new(MIN_PARSI_DATE.year(), 1, 1).is_ok());
    assert!(ParsiDate::new(MAX_PARSI_DATE.year(), 12, 29).is_ok());
}

#[test]
fn test_new_unchecked() {
    // Create a valid date using unsafe constructor
    let d = unsafe { ParsiDate::new_unchecked(1403, 5, 2) };
    assert!(d.is_valid());
    assert_eq!(d.year(), 1403);

    // Create a logically invalid date using unsafe constructor
    let invalid = unsafe { ParsiDate::new_unchecked(1404, 12, 30) }; // Esfand 30 in common year
    assert!(
        !invalid.is_valid(),
        "is_valid correctly identifies invalid date created with new_unchecked"
    );
    // Accessing fields still works, but operations might fail or give wrong results
    assert_eq!(invalid.year(), 1404);
    assert_eq!(invalid.month(), 12);
    assert_eq!(invalid.day(), 30);
}

#[test]
fn test_from_ordinal() {
    // --- Valid cases ---
    assert_eq!(
        ParsiDate::from_ordinal(1403, 1),
        Ok(pd(1403, 1, 1)),
        "Ordinal 1 -> Farvardin 1"
    );
    assert_eq!(
        ParsiDate::from_ordinal(1403, 31),
        Ok(pd(1403, 1, 31)),
        "Ordinal 31 -> Farvardin 31"
    );
    assert_eq!(
        ParsiDate::from_ordinal(1403, 32),
        Ok(pd(1403, 2, 1)),
        "Ordinal 32 -> Ordibehesht 1"
    );
    assert_eq!(
        ParsiDate::from_ordinal(1403, 186),
        Ok(pd(1403, 6, 31)),
        "Ordinal 186 -> Shahrivar 31 (end of first 6 months)"
    );
    assert_eq!(
        ParsiDate::from_ordinal(1403, 187),
        Ok(pd(1403, 7, 1)),
        "Ordinal 187 -> Mehr 1"
    );
    assert_eq!(
        ParsiDate::from_ordinal(1403, 366),
        Ok(pd(1403, 12, 30)),
        "Ordinal 366 -> Last day of leap year 1403"
    );
    assert_eq!(
        ParsiDate::from_ordinal(1404, 365),
        Ok(pd(1404, 12, 29)),
        "Ordinal 365 -> Last day of common year 1404"
    );

    // --- Invalid cases ---
    assert_eq!(
        ParsiDate::from_ordinal(1403, 0),
        Err(DateError::InvalidOrdinal),
        "Ordinal 0 is invalid"
    );
    assert_eq!(
        ParsiDate::from_ordinal(1403, 367),
        Err(DateError::InvalidOrdinal),
        "Ordinal 367 invalid for leap year 1403"
    );
    assert_eq!(
        ParsiDate::from_ordinal(1404, 366),
        Err(DateError::InvalidOrdinal),
        "Ordinal 366 invalid for common year 1404"
    );
    assert_eq!(ParsiDate::from_ordinal(0, 100), Err(DateError::InvalidDate)); // Example check
}

// --- Conversion Tests ---
#[test]
fn test_gregorian_to_persian() {
    // Standard conversion
    assert_eq!(
        ParsiDate::from_gregorian(NaiveDate::from_ymd_opt(2024, 7, 23).unwrap()),
        Ok(pd(1403, 5, 2))
    );
    // Nowruz (Persian New Year)
    assert_eq!(
        ParsiDate::from_gregorian(NaiveDate::from_ymd_opt(2024, 3, 20).unwrap()),
        Ok(pd(1403, 1, 1)),
        "Nowruz 1403"
    );
    assert_eq!(
        ParsiDate::from_gregorian(NaiveDate::from_ymd_opt(2025, 3, 21).unwrap()),
        Ok(pd(1404, 1, 1)),
        "Nowruz 1404"
    );
    // Day before Nowruz
    assert_eq!(
        ParsiDate::from_gregorian(NaiveDate::from_ymd_opt(2024, 3, 19).unwrap()),
        Ok(pd(1402, 12, 29)), // 1402 was common year
        "Day before Nowruz 1403"
    );
    // Specific historical date
    assert_eq!(
        ParsiDate::from_gregorian(NaiveDate::from_ymd_opt(1979, 2, 11).unwrap()),
        Ok(pd(1357, 11, 22))
    );
    // Epoch start
    assert_eq!(
        ParsiDate::from_gregorian(NaiveDate::from_ymd_opt(622, 3, 21).unwrap()),
        Ok(pd(1, 1, 1)),
        "Persian epoch start"
    );
    // Before epoch
    assert_eq!(
        ParsiDate::from_gregorian(NaiveDate::from_ymd_opt(622, 3, 20).unwrap()),
        Err(DateError::GregorianConversionError),
        "Date before Persian epoch"
    );
    // Test around year boundary (end of a leap year 1403)
    assert_eq!(
        ParsiDate::from_gregorian(NaiveDate::from_ymd_opt(2025, 3, 20).unwrap()),
        Ok(pd(1403, 12, 30)),
        "Last day of Persian leap year 1403"
    );
    // Test around year boundary (end of a common year 1404)
    assert_eq!(
        ParsiDate::from_gregorian(NaiveDate::from_ymd_opt(2026, 3, 20).unwrap()),
        Ok(pd(1404, 12, 29)),
        "Last day of Persian common year 1404"
    );
    // Test a date far in the future
    assert_eq!(
        ParsiDate::from_gregorian(NaiveDate::from_ymd_opt(2622, 3, 21).unwrap()),
        Ok(pd(2001, 1, 1)), // Example calculation, needs verification if precise relation is needed
        "Future date conversion"
    );
}

#[test]
fn test_persian_to_gregorian() {
    // Standard conversion
    assert_eq!(
        pd(1403, 5, 2).to_gregorian(),
        Ok(NaiveDate::from_ymd_opt(2024, 7, 23).unwrap())
    );
    // Nowruz
    assert_eq!(
        pd(1403, 1, 1).to_gregorian(),
        Ok(NaiveDate::from_ymd_opt(2024, 3, 20).unwrap())
    );
    assert_eq!(
        pd(1404, 1, 1).to_gregorian(),
        Ok(NaiveDate::from_ymd_opt(2025, 3, 21).unwrap())
    );
    // Last day of leap year
    assert_eq!(
        pd(1403, 12, 30).to_gregorian(),
        Ok(NaiveDate::from_ymd_opt(2025, 3, 20).unwrap())
    );
    // Last day of common year
    assert_eq!(
        pd(1404, 12, 29).to_gregorian(),
        Ok(NaiveDate::from_ymd_opt(2026, 3, 20).unwrap())
    );
    // Specific historical date
    assert_eq!(
        pd(1357, 11, 22).to_gregorian(),
        Ok(NaiveDate::from_ymd_opt(1979, 2, 11).unwrap())
    );
    // Epoch start
    assert_eq!(
        pd(1, 1, 1).to_gregorian(),
        Ok(NaiveDate::from_ymd_opt(622, 3, 21).unwrap())
    );
    // Test invalid date conversion attempt (created via unsafe)
    let invalid_date = unsafe { ParsiDate::new_unchecked(1404, 12, 30) };
    assert!(!invalid_date.is_valid());
    // `to_gregorian` performs validation first.
    assert_eq!(invalid_date.to_gregorian(), Err(DateError::InvalidDate));
}

#[test]
fn test_today_function() {
    // This test checks if `today()` runs successfully and returns a logically valid date
    // within the expected Persian year range based on the system clock at runtime.
    match ParsiDate::today() {
        Ok(today) => {
            // Print for info during test runs.
            println!(
                "Today's Persian date (captured by test): {}",
                today.format("long")
            );
            // Check if the returned date is valid according to library rules.
            assert!(
                today.is_valid(),
                "ParsiDate::today() returned an invalid date object: y={}, m={}, d={}",
                today.year(),
                today.month(),
                today.day()
            );
            // Check if the year falls within the globally supported range.
            assert!(
                today.year() >= MIN_PARSI_DATE.year() && today.year() <= MAX_PARSI_DATE.year(),
                "Today's Persian year {} is outside the supported range [{}, {}]",
                today.year(),
                MIN_PARSI_DATE.year(),
                MAX_PARSI_DATE.year()
            );
        }
        Err(e) => {
            // This should only fail if the system clock is drastically wrong, leading to
            // a Gregorian date outside chrono's or this library's conversion range.
            panic!("ParsiDate::today() failed unexpectedly: {}", e);
        }
    }
}

// --- Leap Year & DaysInMonth Tests ---
#[test]
fn test_leap_years() {
    // Test cases based on the 33-year cycle rule: year % 33 in {1, 5, 9, 13, 17, 22, 26, 30}
    assert!(
        ParsiDate::is_persian_leap_year(1399),
        "1399 % 33 = 13 -> leap"
    );
    assert!(
        ParsiDate::is_persian_leap_year(1403),
        "1403 % 33 = 17 -> leap"
    );
    assert!(
        !ParsiDate::is_persian_leap_year(1404),
        "1404 % 33 = 18 -> common"
    );
    assert!(
        !ParsiDate::is_persian_leap_year(1407),
        "1407 % 33 = 21 -> common"
    );
    assert!(
        ParsiDate::is_persian_leap_year(1408),
        "1408 % 33 = 22 -> leap"
    );
    assert!(
        ParsiDate::is_persian_leap_year(1420),
        "1420 % 33 = 22 -> leap" // This is a mistake in comment. 1420%33=1. It should be leap.
    );
    assert!(
        ParsiDate::is_persian_leap_year(1424),
        "1424 % 33 = 26 -> leap"
    );
    assert!(
        ParsiDate::is_persian_leap_year(1428),
        "1428 % 33 = 30 -> leap"
    );
    assert!(
        ParsiDate::is_persian_leap_year(1432),
        "1432 % 33 = 1 -> leap"
    ); // Cycle restart
    assert!(
        !ParsiDate::is_persian_leap_year(1400),
        "1400 % 33 = 14 -> common"
    );
    assert!(
        !ParsiDate::is_persian_leap_year(9999),
        "9999 % 33 = 3 -> common (MAX_PARSI_DATE year)"
    );
    // Invalid years should return false
    assert!(!ParsiDate::is_persian_leap_year(0), "Year 0 is not leap");
    assert!(
        !ParsiDate::is_persian_leap_year(-10),
        "Negative year is not leap"
    );
}

#[test]
fn test_days_in_month() {
    // Months 1-6 always have 31 days
    assert_eq!(ParsiDate::days_in_month(1403, 1), 31, "Farvardin");
    assert_eq!(ParsiDate::days_in_month(1404, 6), 31, "Shahrivar");
    // Months 7-11 always have 30 days
    assert_eq!(ParsiDate::days_in_month(1403, 7), 30, "Mehr");
    assert_eq!(ParsiDate::days_in_month(1404, 11), 30, "Bahman");
    // Month 12 (Esfand) depends on leap year
    assert_eq!(
        ParsiDate::days_in_month(1403, 12),
        30,
        "Esfand in leap year 1403"
    );
    assert_eq!(
        ParsiDate::days_in_month(1404, 12),
        29,
        "Esfand in common year 1404"
    );
    assert_eq!(
        ParsiDate::days_in_month(1408, 12),
        30,
        "Esfand in Leap year 1408"
    );

    // Test invalid month numbers should return 0
    assert_eq!(ParsiDate::days_in_month(1403, 0), 0, "Invalid month 0");
    assert_eq!(ParsiDate::days_in_month(1403, 13), 0, "Invalid month 13");
}

// --- Formatting Tests ---
#[test]
fn test_format_predefined() {
    let date = pd(1403, 5, 2);
    assert_eq!(date.format("short"), "1403/05/02");
    assert_eq!(date.format("long"), "2 مرداد 1403"); // Day not padded in "long"
    assert_eq!(date.format("iso"), "1403-05-02");
    // Test Display trait (should default to "short")
    assert_eq!(date.to_string(), "1403/05/02");

    // Test with single digit month/day to ensure padding in short/iso
    let date_single_digit = pd(1400, 1, 9);
    assert_eq!(date_single_digit.format("short"), "1400/01/09");
    assert_eq!(date_single_digit.format("long"), "9 فروردین 1400");
    assert_eq!(date_single_digit.format("iso"), "1400-01-09");
    assert_eq!(date_single_digit.to_string(), "1400/01/09");
}

#[test]
fn test_format_strftime() {
    let date = pd(1403, 1, 7); // 1403-01-07 is a Tue/سه‌شنبه (Gregorian: 2024-03-26)
    let date_common_end = pd(1404, 12, 29); // 1404-12-29 is a Fri/جمعه (Gregorian: 2026-03-20)
    let date_leap_end = pd(1403, 12, 30); // 1403-12-30 is a Thu/پنجشنبه (Gregorian: 2025-03-20)
    let date_sat = pd(1403, 1, 4); // 1403-01-04 is a Sat/شنبه (Gregorian: 2024-03-23)
    let date_sun = pd(1403, 1, 5); // 1403-01-05 is a Sun/یکشنبه (Gregorian: 2024-03-24)

    // Basic specifiers (%Y, %m, %d, %B)
    assert_eq!(date.format("%Y/%m/%d"), "1403/01/07");
    assert_eq!(date.format("%d %B %Y"), "07 فروردین 1403"); // %d pads day
    assert_eq!(date_common_end.format("%Y/%m/%d"), "1404/12/29");
    assert_eq!(date_common_end.format("%d %B %Y"), "29 اسفند 1404");

    // Ordinal day (%j) - 3 digits zero-padded
    assert_eq!(date.format("Day %j of %Y"), "Day 007 of 1403");
    assert_eq!(
        date_common_end.format("Day %j"),
        "Day 365",
        "Last day of common year"
    );
    assert_eq!(date_leap_end.format("%j"), "366", "Last day of leap year");

    // Weekday (%A name, %w number Sat=0)
    assert_eq!(
        date_common_end.format("Weekday %A (num %w)"),
        "Weekday جمعه (num 6)"
    ); // Friday
    assert_eq!(date.format("%A"), "سه‌شنبه"); // Tuesday
    assert_eq!(date_sat.format("%A (%w)"), "شنبه (0)"); // Saturday
    assert_eq!(date_sun.format("%A (%w)"), "یکشنبه (1)"); // Sunday

    // Literal percent sign (%%)
    assert_eq!(date.format("%% %Y %%"), "% 1403 %");

    // Combined and complex patterns
    assert_eq!(date.format("%d-%B-%Y (%A)"), "07-فروردین-1403 (سه‌شنبه)");

    // Unknown specifier (%x) should be output literally
    assert_eq!(date.format("%Y-%m-%d %x %!"), "1403-01-07 %x %!");

    // Test formatting of potentially invalid date (via unsafe)
    let invalid_date = unsafe { ParsiDate::new_unchecked(1400, 13, 1) }; // Invalid month 13
                                                                         // Behavior here depends on implementation; robust formatting handles invalid components gracefully.
    assert!(
        invalid_date.format("%Y/%m/%d").contains("1400/13/01"),
        "Display might show raw invalid data"
    );
    assert!(
        invalid_date.format("%B").contains("?InvalidMonth?"),
        "Formatting %B for invalid month should indicate error"
    );
    // Weekday/Ordinal calculation on invalid date should indicate error
    assert!(
        invalid_date.format("%A").contains("?WeekdayError?"),
        "Formatting %A for invalid date should indicate error"
    );
    assert!(
        invalid_date.format("%j").contains("???"),
        "Formatting %j for invalid date should indicate error"
    );
}

// --- Parsing Tests ---
#[test]
fn test_parse_simple() {
    // Basic YMD formats with different separators
    assert_eq!(
        ParsiDate::parse("1403/05/02", "%Y/%m/%d"),
        Ok(pd(1403, 5, 2))
    );
    assert_eq!(
        ParsiDate::parse("1403-01-31", "%Y-%m-%d"),
        Ok(pd(1403, 1, 31))
    );
    // Different order of components
    assert_eq!(
        ParsiDate::parse("07/04/1399", "%d/%m/%Y"),
        Ok(pd(1399, 4, 7))
    );
    // Test parsing epoch start and max supported date
    assert_eq!(ParsiDate::parse("0001/01/01", "%Y/%m/%d"), Ok(pd(1, 1, 1)));
    assert_eq!(
        ParsiDate::parse("9999/12/29", "%Y/%m/%d"),
        Ok(pd(9999, 12, 29)),
        "Max date (9999 is common)"
    );
}

#[test]
fn test_parse_month_name() {
    // %d requires padded day (2 digits)
    assert_eq!(
        ParsiDate::parse("02 مرداد 1403", "%d %B %Y"),
        Ok(pd(1403, 5, 2))
    );
    // End of leap year with month name
    assert_eq!(
        ParsiDate::parse("30 اسفند 1399", "%d %B %Y"),
        Ok(pd(1399, 12, 30)), // 1399 is leap
        "End of leap year with %B"
    );
    // End of common year with month name
    assert_eq!(
        ParsiDate::parse("29 اسفند 1404", "%d %B %Y"),
        Ok(pd(1404, 12, 29)), // 1404 is common
        "End of common year with %B"
    );
    // Test with exact single spaces as required by the current parser implementation
    assert_eq!(
        ParsiDate::parse("10 دی 1400", "%d %B %Y"),
        Ok(pd(1400, 10, 10))
    );
    // Test month name at different positions in format string
    assert_eq!(
        ParsiDate::parse("1400-دی-10", "%Y-%B-%d"),
        Ok(pd(1400, 10, 10))
    );
    assert_eq!(
        ParsiDate::parse("فروردین-01-1390", "%B-%d-%Y"),
        Ok(pd(1390, 1, 1))
    );
    // Test month name followed immediately by year
    assert_eq!(
        ParsiDate::parse("01اردیبهشت1395", "%d%B%Y"),
        Ok(pd(1395, 2, 1))
    );
}

#[test]
fn test_parse_errors() {
    // --- Invalid Number Errors ---
    // %m and %d require exactly two digits
    assert_eq!(
        ParsiDate::parse("1403/5/02", "%Y/%m/%d").unwrap_err(),
        DateError::ParseError(ParseErrorKind::InvalidNumber),
        "Single digit month for %m"
    );
    assert_eq!(
        ParsiDate::parse("1403/05/2", "%Y/%m/%d").unwrap_err(),
        DateError::ParseError(ParseErrorKind::InvalidNumber),
        "Single digit day for %d"
    );
    // %Y requires exactly four digits
    assert_eq!(
        ParsiDate::parse("403/01/01", "%Y/%m/%d").unwrap_err(),
        DateError::ParseError(ParseErrorKind::InvalidNumber),
        "Three digit year for %Y"
    );
    // Non-digit characters where digits are expected
    assert_eq!(
        ParsiDate::parse("1403/XX/01", "%Y/%m/%d").unwrap_err(),
        DateError::ParseError(ParseErrorKind::InvalidNumber),
        "Non-digit month"
    );
    assert_eq!(
        ParsiDate::parse("ABCD/01/01", "%Y/%m/%d").unwrap_err(),
        DateError::ParseError(ParseErrorKind::InvalidNumber),
        "Non-digit year"
    );

    // --- Format Mismatch Errors ---
    // Missing separators
    assert_eq!(
        ParsiDate::parse("14030502", "%Y/%m/%d").unwrap_err(),
        DateError::ParseError(ParseErrorKind::FormatMismatch), // Expected '/', got '0'
        "Missing separators"
    );
    // Wrong separator
    assert_eq!(
        ParsiDate::parse("1403 05 02", "%Y/%m/%d").unwrap_err(),
        DateError::ParseError(ParseErrorKind::FormatMismatch), // Expected '/', got ' '
        "Wrong separator (space vs /)"
    );
    // Trailing text not in format
    assert_eq!(
        ParsiDate::parse("1403/01/01extra", "%Y/%m/%d").unwrap_err(),
        DateError::ParseError(ParseErrorKind::FormatMismatch),
        "Trailing text"
    );
    // Incomplete input for format
    assert_eq!(
        ParsiDate::parse("1403/05", "%Y/%m/%d").unwrap_err(), // Input ends before matching %d
        DateError::ParseError(ParseErrorKind::FormatMismatch),
        "Incomplete input"
    );
    // Mismatch with literal format chars
    assert_eq!(
        ParsiDate::parse("Year: 1403", "Date: %Y").unwrap_err(),
        DateError::ParseError(ParseErrorKind::FormatMismatch),
        "Literal prefix mismatch"
    );

    // --- Invalid Date Value Errors (parsed components are invalid logically) ---
    assert_eq!(
        ParsiDate::parse("1403/13/01", "%Y/%m/%d").unwrap_err(), // Month > 12
        DateError::ParseError(ParseErrorKind::InvalidDateValue),
        "Invalid month value > 12"
    );
    assert_eq!(
        ParsiDate::parse("1403/00/01", "%Y/%m/%d").unwrap_err(), // Month 0
        DateError::ParseError(ParseErrorKind::InvalidDateValue),
        "Invalid month value 0"
    );
    assert_eq!(
        ParsiDate::parse("1404/12/30", "%Y/%m/%d").unwrap_err(), // Day 30 invalid for Esfand in common year 1404
        DateError::ParseError(ParseErrorKind::InvalidDateValue),
        "Invalid day (Esfand 30 common year)"
    );
    assert_eq!(
        ParsiDate::parse("1403/07/31", "%Y/%m/%d").unwrap_err(), // Day 31 invalid for Mehr (Month 7)
        DateError::ParseError(ParseErrorKind::InvalidDateValue),
        "Invalid day (Mehr 31)"
    );
    assert_eq!(
        ParsiDate::parse("1403/01/00", "%Y/%m/%d").unwrap_err(), // Day 0
        DateError::ParseError(ParseErrorKind::InvalidDateValue),
        "Invalid day value 0"
    );
    assert_eq!(
        ParsiDate::parse("0000/01/01", "%Y/%m/%d").unwrap_err(), // Year 0
        DateError::ParseError(ParseErrorKind::InvalidDateValue), // Assuming ParsiDate::new(0, ..) fails
        "Invalid year value 0"
    );

    // --- Invalid Month Name Errors (%B) ---
    assert_eq!(
        ParsiDate::parse("02 Mordad 1403", "%d %B %Y").unwrap_err(), // Non-Persian name
        DateError::ParseError(ParseErrorKind::InvalidMonthName),
        "Non-Persian month name"
    );
    assert_eq!(
        ParsiDate::parse("01 XXX 1400", "%d %B %Y").unwrap_err(), // Completely wrong name
        DateError::ParseError(ParseErrorKind::InvalidMonthName),
        "Unrecognized month name"
    );
    assert_eq!(
        ParsiDate::parse("01 فروردین-1400", "%d %B %Y").unwrap_err(), // Expected space after name, got '-'
        DateError::ParseError(ParseErrorKind::FormatMismatch), // Fails matching the literal space in format
        "Wrong separator after month name"
    );

    // --- Unsupported Specifier Error ---
    assert_eq!(
        ParsiDate::parse("Some text", "%j").unwrap_err(), // %j not supported for parsing
        DateError::ParseError(ParseErrorKind::UnsupportedSpecifier),
        "Unsupported specifier %j for parse"
    );
    assert_eq!(
        ParsiDate::parse("Some text", "%A").unwrap_err(), // %A not supported for parsing
        DateError::ParseError(ParseErrorKind::UnsupportedSpecifier),
        "Unsupported specifier %A for parse"
    );
}

// --- Date Info Tests ---
#[test]
fn test_weekday() {
    // Gregorian: Wed 2024-03-20 -> Persian: Chaharshanbe 1403-01-01 -> Weekday 3
    assert_eq!(
        pd(1403, 1, 1).weekday(),
        Ok("چهارشنبه".to_string()),
        "1403-01-01 -> Wed"
    );
    // Gregorian: Tue 2024-07-23 -> Persian: Seshanbe 1403-05-02 -> Weekday 2
    assert_eq!(
        pd(1403, 5, 2).weekday(),
        Ok("سه‌شنبه".to_string()),
        "1403-05-02 -> Tue"
    );
    // Gregorian: Fri 2025-03-21 -> Persian: Jomeh 1404-01-01 -> Weekday 5
    assert_eq!(
        pd(1404, 1, 1).weekday(),
        Ok("جمعه".to_string()),
        "1404-01-01 -> Fri"
    );
    // Gregorian: Sun 1979-02-11 -> Persian: Yekshanbe 1357-11-22 -> Weekday 0
    assert_eq!(
        pd(1357, 11, 22).weekday(),
        Ok("یکشنبه".to_string()),
        "1357-11-22 -> Sun"
    );
    // Gregorian: Fri 2026-03-20 -> Persian: Jomeh 1404-12-29 -> Weekday 5
    assert_eq!(
        pd(1404, 12, 29).weekday(),
        Ok("جمعه".to_string()),
        "1404-12-29 -> Fri"
    );
    // Gregorian: Sat 2024-03-23 -> Persian: Shanbe 1403-01-04 -> Weekday 6
    assert_eq!(
        pd(1403, 1, 4).weekday(),
        Ok("شنبه".to_string()),
        "1403-01-04 -> Sat"
    );
    // Test on invalid date (created via unsafe)
    let invalid_date = unsafe { ParsiDate::new_unchecked(1404, 12, 30) };
    assert_eq!(invalid_date.weekday(), Err(DateError::InvalidDate)); // Should fail validation first
}

#[test]
fn test_ordinal() {
    assert_eq!(pd(1403, 1, 1).ordinal(), Ok(1)); // First day of year
    assert_eq!(pd(1403, 1, 31).ordinal(), Ok(31));
    assert_eq!(pd(1403, 2, 1).ordinal(), Ok(32)); // 31 (Far) + 1
    assert_eq!(pd(1403, 5, 2).ordinal(), Ok(126)); // 4*31 (Far-Tir) + 2 = 124 + 2 = 126
    assert_eq!(pd(1403, 7, 1).ordinal(), Ok(187)); // 6*31 (Far-Sha) + 1 = 186 + 1 = 187
    assert_eq!(pd(1403, 12, 30).ordinal(), Ok(366)); // Last day of leap year 1403
    assert_eq!(pd(1404, 1, 1).ordinal(), Ok(1)); // First day of common year 1404
    assert_eq!(pd(1404, 12, 29).ordinal(), Ok(365)); // Last day of common year 1404
                                                     // Test on invalid date
    let invalid_date = unsafe { ParsiDate::new_unchecked(1404, 12, 30) };
    assert_eq!(invalid_date.ordinal(), Err(DateError::InvalidDate)); // Fails validation
}

// --- Arithmetic Tests ---
#[test]
fn test_add_sub_days() {
    let d_mid_month = pd(1403, 6, 30); // End of 31-day month
    assert_eq!(d_mid_month.add_days(1), Ok(pd(1403, 6, 31)));
    assert_eq!(d_mid_month.add_days(2), Ok(pd(1403, 7, 1))); // Cross to 30-day month
    assert_eq!(d_mid_month.add_days(32), Ok(pd(1403, 8, 1))); // Cross Shahrivar (1d) + Mehr (30d) = 31d -> Aban 1st

    let d_leap_end = pd(1403, 12, 29); // Near end of leap year
    assert_eq!(d_leap_end.add_days(1), Ok(pd(1403, 12, 30))); // To last day
    assert_eq!(d_leap_end.add_days(2), Ok(pd(1404, 1, 1))); // Cross to common year

    let d_common_end = pd(1404, 12, 29); // Last day of common year
    assert_eq!(d_common_end.add_days(1), Ok(pd(1405, 1, 1))); // Cross to common year (1405 is common)

    // Subtraction
    let d_start_common = pd(1404, 1, 1); // Start of common year
    assert_eq!(d_start_common.add_days(-1), Ok(pd(1403, 12, 30))); // Subtract 1 day -> end of leap year
    assert_eq!(d_start_common.sub_days(1), Ok(pd(1403, 12, 30))); // Using sub_days

    let d_start_common2 = pd(1405, 1, 1); // Start of common year
    assert_eq!(d_start_common2.sub_days(1), Ok(pd(1404, 12, 29))); // Subtract 1 day -> end of common year

    // Add/subtract zero
    assert_eq!(d_mid_month.add_days(0), Ok(d_mid_month));
    assert_eq!(d_mid_month.sub_days(0), Ok(d_mid_month));

    // Add/subtract large number of days
    let base = pd(1400, 1, 1);
    let expected_greg_plus_1000 = NaiveDate::from_ymd_opt(2021, 3, 21)
        .unwrap()
        .checked_add_days(chrono::Days::new(1000))
        .unwrap();
    let expected_parsi_plus_1000 = ParsiDate::from_gregorian(expected_greg_plus_1000).unwrap();
    assert_eq!(base.add_days(1000), Ok(expected_parsi_plus_1000));
    assert_eq!(expected_parsi_plus_1000.sub_days(1000), Ok(base));
    assert_eq!(expected_parsi_plus_1000.add_days(-1000), Ok(base));

    // Test potential overflow
    let large_days = i64::MAX / 10;
    assert!(base.add_days(large_days).is_err());
    assert!(base.add_days(-large_days).is_err());

    // Test arithmetic on invalid date
    let invalid_date = unsafe { ParsiDate::new_unchecked(1404, 12, 30) };
    assert_eq!(invalid_date.add_days(1), Err(DateError::InvalidDate));
    assert_eq!(invalid_date.sub_days(1), Err(DateError::InvalidDate));
}

#[test]
fn test_add_sub_months() {
    let d_31 = pd(1403, 1, 31); // End of 31-day month (Farvardin, leap year)
    assert_eq!(
        d_31.add_months(1),
        Ok(pd(1403, 2, 31)),
        "To Ordibehesht (31d)"
    );
    assert_eq!(
        d_31.add_months(5),
        Ok(pd(1403, 6, 31)),
        "To Shahrivar (31d)"
    );
    assert_eq!(
        d_31.add_months(6),
        Ok(pd(1403, 7, 30)),
        "To Mehr (30d), clamped"
    );
    assert_eq!(
        d_31.add_months(11),
        Ok(pd(1403, 12, 30)),
        "To Esfand (30d, leap), clamped"
    );

    let d_31_common = pd(1404, 1, 31);
    assert_eq!(
        d_31_common.add_months(11),
        Ok(pd(1404, 12, 29)),
        "To Esfand (29d, common), clamped"
    );

    let d_mid = pd(1403, 5, 15);
    assert_eq!(d_mid.add_months(1), Ok(pd(1403, 6, 15)));
    assert_eq!(
        d_mid.add_months(7),
        Ok(pd(1403, 12, 15)),
        "To Esfand (leap)"
    );
    assert_eq!(d_mid.add_months(12), Ok(pd(1404, 5, 15)), "Add full year");

    // Subtraction
    assert_eq!(
        d_mid.sub_months(5),
        Ok(pd(1402, 12, 15)),
        "Subtract 5 months -> Esfand 1402 (common)"
    );
    assert_eq!(
        d_31.sub_months(1),
        Ok(pd(1402, 12, 29)),
        "1403-01-31 minus 1m -> Esfand 1402 (common), clamped"
    );

    // Test arithmetic on invalid date
    let invalid_date = unsafe { ParsiDate::new_unchecked(1404, 12, 30) };
    assert_eq!(invalid_date.add_months(1), Err(DateError::InvalidDate));
}

#[test]
fn test_add_sub_years() {
    let d1 = pd(1403, 5, 2); // Leap year
    assert_eq!(d1.add_years(1), Ok(pd(1404, 5, 2)), "Leap -> Common");
    assert_eq!(d1.add_years(-1), Ok(pd(1402, 5, 2)), "Leap -> Common");

    // Test leap day adjustment
    let d_leap_end = pd(1403, 12, 30);
    assert_eq!(
        d_leap_end.add_years(1),
        Ok(pd(1404, 12, 29)),
        "Leap day + 1y -> Common year, clamped"
    );
    assert_eq!(
        d_leap_end.sub_years(4),
        Ok(pd(1399, 12, 30)),
        "Leap day - 4y -> Leap year 1399"
    );

    // Test arithmetic on invalid date
    let invalid_date = unsafe { ParsiDate::new_unchecked(1404, 12, 30) };
    assert_eq!(invalid_date.add_years(1), Err(DateError::InvalidDate));
}

#[test]
fn test_days_between() {
    let d1 = pd(1403, 1, 1);
    let d2 = pd(1403, 1, 11);
    let d3 = pd(1404, 1, 1); // 1403 is leap, so 366 days
    let d4 = pd(1402, 12, 29); // Day before d1

    assert_eq!(d1.days_between(&d2), Ok(10));
    assert_eq!(d1.days_between(&d3), Ok(366));
    assert_eq!(d1.days_between(&d4), Ok(1));

    // Test with invalid dates
    let invalid_date = unsafe { ParsiDate::new_unchecked(1404, 12, 30) };
    assert_eq!(d1.days_between(&invalid_date), Err(DateError::InvalidDate));
}

// --- Helper Method Tests ---
#[test]
fn test_with_year() {
    let d_mid_leap = pd(1403, 5, 2);
    let d_leap_end = pd(1403, 12, 30);

    assert_eq!(d_mid_leap.with_year(1404), Ok(pd(1404, 5, 2)));
    assert_eq!(d_leap_end.with_year(1404), Ok(pd(1404, 12, 29)));

    assert_eq!(d_mid_leap.with_year(0), Err(DateError::InvalidDate));
}

#[test]
fn test_with_month() {
    let d_31 = pd(1403, 1, 31);

    assert_eq!(
        d_31.with_month(7),
        Ok(pd(1403, 7, 30)),
        "To Mehr (30d), clamped"
    );
    assert_eq!(
        pd(1404, 1, 31).with_month(12),
        Ok(pd(1404, 12, 29)),
        "To Esfand (29d, common), clamped"
    );

    assert_eq!(d_31.with_month(13), Err(DateError::InvalidDate));
}

#[test]
fn test_with_day() {
    let d_mehr = pd(1403, 7, 1);

    assert_eq!(d_mehr.with_day(30), Ok(pd(1403, 7, 30)));
    assert_eq!(
        d_mehr.with_day(31),
        Err(DateError::InvalidDate),
        "Invalid day 31 for Mehr"
    );

    assert_eq!(d_mehr.with_day(0), Err(DateError::InvalidDate));
}

#[test]
fn test_day_of_boundaries() {
    let d_mid_leap = pd(1403, 5, 15);
    assert_eq!(d_mid_leap.first_day_of_month(), pd(1403, 5, 1));
    assert_eq!(d_mid_leap.last_day_of_month(), pd(1403, 5, 31));
    assert_eq!(d_mid_leap.first_day_of_year(), pd(1403, 1, 1));
    assert_eq!(
        d_mid_leap.last_day_of_year(),
        pd(1403, 12, 30),
        "Last day of leap year 1403"
    );

    let d_mid_common = pd(1404, 7, 10);
    assert_eq!(d_mid_common.last_day_of_year(), pd(1404, 12, 29));
}

// --- Constant Tests ---
#[test]
fn test_constants_validity_and_values() {
    assert!(MIN_PARSI_DATE.is_valid());
    assert_eq!(MIN_PARSI_DATE.year(), 1);

    assert!(MAX_PARSI_DATE.is_valid());
    assert_eq!(MAX_PARSI_DATE.year(), 9999);
    assert_eq!(MAX_PARSI_DATE.day(), 29);
    assert!(!ParsiDate::is_persian_leap_year(9999));
}

// --- Serde Tests (conditional on 'serde' feature) ---
#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;

    #[test]
    fn test_serialization_deserialization_valid() {
        let date = pd(1403, 5, 2);
        let expected_json = r#"{"year":1403,"month":5,"day":2}"#;

        let json = serde_json::to_string(&date).expect("Serialization failed");
        assert_eq!(json, expected_json);

        let deserialized: ParsiDate = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized, date);
        assert!(deserialized.is_valid());
    }

    #[test]
    fn test_deserialize_structurally_valid_but_logically_invalid() {
        let json_invalid_day = r#"{"year":1404,"month":12,"day":30}"#;
        let deserialized_invalid: ParsiDate =
            serde_json::from_str(json_invalid_day).expect("Deserialization should succeed");
        assert!(!deserialized_invalid.is_valid());
    }

    #[test]
    fn test_deserialize_structurally_invalid() {
        let json_missing_field = r#"{"year":1403,"month":5}"#;
        assert!(serde_json::from_str::<ParsiDate>(json_missing_field).is_err());
    }
}

#[cfg(test)]
mod season_tests {
    use crate::season::Season;
    use crate::{DateError, ParsiDate, ParsiDateTime};

    fn pd(y: i32, m: u32, d: u32) -> ParsiDate {
        ParsiDate::new(y, m, d).unwrap()
    }
    fn pdt(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> ParsiDateTime {
        ParsiDateTime::new(y, m, d, h, min, s).unwrap()
    }

    #[test]
    fn test_season_enum_methods() {
        assert_eq!(Season::Bahar.name_persian(), "بهار");
        assert_eq!(Season::Tabestan.name_english(), "Summer");
        assert_eq!(Season::Paeez.start_month(), 7);
        assert_eq!(Season::Zemestan.end_month(), 12);
        assert_eq!(format!("{}", Season::Paeez), "پاییز");
    }

    #[test]
    fn test_parsidate_season() {
        assert_eq!(pd(1403, 1, 1).season(), Ok(Season::Bahar));
        assert_eq!(pd(1403, 4, 1).season(), Ok(Season::Tabestan));
        assert_eq!(pd(1403, 7, 1).season(), Ok(Season::Paeez));
        assert_eq!(pd(1403, 10, 1).season(), Ok(Season::Zemestan));

        let invalid_date = unsafe { ParsiDate::new_unchecked(1403, 13, 1) };
        assert_eq!(invalid_date.season(), Err(DateError::InvalidDate));
    }

    #[test]
    fn test_parsidatetime_season() {
        assert_eq!(pdt(1403, 2, 10, 12, 0, 0).season(), Ok(Season::Bahar));
        assert_eq!(pdt(1403, 8, 1, 0, 0, 0).season(), Ok(Season::Paeez));
    }

    #[test]
    fn test_format_season() {
        let dt_summer = pdt(1403, 5, 2, 8, 5, 30);
        assert_eq!(dt_summer.format("%K"), "تابستان");
    }

    #[test]
    fn test_season_boundaries() {
        let d_spring = pd(1403, 2, 15);
        assert_eq!(d_spring.start_of_season(), Ok(pd(1403, 1, 1)));
        assert_eq!(d_spring.end_of_season(), Ok(pd(1403, 3, 31)));

        let d_winter_leap = pd(1403, 10, 5);
        assert_eq!(d_winter_leap.end_of_season(), Ok(pd(1403, 12, 30)));

        let d_winter_common = pd(1404, 11, 10);
        assert_eq!(d_winter_common.end_of_season(), Ok(pd(1404, 12, 29)));
    }
}

// This module is only compiled when the 'timezone' feature is enabled.
#[cfg(all(test, feature = "timezone"))]
mod zoned_datetime_tests {
    use crate::{DateError, ParsiDate, ZonedParsiDateTime};
    use chrono::{Duration, Offset};
    use chrono_tz::{America::New_York, Asia::Tehran, Europe::London, Tz};

    // Helper function for creating a ZonedParsiDateTime, panicking on failure.
    fn z_pdt(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32, tz: Tz) -> ZonedParsiDateTime<Tz> {
        ZonedParsiDateTime::new(y, m, d, h, min, s, tz)
            .unwrap_or_else(|e| panic!("Failed to create zoned datetime: {:?}", e))
    }

    #[test]
    fn test_zoned_new_and_accessors() {
        let dt = z_pdt(1403, 5, 2, 15, 30, 0, Tehran);

        assert_eq!(dt.year(), 1403);
        assert_eq!(dt.month(), 5);
        assert_eq!(dt.day(), 2);
        assert_eq!(dt.hour(), 15);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 0);
        assert_eq!(dt.timezone(), Tehran);
    }

    #[test]
    fn test_zoned_now() {
        let now_tehran = ZonedParsiDateTime::now(Tehran);
        let now_london = ZonedParsiDateTime::now(London);

        // Both should represent the same instant in time.
        // Their difference should be very close to zero.
        let diff = now_tehran.clone() - now_london.clone();
        assert!(diff.num_seconds().abs() <= 1);

        println!("Now in Tehran: {}", now_tehran);
        println!("Now in London: {}", now_london);
    }

    #[test]
    fn test_with_timezone() {
        // Tehran is UTC+3:30 in winter (when Iran does not observe DST).
        let tehran_time = z_pdt(1402, 10, 10, 12, 0, 0, Tehran); // Dey 10th, noon.

        // London is UTC+0 in winter.
        let london_time = tehran_time.with_timezone(&London);

        assert_eq!(london_time.hour(), 8);
        assert_eq!(london_time.minute(), 30);
        assert_eq!(london_time.date(), tehran_time.date());

        // Test crossing a day boundary.
        let tehran_early_morning = z_pdt(1402, 10, 10, 2, 0, 0, Tehran);
        let london_prev_day = tehran_early_morning.with_timezone(&London);

        assert_eq!(london_prev_day.date(), ParsiDate::new(1402, 10, 9).unwrap());
        assert_eq!(london_prev_day.hour(), 22);
        assert_eq!(london_prev_day.minute(), 30);
    }

    #[test]
    fn test_daylight_saving_time_spring_forward() {
        // In New York, 2024, DST started on March 10. Clocks skipped from 1:59:59 AM to 3:00:00 AM.
        // The local time 2:30:00 AM on that day did not exist.
        // Gregorian 2024-03-10 corresponds to Parsi 1402-12-20.
        let p_year = 1402;
        let p_month = 12;
        let p_day = 20;

        let non_existent_time = ZonedParsiDateTime::new(p_year, p_month, p_day, 2, 30, 0, New_York);
        assert_eq!(non_existent_time, Err(DateError::InvalidTime));

        // Test arithmetic across the gap.
        let before_change = z_pdt(p_year, p_month, p_day, 1, 30, 0, New_York);
        let after_change = before_change.clone() + Duration::hours(1);

        assert_eq!(after_change.hour(), 3); // 1:30 AM + 1 hour jumps to 3:30 AM.
        assert_eq!(after_change.minute(), 30);
    }

    #[test]
    fn test_daylight_saving_time_fall_back() {
        // Gregorian 2024-11-03 corresponds to Parsi 1403-08-13.
        let p_year = 1403;
        let p_month = 8;
        let p_day = 13;

        let ambiguous_time =
            ZonedParsiDateTime::new(p_year, p_month, p_day, 1, 30, 0, New_York).unwrap();

        // New York's DST offset (EDT) is UTC-4.
        let dst_offset_seconds = -4 * 3600;
        assert_eq!(
            ambiguous_time.offset().fix().local_minus_utc(),
            dst_offset_seconds
        );

        let one_hour_later = ambiguous_time.clone() + Duration::hours(1);
        assert_eq!(one_hour_later.hour(), 1);
        assert_eq!(one_hour_later.minute(), 30);

        // New York's standard offset (EST) is UTC-5.
        let standard_offset_seconds = -5 * 3600;
        assert_eq!(
            one_hour_later.offset().fix().local_minus_utc(),
            standard_offset_seconds
        );
    }

    #[test]
    fn test_display_and_debug_format() {
        // Tehran winter offset is +03:30.
        let dt = z_pdt(1402, 10, 10, 12, 0, 0, Tehran);
        assert_eq!(dt.to_string(), "1402/10/10 12:00:00 +0330");

        let debug_str = format!("{:?}", dt);
        assert!(debug_str.contains("ZonedParsiDateTime"));
        assert!(debug_str.contains("datetime: ParsiDateTime"));
        assert!(debug_str.contains("timezone: Asia/Tehran"));
    }
}
