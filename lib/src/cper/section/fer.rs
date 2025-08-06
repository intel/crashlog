// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use uguid::Guid;

use crate::region::Region;

pub mod guids {
    //! GUIDs used to identify the type of the Firmware Error Record payload
    use uguid::Guid;
    pub const RECORD_ID_CRASHLOG: Guid = uguid::guid!("8f87f311-c998-4d9e-a0c4-6065518c4f6d");
}

pub const HEADER_REV1_SIZE: usize = 16;
pub const HEADER_REV2_SIZE: usize = 32;

/// UEFI 2.10 N.2.10. Firmware Error Record Reference Header
#[derive(Debug, Clone, Default)]
pub struct FirmwareErrorRecordHeader {
    pub error_type: u8,
    pub revision: u8,
    pub record_identifier: u64,
    pub guid: Guid,
}

/// UEFI 2.10 N.2.10. Firmware Error Record Reference
#[derive(Debug, Clone, Default)]
pub struct FirmwareErrorRecord {
    pub header: FirmwareErrorRecordHeader,
    pub payload: Vec<u8>,
}

impl FirmwareErrorRecordHeader {
    /// Parses the section header from a slice.
    pub fn from_slice(s: &[u8]) -> Option<Self> {
        let revision = *s.get(1)?;
        Some(Self {
            error_type: *s.first()?,
            revision,
            record_identifier: u64::from_le_bytes(s.get(8..16)?.try_into().ok()?),
            guid: if revision >= 2 {
                Guid::from_bytes(s.get(16..32)?.try_into().ok()?)
            } else {
                Guid::ZERO
            },
        })
    }

    /// Converts the section header into a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.error_type);
        bytes.push(self.revision);
        bytes.extend_from_slice(&[0; 6]);
        bytes.extend_from_slice(&self.record_identifier.to_le_bytes());
        bytes.extend_from_slice(&self.guid.to_bytes());
        bytes
    }

    /// Returns the size of the section in bytes
    pub fn len(&self) -> usize {
        if self.revision >= 2 {
            HEADER_REV2_SIZE
        } else {
            HEADER_REV1_SIZE
        }
    }
}

impl FirmwareErrorRecord {
    /// Parses the section from a slice.
    pub fn from_slice(s: &[u8]) -> Option<FirmwareErrorRecord> {
        let header = FirmwareErrorRecordHeader::from_slice(s)?;
        let payload = Vec::from(s.get(header.len()..)?);
        Some(Self { header, payload })
    }

    /// Wraps a Crash Log region into a Firmware Error Record
    pub fn from_crashlog_region(region: &Region) -> Self {
        Self {
            header: FirmwareErrorRecordHeader {
                error_type: 2,
                revision: 2,
                guid: guids::RECORD_ID_CRASHLOG,
                ..FirmwareErrorRecordHeader::default()
            },
            payload: region.to_bytes(),
        }
    }

    /// Converts the section into a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.header.to_bytes();
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}
