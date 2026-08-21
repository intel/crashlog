// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

//! Crash Log sources and their capabilities
//!
//! This module provides abstractions for discovering, querying, and extracting Crash Log data
//! from various platform sources. Each source has different capabilities such as extraction,
//! on-demand triggering, and enable/disable control.
//!
//! # Supported Sources
//!
//! - **ACPI BERT** (`acpi`) - Boot Error Record Table for firmware-reported errors
//! - **Intel PMT** (`pmt:<device>`) - Platform Monitoring Technology devices with Crash Log regions
//! - **Event Log** (`evt`) - Operating system event logs (Windows)
//!
//! # Examples
//!
//! Discovering available sources:
//!
//! ```
//! use intel_crashlog::prelude::*;
//!
//! let sources = CrashLogSource::discover();
//! for source in sources {
//!     println!("{}: {}", source, source.description());
//!     println!("Capabilities: {:?}", source.capabilities());
//! }
//! ```
mod acpi;
mod capability;
mod event_log;
mod pmt;

use crate::CrashLog;
use crate::error::Error;
use acpi::Acpi;
#[cfg(not(feature = "std"))]
use alloc::{fmt, str::FromStr, string::String, string::ToString, vec, vec::Vec};
use event_log::EventLog;
use pmt::{Pmt, PmtDeviceId};
#[cfg(feature = "std")]
use std::{fmt, str::FromStr};

pub use capability::{Capabilities, Capability};

/// Represents a source from which Crash Log data can be extracted
///
/// Each source support different type of capabilities, which are represented as
/// [Capabilities]
#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum CrashLogSource {
    /// ACPI BERT table
    Acpi,
    /// Intel PMT device that exposes a Crash Log region
    PmtDevice(PmtDeviceId),
    /// OS Event Log
    EventLog,
}

impl fmt::Display for CrashLogSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Acpi => write!(f, "acpi"),
            Self::PmtDevice(dev) => write!(f, "pmt:{}", dev),
            Self::EventLog => write!(f, "evt"),
        }
    }
}

/// Error type returned when parsing a [`CrashLogSource`] from a string fails
///
/// This error is produced by the [`FromStr`] implementation for [`CrashLogSource`]
/// when the input string cannot be parsed into a valid Crash Log source.
#[derive(Debug, PartialEq)]
pub enum ParseCrashLogSourceError {
    /// The source name is not recognized
    ///
    /// Valid source names are: `acpi`, `pmt`, and `evt`
    InvalidSource,
    /// The parameter provided for the source is invalid
    ///
    /// This occurs when:
    /// - A parameter is provided for sources that don't accept parameters (`acpi`, `evt`)
    /// - The parameter format is invalid for PMT sources (must be either a device name
    ///   like `crashlog0` or a PCI BDF address like `0000:00:1f.5`)
    InvalidParameter,
}

impl fmt::Display for ParseCrashLogSourceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidSource => write!(f, "Invalid source"),
            Self::InvalidParameter => write!(f, "Invalid parameter"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseCrashLogSourceError {}

impl FromStr for CrashLogSource {
    type Err = ParseCrashLogSourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (source, parameter) = s.split_once(":").unwrap_or((s, ""));

        match source {
            "acpi" => {
                if parameter.is_empty() {
                    Ok(Self::Acpi)
                } else {
                    Err(Self::Err::InvalidParameter)
                }
            }
            "pmt" => {
                if let Ok(dev) = parameter.parse() {
                    Ok(Self::PmtDevice(dev))
                } else {
                    Err(Self::Err::InvalidParameter)
                }
            }
            "evt" => {
                if parameter.is_empty() {
                    Ok(Self::EventLog)
                } else {
                    Err(Self::Err::InvalidParameter)
                }
            }
            _ => Err(Self::Err::InvalidSource),
        }
    }
}

impl CrashLogSource {
    /// Returns all the Crash Log sources that are available in the platform
    pub fn discover() -> Vec<Self> {
        let mut sources = Vec::new();

        if Acpi::default().is_available() {
            sources.push(Self::Acpi);
        }

        if EventLog::default().is_available() {
            sources.push(Self::EventLog);
        }

        let pmt_devices: Vec<CrashLogSource> = Pmt::default()
            .discover()
            .into_iter()
            .map(Self::PmtDevice)
            .collect();

        sources.extend(pmt_devices);

        sources
    }

    /// Returns the Crash Log extracted from the platform using the current Crash Log source
    #[cfg(feature = "extraction")]
    pub fn extract(&self) -> Result<Vec<CrashLog>, Error> {
        let mut crashlogs = match self {
            Self::Acpi => Acpi::default().extract().map(|crashlog| vec![crashlog]),
            Self::PmtDevice(dev) => Pmt::default().extract(dev),
            Self::EventLog => EventLog::default().extract(),
        };

        if let Ok(ref mut crashlogs) = crashlogs {
            for crashlog in crashlogs.iter_mut() {
                crashlog.metadata.source = Some(self.clone());
            }
        }

        crashlogs
    }

    /// Triggers an on-demand Crash Log collection on this source
    #[cfg(feature = "control_commands")]
    pub fn trigger(&self) -> Result<(), Error> {
        match self {
            Self::PmtDevice(dev) => Pmt::default().trigger(dev),
            _ => Err(Error::Unsupported),
        }
    }

    /// Rearms a Crash Log trigger on this source
    #[cfg(feature = "control_commands")]
    pub fn rearm(&self) -> Result<(), Error> {
        match self {
            Self::PmtDevice(dev) => Pmt::default().rearm(dev),
            _ => Err(Error::Unsupported),
        }
    }

    /// Clears the Crash Log storage on this source
    #[cfg(feature = "control_commands")]
    pub fn clear(&self) -> Result<(), Error> {
        match self {
            Self::PmtDevice(dev) => Pmt::default().clear(dev),
            _ => Err(Error::Unsupported),
        }
    }

    /// Enable the Crash Log collection on this source
    #[cfg(feature = "control_commands")]
    pub fn enable(&self) -> Result<(), Error> {
        match self {
            Self::PmtDevice(dev) => Pmt::default().enable_disable(dev, true),
            _ => Err(Error::Unsupported),
        }
    }

    /// Disable the Crash Log collection on this source
    #[cfg(feature = "control_commands")]
    pub fn disable(&self) -> Result<(), Error> {
        match self {
            Self::PmtDevice(dev) => Pmt::default().enable_disable(dev, false),
            _ => Err(Error::Unsupported),
        }
    }

    /// Returns a human readable description of the Crash Log source
    pub fn description(&self) -> String {
        match &self {
            Self::Acpi => "ACPI BERT".to_string(),
            Self::EventLog => "Windows Event Log".to_string(),
            Self::PmtDevice(dev) => Pmt::default().description(dev),
        }
    }

    /// Returns all the capabilities of the Crash Log Source
    pub fn capabilities(&self) -> Capabilities {
        match self {
            Self::Acpi => Capabilities::from([Capability::Extract]),
            Self::EventLog => Capabilities::from([Capability::Extract]),
            Self::PmtDevice(dev) => Pmt::default().capabilities(dev),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pmt::PciBdf;
    use super::*;

    #[test]
    fn parse() {
        assert_eq!("acpi".parse(), Ok(CrashLogSource::Acpi));
        assert_eq!(
            "pmt:crashlog42".parse(),
            Ok(CrashLogSource::PmtDevice(PmtDeviceId::Name(
                "crashlog42".to_string()
            )))
        );
        assert_eq!(
            "pmt:1111:22:33.4".parse(),
            Ok(CrashLogSource::PmtDevice(PmtDeviceId::Bdf(PciBdf::new(
                0x1111, 0x22, 0x33, 0x4
            ))))
        );
        assert_eq!("evt".parse(), Ok(CrashLogSource::EventLog));
        assert_eq!(
            "foo".parse::<CrashLogSource>(),
            Err(ParseCrashLogSourceError::InvalidSource)
        );
        assert_eq!(
            "acpi:foo".parse::<CrashLogSource>(),
            Err(ParseCrashLogSourceError::InvalidParameter)
        );
    }
}
