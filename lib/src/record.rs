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
    /// Die ID of the record
    pub die_id: Option<u8>,
    /// Socket ID of the record
    pub socket_id: Option<u8>,
}

impl Record {
    pub fn payload(&self) -> &[u8] {
        let begin = self.header.header_size();
        let end = if self.header.version.cldic {
            // Checksum is present at the end of the record
            self.data.len() - 4
        } else {
            self.data.len()
        };
        &self.data[begin..end]
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
