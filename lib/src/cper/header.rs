#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use super::revision::Revision;
use super::utils;
use crate::metadata;
use uguid::Guid;

pub const RECORD_HEADER_SIZE: usize = 128;
const LCF_GUID: Guid = uguid::guid!("eba67344-b876-4237-b80d-27e1297fa2ff");

pub mod validation {
    pub const PLATFORM_ID: u32 = 1;
    pub const TIMESTAMP: u32 = 2;
    pub const PARTITION_ID: u32 = 4;
}

pub mod flags {
    pub const RECOVERED: u32 = 1 << 0;
    pub const PREVERR: u32 = 1 << 1;
    pub const SIMULATED: u32 = 1 << 2;
}

pub mod notification_types {
    use uguid::Guid;
    pub const BOOT: Guid = uguid::guid!("3d61a466-ab40-409a-a698-f362d464b38f");
}

#[derive(Clone, Copy, Default)]
#[repr(u32)]
pub enum ErrorSeverity {
    Recoverable = 0,
    Fatal = 1,
    Corrected = 2,
    #[default]
    Informational = 3,
}

impl From<u32> for ErrorSeverity {
    fn from(value: u32) -> Self {
        match value {
            0 => ErrorSeverity::Recoverable,
            1 => ErrorSeverity::Fatal,
            2 => ErrorSeverity::Corrected,
            _ => ErrorSeverity::Informational,
        }
    }
}

/// Timestamp field used in the CPER Header
#[derive(Clone, Default)]
pub struct Timestamp {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub precise: bool,
    pub day: u8,
    pub month: u8,
    pub year: u8,
    pub century: u8,
}

impl Timestamp {
    pub fn from_crashlog_metadata(time: &metadata::Time) -> Self {
        Self {
            century: utils::bin_to_bcd((time.year / 100) as u8),
            year: utils::bin_to_bcd((time.year % 100) as u8),
            month: utils::bin_to_bcd(time.month),
            day: utils::bin_to_bcd(time.day),
            hours: utils::bin_to_bcd(time.hour),
            minutes: utils::bin_to_bcd(time.minute),
            ..Self::default()
        }
    }

    pub fn from_slice(s: &[u8]) -> Option<Self> {
        Some(Self {
            seconds: *s.first()?,
            minutes: *s.get(1)?,
            hours: *s.get(2)?,
            precise: *s.get(3)? & 1 != 0,
            day: *s.get(4)?,
            month: *s.get(5)?,
            year: *s.get(6)?,
            century: *s.get(7)?,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.seconds,
            self.minutes,
            self.hours,
            self.precise as u8,
            self.day,
            self.month,
            self.year,
            self.century,
        ]
    }
}

/// UEFI 2.10 N.2.1 Record Header.
#[derive(Clone)]
pub struct CperHeader {
    pub revision: Revision,
    pub section_count: u16,
    pub error_severity: ErrorSeverity,
    pub validation_bits: u32,
    pub record_length: u32,
    pub timestamp: Option<Timestamp>,
    pub platform_id: Option<Guid>,
    pub partition_id: Option<Guid>,
    pub creator_id: Guid,
    pub notification_type: Guid,
    pub record_id: u64,
    pub flags: u32,
    pub persistence_information: u64,
}

impl Default for CperHeader {
    fn default() -> Self {
        Self {
            revision: Revision::new(1, 1),
            section_count: 0,
            error_severity: ErrorSeverity::default(),
            validation_bits: 0,
            record_length: RECORD_HEADER_SIZE as u32,
            timestamp: None,
            platform_id: None,
            partition_id: None,
            creator_id: LCF_GUID,
            notification_type: Guid::default(),
            record_id: 0,
            flags: 0,
            persistence_information: 0,
        }
    }
}

impl CperHeader {
    /// Parses the CPER header stored in a byte slice.
    pub fn from_slice(s: &[u8]) -> Option<Self> {
        let signature_end = u32::from_le_bytes(s.get(6..10)?.try_into().ok()?);
        if !s.starts_with(b"CPER") || signature_end != 0xFFFFFFFF {
            return None;
        }
        let revision = Revision::from_slice(s.get(4..6)?)?;
        if revision.major != 1 {
            log::warn!("Unsupported CPER Record Header revision: {revision}.");
        }

        let validation_bits = u32::from_le_bytes(s.get(16..20)?.try_into().ok()?);

        Some(Self {
            section_count: u16::from_le_bytes(s.get(10..12)?.try_into().ok()?),
            error_severity: u32::from_le_bytes(s.get(12..16)?.try_into().ok()?).into(),
            revision,
            validation_bits,
            record_length: u32::from_le_bytes(s.get(20..24)?.try_into().ok()?),
            timestamp: if validation_bits & validation::TIMESTAMP != 0 {
                Some(Timestamp::from_slice(s.get(24..32)?)?)
            } else {
                None
            },
            platform_id: if validation_bits & validation::PLATFORM_ID != 0 {
                Some(Guid::from_bytes(s.get(32..48)?.try_into().ok()?))
            } else {
                None
            },
            partition_id: if validation_bits & validation::PARTITION_ID != 0 {
                Some(Guid::from_bytes(s.get(48..64)?.try_into().ok()?))
            } else {
                None
            },
            creator_id: Guid::from_bytes(s.get(64..80)?.try_into().ok()?),
            notification_type: Guid::from_bytes(s.get(80..96)?.try_into().ok()?),
            record_id: u64::from_le_bytes(s.get(96..104)?.try_into().ok()?),
            flags: u32::from_le_bytes(s.get(104..108)?.try_into().ok()?),
            persistence_information: u64::from_le_bytes(s.get(108..116)?.try_into().ok()?),
        })
    }

    /// Updates the fields of the structures to reflect the actual binary layout of the CPER.
    pub fn normalize(&mut self) {
        self.validation_bits = 0;
        if self.timestamp.is_some() {
            self.validation_bits |= validation::TIMESTAMP;
        }
        if self.platform_id.is_some() {
            self.validation_bits |= validation::PLATFORM_ID;
        }
        if self.partition_id.is_some() {
            self.validation_bits |= validation::PARTITION_ID;
        }
    }

    /// Serializes the CPER Header
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"CPER");
        bytes.append(&mut self.revision.to_bytes());
        bytes.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        bytes.extend_from_slice(&self.section_count.to_le_bytes());
        bytes.extend_from_slice(&(self.error_severity as u32).to_le_bytes());

        bytes.extend_from_slice(&self.validation_bits.to_le_bytes());
        bytes.extend_from_slice(&self.record_length.to_le_bytes());
        bytes.append(&mut self.timestamp.clone().unwrap_or_default().to_bytes());
        bytes.extend_from_slice(&self.platform_id.unwrap_or_default().to_bytes());
        bytes.extend_from_slice(&self.partition_id.unwrap_or_default().to_bytes());
        bytes.extend_from_slice(&self.creator_id.to_bytes());
        bytes.extend_from_slice(&self.notification_type.to_bytes());
        bytes.extend_from_slice(&self.record_id.to_le_bytes());
        bytes.extend_from_slice(&self.flags.to_le_bytes());
        bytes.extend_from_slice(&self.persistence_information.to_le_bytes());
        bytes.extend_from_slice(&[0; 12]);

        debug_assert_eq!(bytes.len(), RECORD_HEADER_SIZE);
        bytes
    }
}
