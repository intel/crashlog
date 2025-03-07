// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

//! Management of the product-specific collateral required to decode the Crash Log records.
//!
//! The term 'collateral tree' used in this module defines the data structure that stores the
//! project-specific collateral files.
//! Each collateral file is indexed based on the these two keys:
//! - [`PVSS`]: uniquely identifies a product.
//! - [`ItemPath`]: defines the location of the item within a given [`PVSS`]

#[cfg(feature = "embedded_collateral_tree")]
mod embedded;
#[cfg(feature = "fs_collateral_tree")]
mod fs;
mod path;
mod pvss;
mod target_info;

use crate::Error;
use crate::header::Header;
use crate::utils::Map;
#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

#[cfg(feature = "embedded_collateral_tree")]
pub use embedded::EmbeddedTree;
#[cfg(feature = "fs_collateral_tree")]
pub use fs::FileSystemTree;
pub use path::ItemPath;
pub use pvss::PVSS;
pub use target_info::TargetInfo;

#[derive(Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct ItemIndex {
    pvss: PVSS,
    path: ItemPath,
}

/// A trait representing a data structure that provides a direct access to the product-specific
/// collateral files.
pub trait CollateralTree {
    /// Returns the content of an item in the collateral tree.
    fn get(&self, pvss: &PVSS, path: &ItemPath) -> Result<Vec<u8>, Error>;
    /// Returns a list of all the `PVSS` that have an item defined at the given `path`.
    fn search(&self, path: &ItemPath) -> Result<Vec<PVSS>, Error>;
}

/// Manages the product-specific collateral files required to decode the Crash Log records.
#[derive(Default)]
pub struct CollateralManager<T: CollateralTree> {
    tree: T,
    items: Map<ItemIndex, Vec<u8>>,
    /// Maps the Crash Log product IDs into a data structure that stores various information
    /// about the associated product.
    pub target_info: Map<u32, TargetInfo>,
}

impl<T: CollateralTree> CollateralManager<T> {
    /// Creates a collateral manager for a given collateral `tree`.
    pub fn new(tree: T) -> Result<Self, Error> {
        let mut cm = Self {
            tree,
            items: Map::default(),
            target_info: Map::default(),
        };
        cm.update_target_info()?;
        Ok(cm)
    }

    /// Returns the content of an item from the collateral tree using the [`PVSS`] of the target.
    ///
    /// ```
    /// use intel_crashlog::prelude::*;
    /// use intel_crashlog::collateral::PVSS;
    ///
    /// let mut cm = CollateralManager::embedded_tree().unwrap();
    /// let pvss = PVSS {
    ///     product: "XYZ".into(),
    ///     ..PVSS::default()
    /// };
    /// assert!(cm.get_item_with_pvss(pvss, "target_info.json").is_ok());
    /// ```
    pub fn get_item_with_pvss(
        &mut self,
        pvss: PVSS,
        path: impl Into<ItemPath>,
    ) -> Result<&[u8], Error> {
        let index = ItemIndex {
            pvss,
            path: path.into(),
        };

        if !self.items.contains_key(&index) {
            self.fetch_item(&index)?;
        }

        self.items
            .get(&index)
            .map(|i| i.as_ref())
            .ok_or(Error::InternalError)
    }

    fn fetch_item(&mut self, index: &ItemIndex) -> Result<(), Error> {
        let security_levels = ["red", "white", "green", "all"];

        if let Some(i) = security_levels
            .iter()
            .position(|s| *s == index.pvss.security)
        {
            for security in &security_levels[i..] {
                let pvss = PVSS {
                    security: security.to_string(),
                    ..index.pvss.clone()
                };
                match self.tree.get(&pvss, &index.path) {
                    Ok(item) => {
                        self.items.insert(index.clone(), item);
                        return Ok(());
                    }
                    Err(Error::MissingCollateral(_, item)) => {
                        log::debug!("No {security} {item} defined")
                    }
                    Err(err) => log::warn!("Unexpected error while fetching item: {err}"),
                }
            }
        }

        Err(Error::MissingCollateral(
            index.pvss.clone(),
            index.path.clone(),
        ))
    }

    /// Returns the content of an item from the collateral tree using the [`PVSS`] of the target.
    ///
    /// ```
    /// use intel_crashlog::prelude::*;
    /// use intel_crashlog::collateral::PVSS;
    ///
    /// let mut cm = CollateralManager::embedded_tree().unwrap();
    /// let pvss = PVSS {
    ///     product: "XYZ".into(),
    ///     ..PVSS::default()
    /// };
    /// assert!(cm.get_item_with_pvss(pvss, "target_info.json").is_ok());
    /// ```
    pub fn get_item_with_pvs(
        &mut self,
        pvss: PVSS,
        path: impl Into<ItemPath>,
    ) -> Result<&[u8], Error> {
        let index = ItemIndex {
            pvss,
            path: path.into(),
        };

        if !self.items.contains_key(&index) {
            let item = self.tree.get(&index.pvss, &index.path)?;
            self.items.insert(index.clone(), item);
        }

        self.items
            .get(&index)
            .map(|i| i.as_ref())
            .ok_or(Error::InternalError)
    }

    /// Returns the content of an item from the collateral tree using the Crash Log header.
    ///
    /// ```
    /// use intel_crashlog::prelude::*;
    ///
    /// let mut cm = CollateralManager::embedded_tree().unwrap();
    /// let data = vec![0x08, 0xa1, 0x07, 0x3e, 0x2, 0x0, 0x0, 0x0];
    /// let header = Header::from_slice(&data).unwrap().unwrap();
    /// assert!(cm.get_item_with_header(&header, "target_info.json").is_ok());
    /// ```
    pub fn get_item_with_header(
        &mut self,
        header: &Header,
        path: impl Into<ItemPath>,
    ) -> Result<&[u8], Error> {
        self.get_item_with_pvss(header.pvss(self)?, path)
    }
}
