// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

//! Information extracted alongside the Crash Log records.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::{fmt, string::String};
#[cfg(feature = "std")]
use std::fmt;

use crate::cper::CperSectionBody;
use crate::source::CrashLogSource;

/// Crash Log Metadata
#[derive(Default, Clone)]
pub struct Metadata {
    /// Name of the computer where the Crash Log has been extracted from.
    pub computer: Option<String>,
    /// Name of the source where the Crash Log has been extracted from.
    pub source: Option<CrashLogSource>,
    /// Time of the extraction
    pub time: Option<Time>,
    /// When the Crash Log is extracted from a CPER, this field stores the extra CPER sections that
    /// could be read from the CPER structure.
    pub extra_cper_sections: Vec<CperSectionBody>,
}

/// Crash Log Extraction Time
#[derive(Clone)]
pub struct Time {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (
            self.computer.as_ref(),
            self.source.as_ref(),
            self.time.as_ref(),
        ) {
            (Some(computer), Some(source), Some(time)) => write!(f, "{computer}-{source}-{time}"),
            (Some(computer), None, Some(time)) => write!(f, "{computer}-{time}"),
            (None, None, Some(time)) => write!(f, "{time}"),
            (None, Some(source), Some(time)) => write!(f, "{source}-{time}"),
            (Some(computer), None, None) => write!(f, "{computer}"),
            (Some(computer), Some(source), None) => write!(f, "{computer}-{source}"),
            (None, Some(source), None) => write!(f, "{source}"),
            (None, None, None) => write!(f, "unnamed"),
        }
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}-{:02}-{:02}",
            self.year, self.month, self.day, self.hour, self.minute
        )
    }
}
