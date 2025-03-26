// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::{CollateralManager, CollateralTree, ItemPath};
use crate::Error;
use crate::utils::Map;
#[cfg(not(feature = "std"))]
use alloc::string::String;
use serde::{Deserialize, Deserializer};

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
    #[serde(default, deserialize_with = "deserialize_die_ids")]
    pub die_id: Map<u8, String>,
}

fn default_variant() -> String {
    String::from("all")
}

fn deserialize_die_ids<'de, D>(deserializer: D) -> Result<Map<u8, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: Map<String, String> = Deserialize::deserialize(deserializer)?;
    Ok(map
        .into_iter()
        .filter_map(|(key, value)| {
            key.parse::<u8>()
                .inspect_err(|_| log::warn!("Invalid die ID value: {key}"))
                .ok()
                .map(|k| (k, value))
        })
        .collect())
}

impl<T: CollateralTree> CollateralManager<T> {
    pub(super) fn update_target_info(&mut self) -> Result<(), Error> {
        self.target_info.clear();
        let path = ItemPath::new(["target_info.json"]);
        for pvss in self.tree.search(&path)? {
            let res = serde_json::from_slice::<TargetInfo>(&self.tree.get(&pvss, &path)?)
                .inspect_err(|err| log::warn!("Invalid target info ({pvss}): {err}"));

            let Ok(target_info) = res else {
                continue;
            };

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
