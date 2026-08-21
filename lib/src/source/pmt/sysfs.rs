// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use super::PmtDeviceId;
use super::bdf::PciBdf;
use crate::CrashLog;
use crate::error::Error;
use crate::region::Region;
use crate::source::{Capabilities, Capability};
use std::collections::BTreeSet;
#[cfg(feature = "control_commands")]
use std::io::Write;
use std::path::{Path, PathBuf};

pub(super) struct PmtSysFs {
    root: PathBuf,
}

pub(super) struct PmtSysFsEndpoint {
    path: PathBuf,
}

impl Default for PmtSysFs {
    fn default() -> Self {
        Self::new(Path::new("/"))
    }
}

impl PmtSysFs {
    pub fn new(path: &Path) -> Self {
        Self {
            root: path.to_owned(),
        }
    }

    fn class_path(&self) -> PathBuf {
        let mut path = self.root.clone();
        path.push("sys");
        path.push("class");
        path.push("intel_pmt");
        path
    }

    fn pci_dev_path(&self) -> PathBuf {
        let mut path = self.root.clone();
        path.push("sys");
        path.push("bus");
        path.push("pci");
        path.push("devices");
        path
    }

    fn discover_from_class(&self) -> Vec<PmtDeviceId> {
        let path = self.class_path();

        std::fs::read_dir(&path)
            .inspect_err(|err| log::warn!("Cannot read {}: {err}", path.display()))
            .ok()
            .map(|read_dir| {
                read_dir
                    .filter_map(|entry| entry.ok())
                    .filter_map(|entry| {
                        let name = entry.file_name();
                        let entry_name = name.to_str().unwrap_or_default();

                        if entry_name.trim_end_matches(char::is_numeric) == "crashlog" {
                            Some(PmtDeviceId::Name(entry_name.to_string()))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn has_crashlog_vsec(path: &Path) -> bool {
        match std::fs::read_dir(path) {
            Err(err) => log::warn!("Cannot read {}: {err}", path.display()),
            Ok(entries) => {
                for entry in entries {
                    let Ok(entry) = entry else { continue };
                    let filename = entry.file_name();
                    let Some(filename) = filename.to_str() else {
                        continue;
                    };
                    if filename.trim_end_matches(char::is_numeric) == "intel_vsec.crashlog." {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn discover_from_pci(&self) -> Vec<PmtDeviceId> {
        let path = self.pci_dev_path();
        let mut devices = BTreeSet::default();

        match std::fs::read_dir(&path) {
            Err(err) => log::warn!("Cannot read {}: {err}", path.display()),
            Ok(entries) => {
                for entry in entries {
                    let Ok(entry) = entry else {
                        continue;
                    };
                    let filename = entry.file_name();
                    let Some(filename) = filename.to_str() else {
                        continue;
                    };
                    let Ok(bdf) = filename.parse() else {
                        continue;
                    };

                    if Self::has_crashlog_vsec(&entry.path()) {
                        devices.insert(PmtDeviceId::Bdf(bdf));
                    }
                }
            }
        }

        devices.into_iter().collect()
    }

    pub fn discover(&self) -> Vec<PmtDeviceId> {
        let mut devices = self.discover_from_class();
        devices.extend(self.discover_from_pci());
        devices
    }

    pub fn get_all_endpoints(&self) -> Vec<PmtSysFsEndpoint> {
        self.discover()
            .into_iter()
            .flat_map(|devid| self.get_endpoints(&devid))
            .collect()
    }

    fn get_endpoints_from_vsec(path: &Path) -> Vec<PmtSysFsEndpoint> {
        let mut pmt_path = path.to_owned();
        pmt_path.push("intel_pmt");
        let mut endpoints = Vec::default();

        match std::fs::read_dir(&pmt_path) {
            Err(err) => log::warn!("Cannot read {}: {err}", path.display()),
            Ok(entries) => {
                for entry in entries {
                    let Ok(entry) = entry else {
                        continue;
                    };
                    let filename = entry.file_name();
                    let Some(filename) = filename.to_str() else {
                        continue;
                    };
                    if filename.trim_end_matches(char::is_numeric) == "crashlog"
                        && let Some(endpoint) = PmtSysFsEndpoint::new(&entry.path())
                    {
                        endpoints.push(endpoint);
                    }
                }
            }
        }

        endpoints
    }

    fn get_endpoints_from_bdf(&self, bdf: &PciBdf) -> Vec<PmtSysFsEndpoint> {
        let mut endpoints = Vec::default();
        let mut path = self.pci_dev_path();
        path.push(bdf.to_string());

        match std::fs::read_dir(&path) {
            Err(err) => log::error!("Cannot read {}: {err}", path.display()),
            Ok(entries) => {
                for entry in entries {
                    let Ok(entry) = entry else {
                        continue;
                    };
                    let filename = entry.file_name();
                    let Some(filename) = filename.to_str() else {
                        continue;
                    };
                    if filename.trim_end_matches(char::is_numeric) == "intel_vsec.crashlog." {
                        endpoints.extend(Self::get_endpoints_from_vsec(&entry.path()));
                    }
                }
            }
        }

        endpoints
    }

    pub fn get_endpoints(&self, id: &PmtDeviceId) -> Vec<PmtSysFsEndpoint> {
        match id {
            PmtDeviceId::Name(name) => {
                let mut path = self.class_path();
                path.push(name);

                let mut endpoints = Vec::new();
                if let Some(endpoint) = PmtSysFsEndpoint::new(&path) {
                    endpoints.push(endpoint);
                }
                endpoints
            }
            PmtDeviceId::Bdf(bdf) => self.get_endpoints_from_bdf(bdf),
        }
    }

    pub fn extract(&self, dev: &PmtDeviceId) -> Result<Vec<CrashLog>, Error> {
        let mut crashlogs = Vec::new();

        for endpoint in self.get_endpoints(dev) {
            match endpoint.extract() {
                Ok(crashlog) => crashlogs.push(crashlog),
                Err(Error::EmptyRegion) => (),
                Err(err) => return Err(err),
            }
        }

        Ok(crashlogs)
    }

    pub fn capabilities(&self, dev: &PmtDeviceId) -> Capabilities {
        let mut capabilities = Capabilities::new();

        for endpoint in self.get_endpoints(dev) {
            capabilities.append(&mut endpoint.capabilities());
        }

        capabilities
    }
}

impl PmtSysFsEndpoint {
    pub fn new(path: &Path) -> Option<Self> {
        if path.is_dir() {
            Some(Self {
                path: path.to_owned(),
            })
        } else {
            log::error!("{} is not a valid PMT Crash Log endpoint", path.display());
            None
        }
    }

    #[cfg(feature = "control_commands")]
    pub fn trigger(&self) -> Result<(), Error> {
        self.write_command("trigger", b"1")
    }

    #[cfg(feature = "control_commands")]
    pub fn clear(&self) -> Result<(), Error> {
        self.write_command("clear", b"1")
    }

    #[cfg(feature = "control_commands")]
    pub fn enable_disable(&self, enable: bool) -> Result<(), Error> {
        self.write_command("enable", if enable { b"1" } else { b"0" })
    }

    #[cfg(feature = "control_commands")]
    fn write_command(&self, entry: &str, value: &[u8]) -> Result<(), Error> {
        let path = self.path.join(entry);

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)
            .map_err(Error::IOError)
            .inspect_err(|err| log::warn!("{}: {err}", path.display()))?;

        file.write_all(value)
            .map_err(Error::IOError)
            .inspect_err(|err| log::warn!("Failed to write to {}: {err}", path.display()))?;

        Ok(())
    }

    pub fn capabilities(&self) -> Capabilities {
        let mut capabilities: Capabilities = Capabilities::new();

        if self.path.join("crashlog").exists() {
            capabilities.insert(Capability::Extract);
        }

        if self.path.join("trigger").exists() {
            capabilities.insert(Capability::Trigger);
        }

        if self.path.join("enable").exists() {
            capabilities.insert(Capability::EnableDisable);
        }

        if self.path.join("clear").exists() {
            capabilities.insert(Capability::Clear);
        }

        capabilities
    }

    pub fn extract(&self) -> Result<CrashLog, Error> {
        CrashLog::from_regions(vec![
            std::fs::read(self.path.join("crashlog"))
                .map_err(Error::IOError)
                .and_then(|region| Region::from_slice(&region))
                .inspect(|_| log::info!("Extracted valid record from {}", self.path.display()))
                .inspect_err(|err| log::error!("{}: {err}", self.path.display()))?,
        ])
    }
}

impl CrashLog {
    /// Reads the Crash Log reported through all Intel PMT devices from the Linux sysfs
    pub fn from_pmt_sysfs() -> Result<Self, Error> {
        let regions: Vec<Region> = PmtSysFs::default()
            .get_all_endpoints()
            .into_iter()
            .filter_map(|dev| dev.extract().ok())
            .flat_map(|crashlog| crashlog.regions)
            .collect();

        CrashLog::from_regions(regions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn discovery() {
        let root = tempfile::tempdir().unwrap();

        let sysfs = PmtSysFs::new(root.path());
        assert!(sysfs.discover().is_empty());

        let mut class_path = root.path().to_owned();
        class_path.push("sys");
        class_path.push("class");
        class_path.push("intel_pmt");
        std::fs::create_dir_all(&class_path).unwrap();

        for dev in 0..2 {
            let mut dev_path = class_path.clone();
            dev_path.push(format!("crashlog{}", dev));
            std::fs::create_dir(dev_path).unwrap();
        }

        let mut dummy_dev_path = class_path.clone();
        dummy_dev_path.push("foo");
        std::fs::create_dir(dummy_dev_path).unwrap();

        assert_eq!(sysfs.discover_from_class().len(), 2);
    }

    #[test]
    fn discovery_pci() {
        let root = tempfile::tempdir().unwrap();

        let sysfs = PmtSysFs::new(root.path());
        assert!(sysfs.discover().is_empty());

        let mut pci_root_path = root.path().to_owned();
        pci_root_path.push("sys");
        pci_root_path.push("bus");
        pci_root_path.push("pci");
        pci_root_path.push("devices");
        std::fs::create_dir_all(&pci_root_path).unwrap();

        // Create PCI BDF directories with intel_vsec.crashlog markers
        let bdfs = vec!["0000:00:1f.5", "0000:00:0e.0"];

        for bdf in &bdfs {
            let mut bdf_dir = pci_root_path.clone();
            bdf_dir.push(bdf);
            std::fs::create_dir(&bdf_dir).unwrap();

            let mut vsec_marker = bdf_dir.clone();
            vsec_marker.push("intel_vsec.crashlog.42");
            std::fs::create_dir(&vsec_marker).unwrap();

            let mut vsec_marker = bdf_dir.clone();
            vsec_marker.push("intel_vsec.crashlog.1337");
            std::fs::create_dir(&vsec_marker).unwrap();
        }

        let devices = sysfs.discover();
        assert_eq!(devices.len(), 2);

        assert!(devices.contains(&PmtDeviceId::Bdf(PciBdf::new(0, 0, 0x1f, 5))));
        assert!(devices.contains(&PmtDeviceId::Bdf(PciBdf::new(0, 0, 0x0e, 0))));
    }

    #[test]
    fn extract() {
        let root = tempfile::tempdir().unwrap();

        let sysfs = PmtSysFs::new(root.path());
        assert!(sysfs.discover().is_empty());

        let mut dev_path = root.path().to_owned();
        dev_path.push("sys");
        dev_path.push("class");
        dev_path.push("intel_pmt");
        dev_path.push("crashlog0");
        std::fs::create_dir_all(&dev_path).unwrap();

        let data = std::fs::read("tests/samples/dummy_crashlog_agent_rev1.crashlog").unwrap();
        let crashlog = CrashLog::from_slice(&data).unwrap();

        let mut path = dev_path.to_owned();
        path.push("crashlog");
        std::fs::write(path, crashlog.regions[0].to_bytes()).unwrap();

        let devices = sysfs.discover();
        let dev = sysfs.get_endpoints(&devices[0]);

        let extracted_crashlog = dev[0].extract().unwrap();
        assert_eq!(crashlog.to_bytes(), extracted_crashlog.to_bytes());
    }

    #[test]
    fn trigger() {
        let root = tempfile::tempdir().unwrap();

        let dev = PmtSysFsEndpoint::new(root.path()).unwrap();
        assert!(matches!(dev.trigger(), Err(Error::IOError(_))));

        let mut path = root.path().to_owned();
        path.push("trigger");
        std::fs::write(&path, b"0").unwrap();

        dev.trigger().unwrap();
        assert_eq!(&std::fs::read_to_string(&path).unwrap(), "1");
    }

    #[test]
    fn enable_disable() {
        let root = tempfile::tempdir().unwrap();

        let dev = PmtSysFsEndpoint::new(root.path()).unwrap();
        assert!(matches!(dev.enable_disable(true), Err(Error::IOError(_))));

        let mut path = root.path().to_owned();
        path.push("enable");
        std::fs::write(&path, b"0").unwrap();

        dev.enable_disable(true).unwrap();
        assert_eq!(&std::fs::read_to_string(&path).unwrap(), "1");

        dev.enable_disable(false).unwrap();
        assert_eq!(&std::fs::read_to_string(&path).unwrap(), "0");
    }

    #[test]
    fn capabilities() {
        let root = tempfile::tempdir().unwrap();

        let sysfs = PmtSysFs::new(root.path());
        assert!(sysfs.discover().is_empty());

        let mut dev_path = root.path().to_owned();
        dev_path.push("sys");
        dev_path.push("class");
        dev_path.push("intel_pmt");
        dev_path.push("crashlog0");
        std::fs::create_dir_all(&dev_path).unwrap();

        let mut cl_path = dev_path.to_owned();
        cl_path.push("crashlog");
        std::fs::create_dir(&cl_path).unwrap();

        let mut trigger_path = dev_path.to_owned();
        trigger_path.push("trigger");
        std::fs::create_dir(&trigger_path).unwrap();

        let devices = sysfs.discover();
        let dev = sysfs.get_endpoints(&devices[0]);
        let dev_capabilities = dev[0].capabilities();

        assert_eq!(dev_capabilities.len(), 2);
        assert!(dev_capabilities.contains(&Capability::Extract));
        assert!(dev_capabilities.contains(&Capability::Trigger));
    }

    #[test]
    fn control_commands() {
        let root = tempfile::tempdir().unwrap();

        let sysfs = PmtSysFs::new(root.path());
        assert!(sysfs.discover().is_empty());

        let mut dev_path = root.path().to_owned();
        dev_path.push("sys");
        dev_path.push("class");
        dev_path.push("intel_pmt");
        dev_path.push("crashlog0");
        std::fs::create_dir_all(&dev_path).unwrap();

        let mut enable_path = dev_path.to_owned();
        enable_path.push("enable");
        std::fs::write(&enable_path, b"0").unwrap();

        let mut trigger_path = dev_path.to_owned();
        trigger_path.push("trigger");
        std::fs::write(&trigger_path, b"0").unwrap();

        let devices = sysfs.discover();
        let dev = sysfs.get_endpoints(&devices[0]);

        dev[0].trigger().unwrap();
        assert_eq!(&std::fs::read_to_string(&trigger_path).unwrap(), "1");

        dev[0].enable_disable(true).unwrap();
        assert_eq!(&std::fs::read_to_string(&enable_path).unwrap(), "1");

        dev[0].enable_disable(false).unwrap();
        assert_eq!(&std::fs::read_to_string(&enable_path).unwrap(), "0");
    }
}
