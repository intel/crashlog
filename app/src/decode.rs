// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;
use std::path::Path;

pub fn decode<T: CollateralTree, O: std::io::Write>(
    cm: &mut CollateralManager<T>,
    input: &Path,
    output: O,
) -> Result<(), Error> {
    let crashlog = CrashLog::from_slice(&std::fs::read(input)?)?;
    let nodes = crashlog.decode(cm);
    Ok(serde_json::to_writer_pretty(output, &nodes)?)
}

pub fn info<T: CollateralTree>(cm: &CollateralManager<T>, input: &Path) -> Result<(), Error> {
    let crashlog = CrashLog::from_slice(&std::fs::read(input)?)?;

    // column widths for headers
    let w0 = 18; // #(Region-Record)
    let w1 = 15; // Record Type
    let w2 = 8; // Rev.
    let w3 = 14; // Product
    let w4 = 10; // Size
    let w5 = 8; // Skt
    let w6 = 12; // Checksum
    let w7 = 10; // Die

    // update # to #(Region-Record)
    let header = format!(
        "| {:<w0$} | {:<w1$} | {:<w2$} | {:<w3$} | {:<w4$} | {:<w5$} | {:<w6$} | {:<w7$} |",
        "#(Region-Record)",
        "Record Type",
        "Rev.",
        "Product",
        "Size",
        "Skt",
        "Checksum",
        "Die",
        w0 = w0,
        w1 = w1,
        w2 = w2,
        w3 = w3,
        w4 = w4,
        w5 = w5,
        w6 = w6,
        w7 = w7
    );
    println!("{}", &header);
    // separator line for table headers
    println!("{}", "-".repeat(header.len()));

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

            // populate the table
            println!(
                "| {:<w0$} | {:<w1$} | {:<w2$} | {:<w3$} | {:<w4$} | {:<w5$} | {:<w6$} | {:<w7$} |",
                format!("{}-{}", i, j),
                record_type,
                record.header.revision(),
                product,
                record.header.record_size(),
                record.header.socket_id(),
                checksum,
                die,
                w0 = w0,
                w1 = w1,
                w2 = w2,
                w3 = w3,
                w4 = w4,
                w5 = w5,
                w6 = w6,
                w7 = w7
            );
        }
    }

    Ok(())
}
