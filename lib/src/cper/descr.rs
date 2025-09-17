// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use super::revision::Revision;
use uguid::Guid;

/// cbindgen:ignore
pub const SECTION_DESCRIPTOR_SIZE: usize = 72;

/// cbindgen:ignore
mod validation {
    pub const FRU_ID: u8 = 1;
    pub const FRU_STRING: u8 = 2;
}

/// cbindgen:ignore
mod flags {
    pub const PRIMARY: u32 = 1 << 0;
    pub const CONTAINMENT_WARNING: u32 = 1 << 1;
    pub const RESET: u32 = 1 << 2;
    pub const ERROR_THRESHOLD_EXCEEDED: u32 = 1 << 3;
    pub const RESOURCE_NOT_ACCESSIBLE: u32 = 1 << 4;
    pub const LATENT_ERROR: u32 = 1 << 5;
    pub const PROPAGATED: u32 = 1 << 6;
    pub const OVERFLOW: u32 = 1 << 7;
}

#[derive(Clone, Copy, Default)]
#[repr(u32)]
pub enum SectionSeverity {
    Recoverable = 0,
    Fatal = 1,
    Corrected = 2,
    #[default]
    Informational = 3,
}

impl From<u32> for SectionSeverity {
    fn from(value: u32) -> Self {
        match value {
            0 => SectionSeverity::Recoverable,
            1 => SectionSeverity::Fatal,
            2 => SectionSeverity::Corrected,
            _ => SectionSeverity::Informational,
        }
    }
}

/// UEFI 2.10 N.2.2 Section Descriptor.
#[derive(Clone)]
pub struct CperSectionDescriptor {
    pub section_offset: u32,
    pub section_length: u32,
    pub revision: Revision,
    pub validation_bits: u8,
    pub flags: u32,
    pub section_type: Guid,
    pub fru_id: Option<Guid>,
    pub section_severity: SectionSeverity,
    pub fru_text: Option<[u8; 20]>,
}

impl Default for CperSectionDescriptor {
    fn default() -> Self {
        Self {
            section_offset: 0,
            section_length: 0,
            revision: Revision::new(1, 0),
            validation_bits: 0,
            flags: 0,
            section_type: Guid::default(),
            fru_id: None,
            section_severity: SectionSeverity::default(),
            fru_text: None,
        }
    }
}

impl CperSectionDescriptor {
    /// Parses the CPER Section Descriptor stored in a byte slice.
    pub fn from_slice(s: &[u8]) -> Option<Self> {
        let revision = Revision::from_slice(s.get(8..10)?)?;
        if revision.major != 1 {
            log::warn!("Unsupported CPER Section Descriptor revision: {revision}");
        }

        let validation_bits = *s.get(10)?;

        Some(CperSectionDescriptor {
            section_offset: u32::from_le_bytes(s.get(0..4)?.try_into().ok()?),
            section_length: u32::from_le_bytes(s.get(4..8)?.try_into().ok()?),
            revision,
            validation_bits,
            flags: u32::from_le_bytes(s.get(12..16)?.try_into().ok()?),
            section_type: Guid::from_bytes(s.get(16..32)?.try_into().ok()?),
            fru_id: if validation_bits & validation::FRU_ID != 0 {
                Some(Guid::from_bytes(s.get(32..48)?.try_into().ok()?))
            } else {
                None
            },
            section_severity: u32::from_le_bytes(s.get(48..52)?.try_into().ok()?).into(),
            fru_text: if validation_bits & validation::FRU_STRING != 0 {
                Some(s.get(52..72)?.try_into().ok()?)
            } else {
                None
            },
        })
    }

    /// Updates the fields of the structure to reflect the actual binary layout.
    pub fn normalize(&mut self) {
        self.validation_bits = 0;
        if self.fru_id.is_some() {
            self.validation_bits |= validation::FRU_ID;
        }
        if self.fru_text.is_some() {
            self.validation_bits |= validation::FRU_STRING;
        }
    }

    /// Serializes the CPER Section Descriptor.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.section_offset.to_le_bytes());
        bytes.extend_from_slice(&self.section_length.to_le_bytes());
        bytes.append(&mut self.revision.to_bytes());

        bytes.push(self.validation_bits);
        bytes.push(0);
        bytes.extend_from_slice(&self.flags.to_le_bytes());
        bytes.extend_from_slice(&self.section_type.to_bytes());
        bytes.extend_from_slice(&self.fru_id.unwrap_or_default().to_bytes());
        bytes.extend_from_slice(&(self.section_severity as u32).to_le_bytes());
        bytes.extend_from_slice(&self.fru_text.unwrap_or_default());

        debug_assert_eq!(bytes.len(), SECTION_DESCRIPTOR_SIZE);
        bytes
    }
}
