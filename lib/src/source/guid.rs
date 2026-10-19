// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeSet, fmt};
#[cfg(feature = "std")]
use std::{collections::BTreeSet, fmt};

/// Global unique identifier of a Crash Log Source
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Guid {
    id: u32,
}

impl Guid {
    /// Create a GUID from a 32 bit value
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    /// Returns the 32 bit numeric value of the GUID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the name of the GUID in case it is defined
    pub fn name(&self) -> Option<&'static str> {
        match self.id {
            0x4E2F8210 => Some("BMG_X2_OOBMSM"),
            0x0E2F8200 => Some("BMG_X2_PUNIT"),
            0x4E2F8310 => Some("BMG_G31_OOBMSM"),
            0x0E2F8300 => Some("BMG_G31_PUNIT"),
            0x0E2FA200 => Some("CRI_PUNIT"),
            0x4E2FA230 => Some("CRI_OOBMSM"),
            _ => None,
        }
    }
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = self.name() {
            return write!(f, "{}", name);
        }

        write!(f, "0x{:08X}", self.id)
    }
}

/// Set of Guids supported by a Crash Log source
pub type Guids = BTreeSet<Guid>;
