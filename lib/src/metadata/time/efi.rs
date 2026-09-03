// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Time;

pub(super) fn now() -> Option<Time> {
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
