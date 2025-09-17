// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

#![allow(dead_code)]

pub mod descr;
pub mod header;
pub mod revision;
pub mod section;
#[cfg(test)]
mod tests;
mod utils;

use crate::CrashLog;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use descr::{CperSectionDescriptor, SECTION_DESCRIPTOR_SIZE};
use header::{CperHeader, RECORD_HEADER_SIZE};
pub use section::{CperSection, CperSectionBody};

/// UEFI Common Platform Error Record (N)
#[derive(Default)]
pub struct Cper {
    /// CPER Record Header
    record_header: CperHeader,
    /// CPER Sections
    pub sections: Vec<CperSection>,
}

impl Cper {
    /// Parses the CPER stored in a byte slice.
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        let record_header = CperHeader::from_slice(slice.get(0..RECORD_HEADER_SIZE)?)?;

        let sections = (0..record_header.section_count)
            .filter_map(|i| {
                let index = RECORD_HEADER_SIZE + (i as usize * SECTION_DESCRIPTOR_SIZE);
                let descriptor = CperSectionDescriptor::from_slice(slice.get(index..)?)?;
                let offset = descriptor.section_offset as usize;
                let end_offset = offset + descriptor.section_length as usize;
                let body = CperSectionBody::from_slice(
                    descriptor.section_type,
                    slice.get(offset..end_offset)?,
                )?;
                Some(CperSection { descriptor, body })
            })
            .collect::<Vec<CperSection>>();

        let mut cper = Cper {
            record_header,
            sections,
        };
        cper.normalize();
        Some(cper)
    }

    /// Create a CPER Section from a Crash Log.
    pub fn from_raw_crashlog(crashlog: &CrashLog) -> Self {
        let mut cper = Cper::default();

        cper.record_header.notification_type = header::notification_types::BOOT;
        cper.record_header.error_severity = header::ErrorSeverity::Fatal;

        cper.record_header.timestamp = crashlog
            .metadata
            .time
            .as_ref()
            .map(header::Timestamp::from_crashlog_metadata);

        for region in crashlog.regions.iter() {
            let mut section = CperSection::from_crashlog_region(region);
            section.descriptor.section_severity = descr::SectionSeverity::Fatal;
            cper.append_section(section);
        }

        for extra_cper_section in crashlog.metadata.extra_cper_sections.iter() {
            cper.append_section(CperSection::from_body(extra_cper_section.clone()));
        }

        cper
    }

    /// Appends a section to the CPER record and updates the header fields to reflect the actual
    /// binary layout of the CPER.
    pub fn append_section(&mut self, section: CperSection) {
        self.sections.push(section);
        self.normalize();
    }

    /// Updates the fields of the structures to reflect the actual binary layout of the CPER.
    fn normalize(&mut self) {
        self.record_header.section_count = self.sections.len() as u16;

        let mut cursor = RECORD_HEADER_SIZE + SECTION_DESCRIPTOR_SIZE * self.sections.len();

        for section in self.sections.iter_mut() {
            section.descriptor.section_offset = cursor as u32;
            section.descriptor.normalize();
            cursor += section.descriptor.section_length as usize;
        }

        self.record_header.record_length = cursor as u32;
        self.record_header.normalize();
    }

    /// Serializes the CPER
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.append(&mut self.record_header.to_bytes());

        for section in self.sections.iter() {
            bytes.append(&mut section.descriptor.to_bytes())
        }

        for section in self.sections.iter() {
            bytes.append(&mut section.body_bytes())
        }

        bytes
    }
}
