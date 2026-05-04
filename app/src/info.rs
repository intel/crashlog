// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::InfoFormat;
use intel_crashlog::prelude::*;
use std::path::Path;

fn compact<T: CollateralTree>(cm: &CollateralManager<T>, input: &Path) -> Result<(), Error> {
    let crashlog = CrashLog::from_slice(&std::fs::read(input)?)?;

    println!("  #   Record Type      Rev.  Product  Size   Skt  Checksum  Die      ");
    println!("----- ---------------- ----- -------- ------ ---- --------- ---------");
    for (i, region) in crashlog.regions.iter().enumerate() {
        for (j, record) in region.records.iter().enumerate() {
            let product = if let Ok(product) = record.header.product(cm) {
                let variant = record.header.variant(cm).unwrap_or("all");
                format!("{product}/{variant}")
            } else {
                format!("{:#05x}", record.header.product_id())
            };

            let record_type = if let Ok(record_type) = record.header.record_type() {
                record_type.into()
            } else {
                format!("{:#04x}", record.header.version.record_type)
            };

            let checksum = record
                .checksum()
                .map_or("", |check| if check { "Valid" } else { "Invalid" });

            let die = if let Some(die_id) = record.header.die(cm) {
                die_id
            } else {
                &record
                    .header
                    .die_id()
                    .map(|die_id| die_id.to_string())
                    .unwrap_or_default()
            };

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
                die
            );
        }
    }

    Ok(())
}

fn markdown<T: CollateralTree>(cm: &CollateralManager<T>, input: &Path) -> Result<(), Error> {
    let crashlog = CrashLog::from_slice(&std::fs::read(input)?)?;

    // Column widths
    let region_idx_width = 8;
    let record_idx_width = 8;
    let record_type_width = 16;
    let revision_width = 8;
    let product_width = 14;
    let size_width = 10;
    let skt_width = 8;
    let checksum_width = 12;
    let die_width = 10;

    // Header
    println!(
        "| {1:<region_idx_width$} \
         | {2:<record_idx_width$} \
         | {3:<record_type_width$} \
         | {4:<revision_width$} \
         | {5:<product_width$} \
         | {6:<size_width$} \
         | {7:<skt_width$} \
         | {8:<checksum_width$} \
         | {9:<die_width$} \
         |\n\
         | {0:-<region_idx_width$} \
         | {0:-<record_idx_width$} \
         | {0:-<record_type_width$} \
         | {0:-<revision_width$} \
         | {0:-<product_width$} \
         | {0:-<size_width$} \
         | {0:-<skt_width$} \
         | {0:-<checksum_width$} \
         | {0:-<die_width$} \
         |",
        "",
        "Region",
        "Record",
        "Record Type",
        "Revision",
        "Product",
        "Size",
        "Socket",
        "Checksum",
        "Die",
    );

    for (i, region) in crashlog.regions.iter().enumerate() {
        for (j, record) in region.records.iter().enumerate() {
            let product = if let Ok(product) = record.header.product(cm) {
                let variant = record.header.variant(cm).unwrap_or("all");
                format!("{product}/{variant}")
            } else {
                format!("{:#05x}", record.header.product_id())
            };

            let record_type = if let Ok(record_type) = record.header.record_type() {
                record_type.into()
            } else {
                format!("{:#04x}", record.header.version.record_type)
            };

            let checksum = record
                .checksum()
                .map_or("", |check| if check { "Valid" } else { "Invalid" });

            let die = if let Some(die_id) = record.header.die(cm) {
                die_id
            } else {
                &record
                    .header
                    .die_id()
                    .map(|die_id| die_id.to_string())
                    .unwrap_or_default()
            };
            let revision = record.header.revision();
            let record_size = record.header.record_size();
            let socket_id = record.header.socket_id();

            // Populate the table
            println!(
                "| {i:<region_idx_width$} \
                 | {j:<record_idx_width$} \
                 | {record_type:<record_type_width$} \
                 | {revision:<revision_width$} \
                 | {product:<product_width$} \
                 | {record_size:<size_width$} \
                 | {socket_id:<skt_width$} \
                 | {checksum:<checksum_width$} \
                 | {die:<die_width$} \
                 |"
            );
        }
    }
    Ok(())
}

pub(crate) fn info<T, P>(cm: &CollateralManager<T>, input_files: &[P], format: InfoFormat)
where
    T: CollateralTree,
    P: AsRef<Path>,
{
    match format {
        InfoFormat::Compact => {
            for input_file in input_files {
                if input_files.len() > 1 {
                    println!("\n{}:\n", input_file.as_ref().display());
                }
                if let Err(err) = compact(cm, input_file.as_ref()) {
                    log::error!("Error: {err}")
                }
            }
        }
        InfoFormat::Markdown => {
            for input_file in input_files {
                println!("\n## `{}`\n", input_file.as_ref().display());

                if let Err(err) = markdown(cm, input_file.as_ref()) {
                    log::warn!("Error: {err}");
                    println!("\n```\n{err}\n```");
                }
            }
        }
    }
}
