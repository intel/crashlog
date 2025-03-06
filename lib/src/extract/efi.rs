// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use crate::bert::Bert;
use crate::metadata;
use crate::{CrashLog, Error};
use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use alloc::string::ToString;
use core::ptr::NonNull;
use uefi_raw::table::system::SystemTable;

#[derive(Clone)]
struct IdentityMapped;

impl AcpiHandler for IdentityMapped {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        unsafe {
            PhysicalMapping::new(
                physical_address,
                NonNull::new(physical_address as *mut _).unwrap(),
                size,
                size,
                Self,
            )
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

fn find_acpi_tables() -> Option<AcpiTables<IdentityMapped>> {
    uefi::system::with_config_table(|config_tables| {
        for config_table in config_tables {
            if config_table.guid != uefi::table::cfg::ACPI2_GUID {
                continue;
            }

            let acpi_tables = unsafe {
                AcpiTables::<IdentityMapped>::from_rsdp(
                    IdentityMapped,
                    config_table.address as usize,
                )
            };

            match acpi_tables {
                Ok(acpi_tables) => {
                    log::info!("Found valid ACPI tables.");
                    return Some(acpi_tables);
                }
                Err(error) => {
                    log::warn!("Found invalid ACPI table: {:?}", error);
                }
            }
        }

        log::error!("Failed to find ACPI tables.");
        None
    })
}

pub fn find_bert() -> Result<Bert, Error> {
    let tables = find_acpi_tables().ok_or(Error::NoCrashLogFound)?;
    tables
        .find_table::<Bert>()
        .map(|mapping| mapping.clone())
        .map_err(|err| {
            log::info!("Could not find the ACPI BERT table: {:?}", err);
            Error::NoCrashLogFound
        })
}

pub(crate) fn get_crashlog_from_system_table(
    system_table: Option<NonNull<SystemTable>>,
) -> Result<CrashLog, Error> {
    if let Some(system_table) = system_table {
        unsafe { uefi::table::set_system_table(system_table.as_ptr()) }
    }

    let mut crashlog = find_bert()
        .and_then(|bert| {
            unsafe { bert.berr_from_phys_mem() }.ok_or(Error::InvalidBootErrorRecordRegion)
        })
        .and_then(CrashLog::from_berr)?;

    crashlog.metadata = metadata::Metadata {
        computer: Some("efi".to_string()),
        time: uefi::runtime::get_time()
            .map(|time| metadata::Time {
                year: time.year(),
                month: time.month(),
                day: time.day(),
                hour: time.hour(),
                minute: time.minute(),
            })
            .inspect_err(|err| log::warn!("Cannot get time: {err}"))
            .ok(),
    };

    Ok(crashlog)
}
