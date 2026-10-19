// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(not(feature = "std"))]
use alloc::fmt;
#[cfg(feature = "std")]
use std::fmt;

/// PMT Global unique identifier of a Crash Log Source
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PmtGuid {
    id: u32,
}

impl PmtGuid {
    /// Create a PMT GUID from a 32 bit value
    pub const fn new(id: u32) -> Self {
        Self { id }
    }

    /// Returns the 32 bit numeric value of the PMT GUID
    pub const fn id(&self) -> u32 {
        self.id
    }

    /// Returns the name of the PMT GUID in case it is defined
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

impl From<u32> for PmtGuid {
    fn from(id: u32) -> Self {
        Self::new(id)
    }
}

impl From<PmtGuid> for u32 {
    fn from(guid: PmtGuid) -> Self {
        guid.id()
    }
}

impl fmt::Display for PmtGuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = self.name() {
            return f.write_str(name);
        }

        write!(f, "0x{:08X}", self.id)
    }
}
