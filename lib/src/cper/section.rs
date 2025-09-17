// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

pub mod fer;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use super::descr::CperSectionDescriptor;
use crate::region::Region;
use fer::FirmwareErrorRecord;
use uguid::Guid;

pub mod guids {
    //! GUIDs of the standard CPER sections
    use uguid::Guid;
    pub const FW_ERROR_RECORD: Guid = uguid::guid!("81212a96-09ed-4996-9471-8d729c8e69ed");
}

/// One of the CPER section bodies defined in the UEFI 2.10 Specifications (N.2)
#[derive(Clone)]
pub enum CperSectionBody {
    FirmwareErrorRecord(FirmwareErrorRecord),
    Unknown(Guid, Vec<u8>),
}

impl CperSectionBody {
    /// Parses a section from a slice.
    pub fn from_slice(guid: uguid::Guid, s: &[u8]) -> Option<Self> {
        Some(match guid {
            guids::FW_ERROR_RECORD => {
                CperSectionBody::FirmwareErrorRecord(fer::FirmwareErrorRecord::from_slice(s)?)
            }
            _ => CperSectionBody::Unknown(guid, Vec::from(s)),
        })
    }

    /// Returns the GUID associated with section type
    pub fn guid(&self) -> Guid {
        match self {
            CperSectionBody::FirmwareErrorRecord(_) => guids::FW_ERROR_RECORD,
            CperSectionBody::Unknown(guid, _) => *guid,
        }
    }

    /// Returns the expected size of the section in bytes.
    pub fn len(&self) -> usize {
        match self {
            CperSectionBody::FirmwareErrorRecord(fer) => fer.header.len() + fer.payload.len(),
            CperSectionBody::Unknown(_, data) => data.len(),
        }
    }

    /// Converts the section into a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let bytes = match self {
            CperSectionBody::FirmwareErrorRecord(fer) => fer.to_bytes(),
            CperSectionBody::Unknown(_, data) => data.clone(),
        };

        debug_assert_eq!(bytes.len(), self.len());
        bytes
    }
}

/// The descriptor and the body of the CPER Section.
pub struct CperSection {
    pub descriptor: CperSectionDescriptor,
    pub body: CperSectionBody,
}

impl CperSection {
    /// Create a CPER Section from a Crash Log record.
    pub fn from_crashlog_region(region: &Region) -> CperSection {
        Self::from_body(CperSectionBody::FirmwareErrorRecord(
            fer::FirmwareErrorRecord::from_crashlog_region(region),
        ))
    }

    /// Create a CPER Section from a CPER section body and automatically populated the associated
    /// descriptor fields.
    pub fn from_body(body: CperSectionBody) -> Self {
        Self {
            descriptor: CperSectionDescriptor {
                section_type: body.guid(),
                section_length: body.len() as u32,
                ..CperSectionDescriptor::default()
            },
            body,
        }
    }

    /// Converts the section body into a byte vector. The size of the vector matches the section
    /// length specified in the descriptor.
    pub fn body_bytes(&self) -> Vec<u8> {
        let mut bytes = self.body.to_bytes();
        bytes.resize(self.descriptor.section_length as usize, 0);
        bytes
    }
}
