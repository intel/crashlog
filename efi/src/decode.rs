// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use crate::pager::Pager;
use alloc::format;
use intel_crashlog::prelude::*;
use log::{error, warn};
use uefi::fs::{FileSystem, Path};
use uefi::prelude::*;
use uefi::println;

fn read_crashlog_from_file(input_path: &Path) -> Result<CrashLog, uefi::Error> {
    let image = uefi::boot::image_handle();
    let fs_protocol = uefi::boot::get_image_file_system(image)
        .inspect_err(|err| warn!("Failed to get FileSystem protocol: {err}"))?;
    let mut fs = FileSystem::new(fs_protocol);
    let data = fs.read(input_path).map_err(|err| {
        error!("Failed to read file: {err}");
        match err {
            uefi::fs::Error::Io(err) => err.uefi_error,
            _ => uefi::Error::from(Status::INVALID_PARAMETER),
        }
    })?;

    CrashLog::from_slice(&data).map_err(|err| {
        warn!("Cannot decode Crash Log: {err}");
        uefi::Error::from(Status::INVALID_PARAMETER)
    })
}

pub fn decode(input_path: &Path) -> Result<(), uefi::Error> {
    let crashlog = read_crashlog_from_file(input_path)?;

    let nodes = match CollateralManager::embedded_tree() {
        Ok(mut cm) => crashlog.decode(&mut cm),
        Err(_) => crashlog.decode_without_cm(),
    };

    let serialized_data =
        serde_json::to_string_pretty(&nodes).expect("Cannot serialize Crash Log nodes");

    Pager::display(&serialized_data)
}

pub fn info(input_path: &Path) -> Result<(), uefi::Error> {
    let crashlog = read_crashlog_from_file(input_path)?;

    let Ok(cm) = CollateralManager::embedded_tree() else {
        return Err(uefi::Error::from(Status::NOT_READY));
    };

    for (i, region) in crashlog.regions.iter().enumerate() {
        for (j, record) in region.records.iter().enumerate() {
            let product = if let Ok(product) = record.header.product(&cm) {
                let variant = record.header.variant(&cm).unwrap_or("all");
                format!("{product}/{variant}")
            } else {
                format!("{:#05x}", record.header.product_id())
            };

            let record_type = if let Ok(record_type) = record.header.record_type() {
                record_type.into()
            } else {
                format!("{:#02x}", record.header.version.record_type)
            };

            let checksum = record
                .checksum()
                .map_or("", |check| if check { "Valid" } else { "Invalid" });

            println!(
                "{:>2}-{:<2} {:<16} {:>5} {:<8} {:>6} {:>4} {:<9} {}",
                i,
                j,
                record_type,
                record.header.revision(),
                product,
                record.header.record_size(),
                record.header.socket_id(),
                checksum,
                record.header.die(&cm).unwrap_or(""),
            );
        }
    }
    Ok(())
}
