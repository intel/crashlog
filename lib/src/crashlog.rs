// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use crate::Error;
use crate::bert::{Berr, Bert};
#[cfg(feature = "collateral_manager")]
use crate::collateral::{CollateralManager, CollateralTree};
use crate::cper::Cper;
#[cfg(feature = "extraction")]
use crate::extract;
use crate::metadata::Metadata;
use crate::node::Node;
use crate::region::Region;
#[cfg(not(feature = "std"))]
use alloc::{collections::VecDeque, vec, vec::Vec};
#[cfg(target_os = "uefi")]
use core::ptr::NonNull;
#[cfg(feature = "std")]
use std::collections::VecDeque;
#[cfg(target_os = "uefi")]
use uefi_raw::table::system::SystemTable;

use crate::header::record_types;

/// Set of all the Crash Log records captured on a platform.
#[derive(Default)]
pub struct CrashLog {
    /// Crash Log regions captured on the platform.
    pub regions: Vec<Region>,
    /// Extra information extracted alongside the Crash Log records
    pub metadata: Metadata,
}

impl CrashLog {
    fn from_regions(regions: Vec<Region>) -> Result<Self, Error> {
        let mut queue = VecDeque::from(regions);
        let mut regions = Vec::new();

        while let Some(region) = queue.pop_front() {
            for record in region.records.iter() {
                if record.header.version.record_type != record_types::BOX {
                    continue;
                }

                let Some(payload) = record.data.get(record.header.header_size()..) else {
                    log::error!("The Box record has an empty payload");
                    continue;
                };

                match Region::from_slice(payload) {
                    Ok(region) => queue.push_front(region),
                    Err(err) => log::error!("Invalid region in Box record: {err}"),
                }
            }

            regions.push(region)
        }

        if regions.is_empty() {
            return Err(Error::InvalidCrashLog);
        }

        Ok(CrashLog {
            regions,
            ..CrashLog::default()
        })
    }

    /// Extracts the Crash Log records from [Berr].
    pub(crate) fn from_berr(berr: Berr) -> Result<Self, Error> {
        let regions = berr
            .entries
            .iter()
            .filter_map(|entry| Region::from_cper_section(&entry.cper_section))
            .collect();
        CrashLog::from_regions(regions)
    }

    #[cfg(all(target_os = "uefi", feature = "extraction"))]
    /// Reads the Crash Log records from the EFI System Table.
    pub fn from_system_table(system_table: Option<NonNull<SystemTable>>) -> Result<Self, Error> {
        extract::efi::get_crashlog_from_system_table(system_table)
    }

    #[cfg(any(all(target_os = "windows", feature = "extraction"), doc))]
    /// Searches for any Intel Crash Log logged in the Windows event log
    pub fn from_windows_event_logs(path: Option<&std::path::Path>) -> Result<Vec<Self>, Error> {
        extract::event_log::get_crashlogs_from_event_logs(path).map_err(|err| {
            log::error!("Error while accessing windows event logs: {err}");
            Error::InternalError
        })
    }

    #[cfg(any(all(target_os = "linux", feature = "extraction"), doc))]
    /// Reads the Crash Log reported through ACPI from the linux sysfs
    pub fn from_linux_sysfs() -> Result<Self, Error> {
        extract::sysfs::read_berr_from_sysfs().and_then(CrashLog::from_berr)
    }

    /// Extracts the Crash Log records from [Cper] record.
    pub(crate) fn from_cper(cper: Cper) -> Result<Self, Error> {
        let regions: Vec<Region> = cper
            .sections
            .iter()
            .filter_map(|section| Region::from_cper_section(&section.section))
            .collect();

        if regions.is_empty() {
            return Err(Error::NoCrashLogFound);
        }

        CrashLog::from_regions(regions)
    }

    /// Decodes a raw Crash Log binary.
    pub fn from_slice(s: &[u8]) -> Result<Self, Error> {
        if let Some(berr) = Berr::from_bert_file(s) {
            CrashLog::from_berr(berr)
        } else if let Some(cper) = Cper::from_slice(s) {
            CrashLog::from_cper(cper)
        } else {
            // Input file is a single Crash Log region
            CrashLog::from_regions(vec![Region::from_slice(s)?])
        }
    }

    /// Exports the [CrashLog] as a sequence of bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut berr = Berr::from_crashlog(self).to_bytes();
        let bert = Bert {
            region: 0,
            region_length: berr.len() as u32,
            ..Bert::dummy()
        };

        let mut bytes = bert.to_bytes();
        bytes.append(&mut berr);
        bytes
    }

    /// Returns the register tree representation of the Crash Log record headers.
    pub fn decode_without_cm(&self) -> Node {
        let mut root = Node::root();
        for region in self.regions.iter() {
            for record in region.records.iter() {
                root.merge(record.decode_without_cm())
            }
        }
        root
    }

    /// Returns the register tree representation of the Crash Log record content.
    #[cfg(feature = "collateral_manager")]
    pub fn decode<T: CollateralTree>(&self, cm: &mut CollateralManager<T>) -> Node {
        let mut root = Node::root();
        for region in self.regions.iter() {
            for record in region.records.iter() {
                root.merge(record.decode(cm))
            }
        }
        root
    }
}
