// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeSet, fmt};
#[cfg(feature = "std")]
use std::{collections::BTreeSet, fmt};

/// Capability of a Crash Log source
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Capability {
    /// Extraction of the Crash Log records
    Extract,
    /// On-demand Crash Log collection
    Trigger,
    /// Enabling/Disabling of Crash Log collection flow
    EnableDisable,
    /// Clearing of the Crash Log storage
    Clear,
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Extract => write!(f, "extract"),
            Self::Trigger => write!(f, "trigger"),
            Self::EnableDisable => write!(f, "enable/disable"),
            Self::Clear => write!(f, "clear"),
        }
    }
}

/// Set of all the capabilities supported by a Crash Log source
pub type Capabilities = BTreeSet<Capability>;
