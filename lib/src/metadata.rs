// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

//! Information extracted alongside the Crash Log records.

mod time;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::{fmt, format, string::String};
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
    /// Time of the extraction
    pub time: Option<Time>,
    /// When the Crash Log is extracted from a CPER, this field stores the extra CPER sections that
    /// could be read from the CPER structure.
    pub extra_cper_sections: Vec<CperSectionBody>,
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut origin: Vec<String> = Vec::new();

        if let Some(computer) = &self.computer {
            origin.push(computer.clone());
        }

        if let Some(source) = &self.source {
            origin.push(format!("{source}"));
        }

        if let Some(time) = &self.time {
            origin.push(format!("{time}"));
        }

        if origin.is_empty() {
            return write!(f, "unnamed");
        }

        write!(f, "{}", origin.join("-"))
    }
}
