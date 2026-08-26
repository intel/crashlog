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
#[cfg(all(target_os = "linux", feature = "control_commands"))]
use sysfs::PmtSysFsEndpoint;

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
        self.run_on_endpoints(dev, |endpoint| endpoint.enable_disable(enable))
    }

    #[cfg(all(not(target_os = "linux"), feature = "control_commands"))]
    pub fn enable_disable(&self, _dev: &PmtDeviceId, _enable: bool) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    #[cfg(all(target_os = "linux", feature = "control_commands"))]
    pub fn clear(&self, dev: &PmtDeviceId) -> Result<(), Error> {
        self.run_on_endpoints(dev, |endpoint| endpoint.clear())
    }

    #[cfg(all(not(target_os = "linux"), feature = "control_commands"))]
    pub fn clear(&self, _dev: &PmtDeviceId) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    #[cfg(all(target_os = "linux", feature = "control_commands"))]
    pub fn rearm(&self, dev: &PmtDeviceId) -> Result<(), Error> {
        self.run_on_endpoints(dev, |endpoint| endpoint.rearm())
    }

    #[cfg(all(not(target_os = "linux"), feature = "control_commands"))]
    pub fn rearm(&self, _dev: &PmtDeviceId) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    #[cfg(all(target_os = "linux", feature = "control_commands"))]
    pub fn trigger(&self, dev: &PmtDeviceId) -> Result<(), Error> {
        self.run_on_endpoints(dev, |endpoint| endpoint.trigger())
    }

    #[cfg(all(not(target_os = "linux"), feature = "control_commands"))]
    pub fn trigger(&self, _dev: &PmtDeviceId) -> Result<(), Error> {
        Err(Error::Unsupported)
    }

    #[cfg(all(target_os = "linux", feature = "control_commands"))]
    fn run_on_endpoints<F>(&self, dev: &PmtDeviceId, command: F) -> Result<(), Error>
    where
        F: Fn(&PmtSysFsEndpoint) -> Result<(), Error>,
    {
        let endpoints = self.sysfs.get_endpoints(dev);
        if endpoints.is_empty() {
            return Err(Error::NoCrashLogSourceFound);
        }

        let mut cmd_success = false;
        let mut first_error = None;

        for endpoint in &endpoints {
            match command(endpoint) {
                Ok(()) => cmd_success = true,
                Err(err) => {
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                }
            }
        }

        if !cmd_success {
            // Unreachable `unwrap_or`: `endpoints` is not empty and every failing endpoint
            // records an error, so `first_error` is always set when no endpoint succeeded.
            return Err(first_error.unwrap_or(Error::InternalError));
        }

        if let Some(err) = first_error {
            log::warn!("Crash Log command failed on some endpoints: {err}");
        }

        Ok(())
    }

    pub fn description(&self, dev: &PmtDeviceId) -> String {
        match dev {
            PmtDeviceId::Name(name) => format!("PMT endpoint ({name})"),
            PmtDeviceId::Bdf(bdf) => format!("PMT endpoints for PCI device {bdf}"),
        }
    }
}
