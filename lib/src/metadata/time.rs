// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(all(not(feature = "std"), target_os = "uefi"))]
mod efi;
#[cfg(feature = "std")]
mod os;

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
    /// Returns the current [Time]
    pub fn now() -> Option<Self> {
        #[cfg(feature = "std")]
        {
            os::now()
        }
        #[cfg(all(not(feature = "std"), target_os = "uefi"))]
        {
            efi::now()
        }
        #[cfg(not(any(feature = "std", target_os = "uefi")))]
        {
            None
        }
    }
}
