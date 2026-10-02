// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

//! Information extracted alongside the Crash Log records.

mod time;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::{fmt, string::String};
#[cfg(feature = "std")]
use std::fmt;

use crate::cper::CperSectionBody;
use crate::source::CrashLogSource;

pub use time::Time;

/// Crash Log Metadata
#[derive(Default, Clone)]
pub struct Metadata {
    /// Name of the computer where the Crash Log has been extracted from.
    pub computer: Option<String>,
    /// Name of the source where the Crash Log has been extracted from.
    pub source: Option<CrashLogSource>,
    /// Time of the extraction, expressed in UTC
    pub time: Option<Time>,
    /// Types of the records found in the Crash Log (e.g. "Punit", "MCA"...), listed in the order
    /// they were first found and without duplicates. Box records are not listed.
    pub record_types: Vec<&'static str>,
    /// When the Crash Log is extracted from a CPER, this field stores the extra CPER sections that
    /// could be read from the CPER structure.
    pub extra_cper_sections: Vec<CperSectionBody>,
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut sep = "";

        if let Some(computer) = &self.computer {
            write!(f, "{computer}")?;
            sep = "-";
        }

        if let Some(source) = &self.source {
            write!(f, "{sep}{source}")?;
            sep = "-";
        }

        if let Some(time) = &self.time {
            write!(f, "{sep}{time}")?;
            sep = "-";
        }

        if !self.record_types.is_empty() {
            write!(f, "{sep}{}", self.record_types.join("+"))?;
            sep = "-";
        }

        if sep.is_empty() {
            write!(f, "unnamed")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_with_record_types_and_time() {
        let metadata = Metadata {
            record_types: vec!["Punit", "MCA"],
            time: Some(Time {
                year: 2026,
                month: 1,
                day: 2,
                hour: 3,
                minute: 4,
                second: 5,
                millisecond: 6,
            }),
            ..Metadata::default()
        };
        assert_eq!(metadata.to_string(), "20260102T030405.006Z-Punit+MCA");
    }
}
