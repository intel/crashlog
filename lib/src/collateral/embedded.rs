// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::{CollateralManager, CollateralTree, ItemPath, PVSS};
use crate::Error;
use crate::utils::Map;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Provides access to a collateral tree embedded in the library.
#[derive(Default)]
pub struct EmbeddedTree {
    registry: Map<PVSS, Map<ItemPath, &'static [u8]>>,
}

impl EmbeddedTree {
    fn new() -> Self {
        let mut tree = Self::default();
        include!(concat!(env!("OUT_DIR"), "/embedded_collateral_tree.rs"));
        tree
    }

    fn insert_item(
        &mut self,
        product: &str,
        variant: &str,
        stepping: &str,
        security: &str,
        path: &str,
        content: &'static [u8],
    ) {
        let pvss = PVSS {
            product: product.into(),
            variant: variant.into(),
            stepping: stepping.into(),
            security: security.into(),
        };
        if !self.registry.contains_key(&pvss) {
            self.registry.insert(pvss.clone(), Map::default());
        }

        if let Some(items) = self.registry.get_mut(&pvss) {
            items.insert(path.parse().unwrap(), content);
        }
    }
}

impl CollateralTree for EmbeddedTree {
    fn get(&self, pvss: &PVSS, item: &ItemPath) -> Result<Vec<u8>, Error> {
        self.registry
            .get(pvss)
            .ok_or_else(|| Error::MissingCollateral(pvss.clone(), item.clone()))
            .and_then(|items| {
                items
                    .get(item)
                    .ok_or_else(|| Error::MissingCollateral(pvss.clone(), item.clone()))
            })
            .map(|data| Vec::from(*data))
    }

    fn search(&self, item: &ItemPath) -> Result<Vec<PVSS>, Error> {
        let mut hits = Vec::new();

        for (pvss, items) in self.registry.iter() {
            if items.contains_key(item) {
                hits.push(pvss.clone());
            }
        }

        Ok(hits)
    }
}

impl CollateralManager<EmbeddedTree> {
    /// Creates a [`CollateralManager`] that uses a collateral tree embedded in the library.
    /// This allows the collateral items to be used without requiring any file system access.
    /// The collateral items will be loaded in memory alongside the library code.
    ///
    /// # Examples
    ///
    /// ```
    /// use intel_crashlog::prelude::*;
    ///
    /// let collateral_manager = CollateralManager::embedded_tree();
    /// assert!(collateral_manager.is_ok());
    /// ```
    pub fn embedded_tree() -> Result<Self, Error> {
        Self::new(EmbeddedTree::new())
    }
}
