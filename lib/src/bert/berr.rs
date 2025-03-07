// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Bert;
use crate::CrashLog;
use crate::cper::fer::{FirmwareErrorRecord, FirmwareErrorRecordHeader, RECORD_ID_CRASHLOG};
use crate::cper::{CperSection, FW_ERROR_RECORD_GUID};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use core::mem;
#[cfg(feature = "std")]
use std::mem;

/// Generic Error Status Block (ACPI 6.3 - Table 18-391)
#[repr(C, packed)]
#[derive(Debug, Clone, Default)]
pub struct GenericErrorStatusBlock {
    pub block_status: u32,
    pub raw_data_offset: u32,
    pub raw_data_length: u32,
    pub data_length: u32,
    pub error_severity: u32,
}

impl GenericErrorStatusBlock {
    fn from_slice(s: &[u8]) -> Option<Self> {
        Some(Self {
            block_status: u32::from_le_bytes(s.get(0..4)?.try_into().ok()?),
            raw_data_offset: u32::from_le_bytes(s.get(4..8)?.try_into().ok()?),
            raw_data_length: u32::from_le_bytes(s.get(8..12)?.try_into().ok()?),
            data_length: u32::from_le_bytes(s.get(12..16)?.try_into().ok()?),
            error_severity: u32::from_le_bytes(s.get(16..20)?.try_into().ok()?),
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.block_status.to_le_bytes());
        bytes.extend_from_slice(&self.raw_data_offset.to_le_bytes());
        bytes.extend_from_slice(&self.raw_data_length.to_le_bytes());
        bytes.extend_from_slice(&self.data_length.to_le_bytes());
        bytes.extend_from_slice(&self.error_severity.to_le_bytes());
        bytes
    }

    fn size(&self) -> usize {
        mem::size_of::<Self>()
    }
}

/// Generic Error Data Entry (ACPI 6.3 - Table 18-392)
#[repr(C, packed)]
#[derive(Debug, Clone, Default)]
pub struct GenericErrorDataEntryHeader {
    pub section: uguid::Guid,
    pub error_severity: u32,
    pub revision: u16,
    pub validation_bits: u8,
    pub flags: u8,
    pub error_data_length: u32,
    pub fru_id: [u8; 16],
    pub fru_text: [u8; 20],
    pub timestamp: u64,
}

impl GenericErrorDataEntryHeader {
    fn from_slice(s: &[u8]) -> Option<Self> {
        let revision = u16::from_le_bytes(s.get(20..22)?.try_into().ok()?);

        Some(Self {
            section: uguid::Guid::from_bytes(s.get(0..16)?.try_into().ok()?),
            error_severity: u32::from_le_bytes(s.get(16..20)?.try_into().ok()?),
            revision,
            validation_bits: *s.get(22)?,
            flags: *s.get(23)?,
            error_data_length: u32::from_le_bytes(s.get(24..28)?.try_into().ok()?),
            fru_id: s.get(28..44)?.try_into().ok()?,
            fru_text: s.get(44..64)?.try_into().ok()?,
            timestamp: if revision >= 0x300 {
                u64::from_le_bytes(s.get(64..72)?.try_into().ok()?)
            } else {
                0
            },
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.section.to_bytes());
        bytes.extend_from_slice(&self.error_severity.to_le_bytes());
        bytes.extend_from_slice(&self.revision.to_le_bytes());
        bytes.push(self.validation_bits);
        bytes.push(self.flags);
        bytes.extend_from_slice(&self.error_data_length.to_le_bytes());
        bytes.extend_from_slice(&self.fru_id);
        bytes.extend_from_slice(&self.fru_text);

        if self.revision >= 0x300 {
            bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        }

        bytes
    }

    fn size(&self) -> usize {
        if self.revision >= 0x300 {
            mem::size_of::<Self>()
        } else {
            mem::size_of::<Self>() - mem::size_of::<u64>()
        }
    }
}

pub struct GenericErrorDataEntry {
    pub header: GenericErrorDataEntryHeader,
    pub cper_section: CperSection,
}

impl GenericErrorDataEntry {
    fn from_slice(s: &[u8]) -> Option<Self> {
        let header = GenericErrorDataEntryHeader::from_slice(s)?;
        let cper_section = CperSection::from_slice(
            header.section,
            s.get(header.size()..header.size() + header.error_data_length as usize)?,
        )?;
        Some(Self {
            header,
            cper_section,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut header = self.header.clone();
        let mut cper_section = self.cper_section.to_bytes();
        header.error_data_length = cper_section.len() as u32;

        let mut bytes = header.to_bytes();
        bytes.append(&mut cper_section);
        bytes
    }

    fn size(&self) -> usize {
        self.header.size() + self.header.error_data_length as usize
    }
}

#[derive(Default)]
pub struct Berr {
    pub header: GenericErrorStatusBlock,
    pub entries: Vec<GenericErrorDataEntry>,
}

impl Berr {
    pub fn from_crashlog(crashlog: &CrashLog) -> Berr {
        let header = GenericErrorStatusBlock::default();
        let entries = crashlog
            .regions
            .iter()
            .map(|region| {
                let header = GenericErrorDataEntryHeader {
                    section: FW_ERROR_RECORD_GUID,
                    revision: 0x300,
                    ..GenericErrorDataEntryHeader::default()
                };
                let cper_section = CperSection::FirmwareErrorRecord(FirmwareErrorRecord {
                    header: FirmwareErrorRecordHeader {
                        error_type: 2,
                        revision: 2,
                        guid: RECORD_ID_CRASHLOG,
                        ..FirmwareErrorRecordHeader::default()
                    },
                    payload: region.to_bytes(),
                });
                GenericErrorDataEntry {
                    header,
                    cper_section,
                }
            })
            .collect();
        Berr { header, entries }
    }

    pub fn from_bert_file(s: &[u8]) -> Option<Berr> {
        if s.starts_with(b"BERR") {
            Berr::from_slice(s.get(4..)?)
        } else if s.starts_with(b"BERT") {
            Berr::from_slice(s.get(mem::size_of::<Bert>()..)?)
        } else {
            None
        }
    }

    pub fn from_slice(s: &[u8]) -> Option<Berr> {
        let header = GenericErrorStatusBlock::from_slice(s)?;

        let mut ptr = header.size();
        let end = ptr + header.data_length as usize;
        let mut entries = Vec::new();

        while let Some(entry) = GenericErrorDataEntry::from_slice(s.get(ptr..end)?) {
            ptr += entry.size();
            entries.push(entry);
            if ptr >= end {
                break;
            }
        }

        Some(Berr { header, entries })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut header = self.header.clone();
        header.data_length = 0;

        let mut payload = Vec::new();
        for entry in self.entries.iter() {
            let mut entry = entry.to_bytes();
            header.data_length += entry.len() as u32;
            payload.append(&mut entry);
        }

        let mut bytes = header.to_bytes();
        bytes.append(&mut payload);
        bytes
    }
}
