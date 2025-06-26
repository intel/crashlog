// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use crate::CrashLog;
use crate::bert::Berr;
use crate::error::Error;
use crate::region::Region;

const BERR_PATH: &str = "/sys/firmware/acpi/tables/data/BERT";
const PMT_PATH: &str = "/sys/class/intel_pmt";

impl CrashLog {
    pub(crate) fn from_acpi_sysfs() -> Result<Self, Error> {
        let berr = std::fs::read(BERR_PATH)
            .map_err(|err| {
                log::warn!("Cannot read {BERR_PATH}: {err}");
                match err.kind() {
                    std::io::ErrorKind::NotFound => Error::NoCrashLogFound,
                    _ => Error::from(err),
                }
            })
            .and_then(|berr| {
                log::info!("Found ACPI boot error record in sysfs");
                Berr::from_slice(&berr).ok_or(Error::InvalidBootErrorRecordRegion)
            })?;

        Self::from_berr(berr)
    }

    pub(crate) fn from_pmt_sysfs() -> Result<Self, Error> {
        let regions: Vec<Region> = std::fs::read_dir(PMT_PATH)
            .map_err(|err| {
                log::warn!("Cannot read {PMT_PATH}: {err}");
                match err.kind() {
                    std::io::ErrorKind::NotFound => Error::NoCrashLogFound,
                    _ => Error::from(err),
                }
            })?
            .filter_map(|entry| {
                entry
                    .inspect_err(|err| log::error!("Cannot access directory entry: {err}"))
                    .ok()
            })
            .filter(|entry| {
                let is_dir = entry
                    .file_type()
                    .map(|file_type| file_type.is_dir())
                    .unwrap_or(false);
                let is_crashlog_dir = entry
                    .file_name()
                    .to_str()
                    .map(|name| name.starts_with("crashlog"))
                    .unwrap_or(false);

                is_dir && is_crashlog_dir
            })
            .filter_map(|entry| {
                let mut path = entry.path();

                log::info!("Found Crash Log entry in PMT sysfs: {}", path.display());
                path.push("crashlog");

                std::fs::read(&path)
                    .map_err(Error::IOError)
                    .and_then(|region| Region::from_slice(&region))
                    .inspect(|_| log::info!("Extracted valid record from {}", path.display()))
                    .inspect_err(|err| log::error!("{}: {err}", path.display()))
                    .ok()
            })
            .collect();

        Self::from_regions(regions)
    }
}
