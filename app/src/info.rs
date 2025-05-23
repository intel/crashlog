// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;
use std::path::Path;

pub fn compact<T: CollateralTree>(cm: &CollateralManager<T>, input: &Path) -> Result<(), Error> {
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
                format!("{:#02x}", record.header.version.record_type)
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
