// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::InfoFormat;
use super::table::{Alignment, Row, Table};
use intel_crashlog::prelude::*;
use std::path::Path;

fn compact<T: CollateralTree>(cm: &CollateralManager<T>, input: &Path) -> Result<(), Error> {
    let crashlog = CrashLog::from_slice(&std::fs::read(input)?)?;

    let mut table = Table::from([
        "  # ",
        "Record Type",
        "Rev.",
        "Product",
        "Size",
        "Skt",
        "Checksum",
        "Die",
    ]);

    table.columns[0].alignment = Alignment::Center;
    table.columns[2].alignment = Alignment::Right;
    table.columns[4].alignment = Alignment::Right;
    table.columns[5].alignment = Alignment::Right;

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

            table.append_row(Row::from([
                format!("{i}-{j}"),
                record_type,
                record.header.revision().to_string(),
                product,
                record.header.record_size().to_string(),
                record.header.socket_id().to_string(),
                checksum.to_string(),
                die.to_string(),
            ]));
        }
    }

    table.render();

    if !crashlog.metadata.extra_cper_sections.is_empty() {
        println!();

        let mut table = Table::from(["#", "CPER Section GUID", "Length", "Description"]);

        for (i, section) in crashlog.metadata.extra_cper_sections.iter().enumerate() {
            table.append_row(Row::from([
                i.to_string(),
                section.guid().to_string(),
                section.len().to_string(),
                section.to_string(),
            ]));
        }

        table.render();
    }

    Ok(())
}

fn markdown<T: CollateralTree>(cm: &CollateralManager<T>, input: &Path) -> Result<(), Error> {
    let crashlog = CrashLog::from_slice(&std::fs::read(input)?)?;

    println!("### Crash Log Records\n");

    let mut table = Table::from([
        "Region",
        "Record",
        "Record Type",
        "Revision",
        "Product",
        "Size",
        "Socket",
        "Checksum",
        "Die",
    ]);

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
                .map_or("", |check| if check { "Valid" } else { "Invalid" })
                .to_string();

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

            table.append_row(Row::from([
                i.to_string(),
                j.to_string(),
                record_type,
                revision.to_string(),
                product,
                record_size.to_string(),
                socket_id.to_string(),
                checksum,
                die.to_string(),
            ]));
        }
    }

    table.render_markdown();

    if !crashlog.metadata.extra_cper_sections.is_empty() {
        println!("\n### Extra CPER Sections\n");

        let mut table = Table::from(["#", "CPER Section GUID", "Length", "Description"]);

        for (i, section) in crashlog.metadata.extra_cper_sections.iter().enumerate() {
            table.append_row(Row::from([
                i.to_string(),
                section.guid().to_string(),
                section.len().to_string(),
                section.to_string(),
            ]));
        }

        table.render_markdown();
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
