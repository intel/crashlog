// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

mod bdf;
#[cfg(all(target_os = "linux", feature = "std"))]
mod sysfs;

use super::capability::Capability;
use crate::CrashLog;
use crate::error::Error;
#[cfg(not(feature = "std"))]
use alloc::{
    collections::BTreeSet, fmt, format, str::FromStr, string::String, string::ToString, vec::Vec,
};
#[cfg(feature = "std")]
use std::{collections::BTreeSet, fmt, str::FromStr};
#[cfg(all(target_os = "linux", feature = "std"))]
use sysfs::PmtSysFs;

pub use bdf::PciBdf;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum PmtDeviceId {
    Name(String),
    Bdf(PciBdf),
}

impl fmt::Display for PmtDeviceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Name(name) => write!(f, "{name}"),
            Self::Bdf(bdf) => write!(f, "{bdf}"),
        }
    }
}

impl FromStr for PmtDeviceId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim_end_matches(char::is_numeric) == "crashlog" {
            Ok(Self::Name(s.to_string()))
        } else if let Ok(bdf) = s.parse() {
            Ok(Self::Bdf(bdf))
        } else {
            Err(())
        }
    }
}

#[derive(Default)]
pub(super) struct Pmt {
    #[cfg(target_os = "linux")]
    sysfs: PmtSysFs,
}

impl Pmt {
    #[cfg(target_os = "linux")]
    pub fn discover(&self) -> Vec<PmtDeviceId> {
        self.sysfs.discover()
    }

    #[cfg(not(target_os = "linux"))]
    pub fn discover(&self) -> Vec<PmtDeviceId> {
        Vec::new()
    }

    #[cfg(target_os = "linux")]
    pub fn extract(&self, dev: &PmtDeviceId) -> Result<Vec<CrashLog>, Error> {
        self.sysfs.extract(dev)
    }

    #[cfg(not(target_os = "linux"))]
    pub fn extract(&self, _dev: &PmtDeviceId) -> Result<Vec<CrashLog>, Error> {
        Err(Error::Unsupported)
    }

    #[cfg(target_os = "linux")]
    pub fn capabilities(&self, dev: &PmtDeviceId) -> BTreeSet<Capability> {
        self.sysfs.capabilities(dev)
    }

    #[cfg(not(target_os = "linux"))]
    pub fn capabilities(&self, _dev: &PmtDeviceId) -> BTreeSet<Capability> {
        BTreeSet::default()
    }

    #[cfg(all(target_os = "linux", feature = "control_commands"))]
    pub fn enable_disable(&self, dev: &PmtDeviceId, enable: bool) -> Result<(), Error> {
        for endpoint in self.sysfs.get_endpoints(dev) {
            endpoint.enable_disable(enable)?;
        }
        Ok(())
    }

    #[cfg(all(not(target_os = "linux"), feature = "control_commands"))]
    pub fn enable_disable(&self, _dev: &PmtDeviceId, _enable: bool) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    #[cfg(all(target_os = "linux", feature = "control_commands"))]
    pub fn clear(&self, dev: &PmtDeviceId) -> Result<(), Error> {
        for endpoint in self.sysfs.get_endpoints(dev) {
            endpoint.clear()?;
        }
        Ok(())
    }

    #[cfg(all(not(target_os = "linux"), feature = "control_commands"))]
    pub fn clear(&self, _dev: &PmtDeviceId) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    #[cfg(all(target_os = "linux", feature = "control_commands"))]
    pub fn trigger(&self, dev: &PmtDeviceId) -> Result<(), Error> {
        for endpoint in self.sysfs.get_endpoints(dev) {
            endpoint.trigger()?;
        }
        Ok(())
    }

    #[cfg(all(not(target_os = "linux"), feature = "control_commands"))]
    pub fn trigger(&self, _dev: &PmtDeviceId) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    pub fn description(&self, dev: &PmtDeviceId) -> String {
        match dev {
            PmtDeviceId::Name(name) => format!("PMT endpoint ({name})"),
            PmtDeviceId::Bdf(bdf) => format!("PMT endpoints for PCI device {bdf}"),
        }
    }
}
