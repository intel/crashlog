// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

//! Provides access to the content of a Crash Log record.

mod core;
mod decode;

use crate::header::Header;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// A single Crash Log record
#[derive(Default)]
pub struct Record {
    /// Header of the record
    pub header: Header,
    /// Raw content of the record
    pub data: Vec<u8>,
    /// Additional information provided to the record
    pub context: Context,
}

/// Additional data provided to a Crash Log record
#[derive(Clone, Default)]
pub struct Context {
    /// Header of the parent record
    pub parent_header: Option<Header>,
}

impl Record {
    pub fn payload(&self) -> &[u8] {
        let begin = self.header.header_size();

        // The last DWORD of the record is reserved for the checksum when the CLDIC bit is set
        let end = self
            .data
            .len()
            .saturating_sub(if self.header.version.cldic { 4 } else { 0 });

        self.data.get(begin..end).unwrap_or_default()
    }

    pub fn checksum(&self) -> Option<bool> {
        if !self.header.version.cldic {
            return None;
        }

        let checksum = self
            .data
            .chunks(4)
            .map(|dword_slice| u32::from_le_bytes(dword_slice.try_into().unwrap_or([0; 4])))
            .fold(0, |acc: u32, dword| acc.wrapping_add(dword));

        Some(checksum == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::Version;

    #[test]
    fn payload() {
        let record = Record {
            data: (0..16).collect(),
            ..Default::default()
        };
        assert_eq!(record.payload(), &[8, 9, 10, 11, 12, 13, 14, 15]);

        let record_with_cldic = Record {
            header: Header {
                version: Version {
                    cldic: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            data: (0..16).collect(),
            ..Default::default()
        };
        assert_eq!(record_with_cldic.payload(), &[8, 9, 10, 11]);
    }

    #[test]
    fn payload_with_invalid_header() {
        let record = Record::default();
        assert!(record.payload().is_empty());

        let record_with_cldic = Record {
            header: Header {
                version: Version {
                    cldic: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(record_with_cldic.payload().is_empty());
    }
}
