// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(not(feature = "std"))]
use alloc::{fmt, vec, vec::Vec};
#[cfg(feature = "std")]
use std::fmt;

/// Revision field is used in several CPER structures
#[derive(Clone, Default)]
pub struct Revision {
    pub major: u8,
    pub minor: u8,
}

impl fmt::Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl Revision {
    pub fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    pub fn from_slice(s: &[u8]) -> Option<Self> {
        Some(Self {
            minor: *s.first()?,
            major: *s.get(1)?,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        vec![self.minor, self.major]
    }
}
