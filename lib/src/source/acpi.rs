// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(target_os = "uefi")]
mod efi;
#[cfg(all(target_os = "linux", feature = "std"))]
mod sysfs;

use crate::CrashLog;
use crate::error::Error;
#[cfg(all(target_os = "linux", feature = "std"))]
use sysfs::AcpiSysFs;

#[derive(Default)]
pub struct Acpi {
    #[cfg(target_os = "linux")]
    sysfs: AcpiSysFs,
}

impl Acpi {
    pub fn is_available(&self) -> bool {
        cfg!(any(target_os = "linux", target_os = "uefi"))
    }

    pub fn extract(&self) -> Result<CrashLog, Error> {
        #[cfg(target_os = "linux")]
        {
            match self.sysfs.extract() {
                Ok(crashlogs) => return Ok(crashlogs),
                Err(err) => log::error!("Cannot extract Crash Log from ACPI sysfs: {err}"),
            }
        }

        #[cfg(target_os = "uefi")]
        {
            match efi::extract_crashlog() {
                Ok(crashlog) => return Ok(crashlog),
                Err(err) => log::error!("Cannot extract Crash Log from ACPI tables: {err}"),
            }
        }

        Err(Error::NoCrashLogFound)
    }
}
