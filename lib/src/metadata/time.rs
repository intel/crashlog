// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(not(feature = "std"))]
use alloc::fmt;
#[cfg(feature = "std")]
use std::fmt;

/// Time zone the [Time] is expressed in
#[derive(Clone)]
pub enum TimeZone {
    Utc,
    Unspecified,
}

impl TimeZone {
    fn suffix(&self) -> &'static str {
        match self {
            TimeZone::Utc => "Z",
            TimeZone::Unspecified => "",
        }
    }
}

/// Crash Log Extraction Time
#[derive(Clone)]
pub struct Time {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
    pub time_zone: TimeZone,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04}{:02}{:02}T{:02}{:02}{:02}.{:03}{}",
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.millisecond,
            self.time_zone.suffix(),
        )
    }
}

impl Time {
    /// Returns the current [Time], or [None] if it cannot be determined
    ///
    /// The time is read from the operating system in UTC
    #[cfg(feature = "std")]
    pub fn now() -> Option<Self> {
        use jiff::{Timestamp, tz};

        let now = Timestamp::try_from(std::time::SystemTime::now())
            .inspect_err(|err| log::info!("Cannot get the current time: {err}"))
            .ok()?;

        let utc_time = tz::Offset::UTC.to_datetime(now);
        Some(Time {
            year: utc_time.year() as u16,
            month: utc_time.month() as u8,
            day: utc_time.day() as u8,
            hour: utc_time.hour() as u8,
            minute: utc_time.minute() as u8,
            second: utc_time.second() as u8,
            millisecond: utc_time.millisecond() as u16,
            time_zone: TimeZone::Utc,
        })
    }

    /// Returns the current [Time], or [None] if it cannot be determined
    ///
    /// The time is read from the UEFI runtime services as reported by the RTC.
    /// Its time zone is [TimeZone::Unspecified].
    #[cfg(all(not(feature = "std"), target_os = "uefi"))]
    pub fn now() -> Option<Self> {
        let time = uefi::runtime::get_time()
            .inspect_err(|err| log::info!("Cannot get RTC time: {err}"))
            .ok()?;

        Some(Time {
            year: time.year(),
            month: time.month(),
            day: time.day(),
            hour: time.hour(),
            minute: time.minute(),
            second: time.second(),
            millisecond: (time.nanosecond() / 1_000_000) as u16,
            time_zone: TimeZone::Unspecified,
        })
    }
}
