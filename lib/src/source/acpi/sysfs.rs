// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use crate::CrashLog;
use crate::bert::Berr;
use crate::error::Error;
use std::path::{Path, PathBuf};

pub(super) struct AcpiSysFs {
    root: PathBuf,
}

impl Default for AcpiSysFs {
    fn default() -> Self {
        Self::new(Path::new("/"))
    }
}

impl AcpiSysFs {
    pub fn new(path: &Path) -> Self {
        Self {
            root: path.to_owned(),
        }
    }

    fn tables_path(&self) -> PathBuf {
        let mut path = self.root.clone();
        path.push("sys");
        path.push("firmware");
        path.push("acpi");
        path.push("tables");
        path
    }

    fn berr_path(&self) -> PathBuf {
        let mut path = self.tables_path();
        path.push("data");
        path.push("BERT");
        path
    }

    pub fn extract(&self) -> Result<CrashLog, Error> {
        let path = self.berr_path();
        let berr = std::fs::read(&path)
            .map_err(|err| {
                log::warn!("Cannot read {}: {err}", path.display());
                match err.kind() {
                    std::io::ErrorKind::NotFound => Error::NoCrashLogFound,
                    _ => Error::from(err),
                }
            })
            .and_then(|berr| {
                log::info!("Found ACPI boot error record in sysfs");
                Berr::from_slice(&berr).ok_or(Error::InvalidBootErrorRecordRegion)
            })?;

        CrashLog::from_berr(berr)
    }
}

impl CrashLog {
    /// Reads the Crash Log reported through ACPI from the Linux sysfs
    pub fn from_acpi_sysfs() -> Result<Self, Error> {
        AcpiSysFs::default().extract()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn extract() {
        let root = tempfile::tempdir().unwrap();

        let acpi = AcpiSysFs::new(root.path());

        let mut berr_path = root.path().to_owned();
        berr_path.push("sys");
        berr_path.push("firmware");
        berr_path.push("acpi");
        berr_path.push("tables");
        berr_path.push("data");
        std::fs::create_dir_all(&berr_path).unwrap();

        berr_path.push("BERT");

        let bert = std::fs::read("tests/samples/dummy.bert").unwrap();
        let crashlog = CrashLog::from_slice(&bert).unwrap();

        let berr = Berr::from_crashlog(&crashlog);
        std::fs::write(berr_path, berr.to_bytes()).unwrap();

        let extracted_crashlog = acpi.extract().unwrap();

        assert_eq!(crashlog.to_bytes(), extracted_crashlog.to_bytes());
    }
}
