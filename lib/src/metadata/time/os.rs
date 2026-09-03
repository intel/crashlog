// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Time;
use jiff::{Timestamp, tz};

pub(super) fn now() -> Option<Time> {
    let now = Timestamp::try_from(std::time::SystemTime::now())
        .inspect_err(|err| log::info!("Cannot get the current time: {err}"))
        .ok()?;

    let tz = tz::TimeZone::system();

    if tz == tz::TimeZone::unknown() {
        log::warn!("Cannot determine the system time zone; the time is reported in UTC");
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
