// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(not(feature = "std"))]
use alloc::{fmt, string::String};
#[cfg(feature = "std")]
use std::{fmt, path::PathBuf};

/// A tuple of 4 strings that uniquely identifies a product.
///
/// Undefined elements of the tuples must be set to "all". For example: `XYZ/all/all/all`
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PVSS {
    /// Product TLA
    pub product: String,
    /// Product variant
    pub variant: String,
    /// Product stepping
    pub stepping: String,
    /// Security level
    pub security: String,
}

impl Default for PVSS {
    fn default() -> Self {
        PVSS {
            product: "all".into(),
            variant: "all".into(),
            stepping: "all".into(),
            security: "green".into(),
        }
    }
}

#[cfg(feature = "std")]
impl From<&PVSS> for PathBuf {
    fn from(pvss: &PVSS) -> Self {
        [&pvss.product, &pvss.variant, &pvss.stepping, &pvss.security]
            .iter()
            .filter(|p| **p != "." && **p != "..")
            .collect()
    }
}

impl fmt::Display for PVSS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            self.product, self.variant, self.stepping, self.security
        )?;
        Ok(())
    }
}
