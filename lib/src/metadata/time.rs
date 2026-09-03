// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(not(feature = "std"))]
use alloc::fmt;
#[cfg(feature = "std")]
use std::fmt;

/// Crash Log Extraction Time, expressed in the local time of the system
#[derive(Clone)]
pub struct Time {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04}{:02}{:02}T{:02}{:02}{:02}.{:03}",
            self.year, self.month, self.day, self.hour, self.minute, self.second, self.millisecond,
        )
    }
}

impl Time {
    /// Returns the current [Time], or [None] if it cannot be determined
    ///
    /// The time is read from the operating system
    #[cfg(feature = "std")]
    pub fn now() -> Option<Self> {
        use jiff::{Timestamp, tz};

        let now = Timestamp::try_from(std::time::SystemTime::now())
            .inspect_err(|err| log::info!("Cannot get the current time: {err}"))
            .ok()?;

        let tz = tz::TimeZone::system();

        if tz == tz::TimeZone::unknown() {
            log::warn!("Cannot determine the system time zone");
        }

        let local = now.to_zoned(tz).datetime();
        Some(Time {
            year: local.year() as u16,
            month: local.month() as u8,
            day: local.day() as u8,
            hour: local.hour() as u8,
            minute: local.minute() as u8,
            second: local.second() as u8,
            millisecond: local.millisecond() as u16,
        })
    }

    /// Returns the current [Time], or [None] if it cannot be determined
    ///
    /// The time is read from the UEFI runtime services
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
        })
    }
}
