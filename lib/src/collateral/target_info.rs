// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::{CollateralManager, CollateralTree, ItemPath};
use crate::Error;
use crate::utils::Map;
#[cfg(not(feature = "std"))]
use alloc::string::String;
use serde::Deserialize;

/// Stores various product information
#[derive(Debug, Deserialize)]
pub struct TargetInfo {
    /// Product TLA
    pub product: String,
    /// Product ID used in the Crash Log headers
    pub product_id: String,
    /// Product variant
    #[serde(default = "default_variant")]
    pub variant: String,
    /// Die IDs/names
    #[serde(default)]
    pub die_id: Map<String, String>,
}

fn default_variant() -> String {
    String::from("all")
}

impl<T: CollateralTree> CollateralManager<T> {
    pub(super) fn update_target_info(&mut self) -> Result<(), Error> {
        self.target_info.clear();
        let path = ItemPath::new(["target_info.json"]);
        for pvss in self.tree.search(&path)? {
            let target_info: TargetInfo = serde_json::from_slice(&self.tree.get(&pvss, &path)?)?;

            let product_id = if target_info.product_id.starts_with("0x") {
                u32::from_str_radix(&target_info.product_id[2..], 16)
            } else {
                target_info.product_id.parse()
            };

            if let Ok(product_id) = product_id {
                log::trace!("Loading target info: {target_info:?}");
                self.target_info.insert(product_id, target_info);
            }
        }
        Ok(())
    }
}
