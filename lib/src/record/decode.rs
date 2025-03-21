// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Record;
#[cfg(feature = "collateral_manager")]
use crate::collateral::{CollateralManager, CollateralTree};
use crate::error::Error;
use crate::header::record_types;
use crate::node::Node;
use crate::node::NodeType;
#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, str, string::String, vec::Vec};
use log::debug;
#[cfg(feature = "std")]
use std::str;

const DELIMITER: char = ';';

#[derive(Default, Debug)]
struct DecodeDefinitionEntry {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub description: String,
}

impl Record {
    fn read_field(&self, offset: usize, size: usize) -> Option<u64> {
        if size > 64 {
            // Large fields don't need to be decoded.
            return None;
        }

        let mut value = 0;
        let mut bit = 0;

        while bit < size {
            let chunk_size = 8;
            let chunk = (offset + bit) / chunk_size;
            if chunk >= self.data.len() {
                return None;
            }

            let bit_offset = (offset + bit) % chunk_size;
            let mask = (1 << (size - bit).min(chunk_size)) - 1;
            value |= ((self.data[chunk] as u64 >> bit_offset) & mask) << bit;
            bit += chunk_size - bit_offset;
        }

        Some(value)
    }

    pub fn decode_with_csv(&self, layout: &[u8], offset: usize) -> Result<Node, Error> {
        let mut root = Node::root();
        let record_root = if let Some(custom_root) = self.header.get_root_path() {
            root.create_hierarchy(&custom_root)
        } else {
            &mut root
        };

        let csv = str::from_utf8(layout)?;
        let mut columns = Vec::new();
        let mut current_path = Vec::new();

        for (i, line) in csv.lines().enumerate() {
            if i == 0 {
                columns = line.split(DELIMITER).collect();
                debug!("CSV columns: {columns:?}");
                continue;
            }

            let mut entry = DecodeDefinitionEntry::default();

            for (i, field) in line.split(DELIMITER).enumerate() {
                if let Some(column) = columns.get(i) {
                    match *column {
                        "name" => entry.name = field.into(),
                        "offset" => entry.offset = field.parse()?,
                        "size" => entry.size = field.parse()?,
                        "description" => entry.description = field.into(),
                        _ => (),
                    }
                }
            }

            if entry.name.is_empty() {
                continue;
            }

            let mut segments = entry.name.split(".");
            let Some(top) = segments.next() else {
                continue;
            };

            if !top.is_empty() {
                // Absolute path
                current_path.clear();
                current_path.push(top.to_owned());

                if record_root.get(top).is_none() {
                    // Top-level is assumed to be the record name
                    record_root.add(Node::record(top));
                }
            }

            for segment in segments {
                if segment.is_empty() {
                    let _ = current_path.pop();
                } else {
                    current_path.push(segment.to_owned());
                }
            }

            let node = record_root.create_hierarchy_from_iter(&current_path);
            node.description = entry.description;
            if let Some(value) = self.read_field(offset * 8 + entry.offset, entry.size) {
                node.kind = NodeType::Field { value }
            }
        }
        Ok(root)
    }

    pub fn basic_decode(&self) -> Node {
        let mut record = Node::record(self.header.record_type().unwrap_or("record"));
        record.add(Node::from(&self.header));

        let mut root = Node::root();
        let record_root = if let Some(custom_root) = self.header.get_root_path() {
            root.create_hierarchy(&custom_root)
        } else {
            &mut root
        };

        record_root.add(record);
        root
    }

    #[cfg(feature = "collateral_manager")]
    pub fn decode_with_decode_def<T: CollateralTree>(
        &self,
        cm: &mut CollateralManager<T>,
        decode_def: &str,
        offset: usize,
    ) -> Result<Node, Error> {
        let paths = self.header.decode_definitions_paths(cm)?;

        for mut path in paths {
            path.push(decode_def);
            let Ok(layout) = cm.get_item_with_header(&self.header, path) else {
                continue;
            };
            return self.decode_with_csv(layout, offset);
        }

        Err(Error::MissingDecodeDefinitions(self.header.version.clone()))
    }

    #[cfg(feature = "collateral_manager")]
    pub fn decode<T: CollateralTree>(&self, cm: &mut CollateralManager<T>) -> Result<Node, Error> {
        if let record_types::PCORE | record_types::ECORE = self.header.version.record_type {
            return self.decode_as_core_record(cm);
        }

        self.decode_with_decode_def(cm, "layout.csv", 0)
    }
}
