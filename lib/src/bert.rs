// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

pub mod berr;
#[cfg(test)]
mod tests;

use acpi::{
    AcpiTable,
    sdt::{SdtHeader, Signature},
};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
pub use berr::Berr;

/// Boot Error Record Table (ACPI 6.3 - Table 18-381)
#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct Bert {
    pub header: SdtHeader,
    pub region_length: u32,
    pub region: u64,
}

unsafe impl AcpiTable for Bert {
    const SIGNATURE: Signature = Signature::BERT;

    fn header(&self) -> &SdtHeader {
        &self.header
    }
}

impl Bert {
    pub(super) fn dummy() -> Bert {
        Bert {
            header: SdtHeader {
                signature: Signature::BERT,
                length: 0,
                revision: 0,
                checksum: 0,
                oem_id: [0; 6],
                oem_table_id: [0; 8],
                oem_revision: 0,
                creator_id: 0,
                creator_revision: 0,
            },
            region_length: 0,
            region: 0,
        }
    }

    #[cfg(all(target_os = "uefi", feature = "extraction"))]
    unsafe fn raw_berr_from_phys_mem(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.region as *const _, self.region_length as usize) }
    }

    /// Reads the [Berr] associated to this [Bert] from physical memory.
    ///
    /// # Safety
    ///
    /// This function accesses the [Berr] stored in physical memory pointed by the [Bert] ACPI
    /// table. This must only be called from kernel mode.
    ///
    /// # Error
    ///
    /// If the [Berr] is not valid, None is returned.
    #[cfg(all(target_os = "uefi", feature = "extraction"))]
    pub unsafe fn berr_from_phys_mem(&self) -> Option<Berr> {
        Berr::from_slice(unsafe { self.raw_berr_from_phys_mem() })
    }

    pub(super) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"BERT");
        bytes.extend_from_slice(&self.header.length.to_le_bytes());
        bytes.push(self.header.revision);
        bytes.push(self.header.checksum);
        bytes.extend_from_slice(&self.header.oem_id);
        bytes.extend_from_slice(&self.header.oem_table_id);
        bytes.extend_from_slice(&self.header.oem_revision.to_le_bytes());
        bytes.extend_from_slice(&self.header.creator_id.to_le_bytes());
        bytes.extend_from_slice(&self.header.creator_revision.to_le_bytes());
        bytes.extend_from_slice(&self.region_length.to_le_bytes());
        bytes.extend_from_slice(&self.region.to_le_bytes());
        bytes
    }
}
