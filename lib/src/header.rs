// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

//! Data structures used in the Crash Log record headers.

#[cfg(feature = "collateral_manager")]
use crate::collateral::{CollateralManager, CollateralTree, ItemPath, PVSS};
use crate::error::Error;
use crate::node::Node;
#[cfg(not(feature = "std"))]
use alloc::{
    fmt, format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
#[cfg(feature = "std")]
use std::fmt;

/// Lists all the Crash Log record types.
///
/// cbindgen:ignore
pub mod record_types {
    pub const PMC: u8 = 0x1;
    pub const PMC_FW_TRACE: u8 = 0x2;
    pub const PUNIT: u8 = 0x3;
    pub const PCORE: u8 = 0x4;
    pub const ECORE: u8 = 0x6;
    pub const UNCORE: u8 = 0x8;
    pub const PMC_TRACE: u8 = 0x11;
    pub const TCSS: u8 = 0x16;
    pub const PMC_RST: u8 = 0x17;
    pub const PCODE: u8 = 0x19;
    pub const CRASHLOG_AGENT: u8 = 0x1C;
    pub const BOX: u8 = 0x3D;
    pub const MCA: u8 = 0x3E;
}

#[derive(Debug, Default)]
pub enum HeaderType {
    Type0,
    #[default]
    Type1,
    Type2 {
        timestamp: u64,
        agent_version: u32,
        reason: u32,
    },
    Type3 {
        timestamp: u64,
        agent_version: u32,
        reason: u32,
        completion_status: u32,
        collection_complete: bool,
    },
    Type4 {
        timestamp: u64,
        agent_version: u32,
        reason: u32,
        whoami: u32,
        misc: u32,
    },
    Type5 {
        timestamp: u64,
        agent_version: u32,
        reason: u32,
        completion_status: u32,
        collection_complete: bool,
        error_status: u32,
    },
    Type6 {
        timestamp: u64,
        agent_version: u32,
        reason: u32,
        die_id: u8,
        socket_id: u8,
        completion_status_size: u16,
        completion_status: Vec<u32>,
        collection_complete: bool,
    },
}

impl HeaderType {
    fn type2_from_slice(slice: &[u8]) -> Option<Self> {
        let timestamp = u64::from_le_bytes(slice.get(8..16)?.try_into().ok()?);
        let agent_version = u32::from_le_bytes(slice.get(16..20)?.try_into().ok()?);
        let reason = u32::from_le_bytes(slice.get(20..24)?.try_into().ok()?);

        Some(HeaderType::Type2 {
            timestamp,
            agent_version,
            reason,
        })
    }

    fn type3_from_slice(slice: &[u8]) -> Option<Self> {
        let timestamp = u64::from_le_bytes(slice.get(8..16)?.try_into().ok()?);
        let agent_version = u32::from_le_bytes(slice.get(16..20)?.try_into().ok()?);
        let reason = u32::from_le_bytes(slice.get(20..24)?.try_into().ok()?);
        let cs_data = u32::from_le_bytes(slice.get(24..28)?.try_into().ok()?);
        let completion_status = cs_data & 0x7FFFFFFF;
        let collection_complete = (cs_data >> 31) != 0;

        Some(HeaderType::Type3 {
            timestamp,
            agent_version,
            reason,
            completion_status,
            collection_complete,
        })
    }

    fn type4_from_slice(slice: &[u8]) -> Option<Self> {
        let timestamp = u64::from_le_bytes(slice.get(8..16)?.try_into().ok()?);
        let agent_version = u32::from_le_bytes(slice.get(16..20)?.try_into().ok()?);
        let reason = u32::from_le_bytes(slice.get(20..24)?.try_into().ok()?);
        let whoami = u32::from_le_bytes(slice.get(24..28)?.try_into().ok()?);
        let misc = u32::from_le_bytes(slice.get(28..32)?.try_into().ok()?);

        Some(HeaderType::Type4 {
            timestamp,
            agent_version,
            reason,
            whoami,
            misc,
        })
    }

    fn type5_from_slice(slice: &[u8]) -> Option<Self> {
        let timestamp = u64::from_le_bytes(slice.get(8..16)?.try_into().ok()?);
        let agent_version = u32::from_le_bytes(slice.get(16..20)?.try_into().ok()?);
        let reason = u32::from_le_bytes(slice.get(20..24)?.try_into().ok()?);
        let cs_data = u32::from_le_bytes(slice.get(24..28)?.try_into().ok()?);
        let completion_status = cs_data & 0x7FFFFFFF;
        let collection_complete = (cs_data >> 31) != 0;
        let error_status = u32::from_le_bytes(slice.get(28..32)?.try_into().ok()?);

        Some(HeaderType::Type5 {
            timestamp,
            agent_version,
            reason,
            completion_status,
            collection_complete,
            error_status,
        })
    }

    fn type6_from_slice(slice: &[u8]) -> Option<Self> {
        let timestamp = u64::from_le_bytes(slice.get(8..16)?.try_into().ok()?);
        let agent_version = u32::from_le_bytes(slice.get(16..20)?.try_into().ok()?);
        let reason = u32::from_le_bytes(slice.get(20..24)?.try_into().ok()?);
        let die_skt_info = slice.get(24..28)?;

        let die_id = die_skt_info[0];
        let socket_id = die_skt_info[1];
        let completion_status_size = u16::from_le_bytes(die_skt_info[2..4].try_into().ok()?) & 0x7F;
        let collection_complete = (die_skt_info[3] & 0x80) != 0;

        let completion_status = (0..completion_status_size)
            .map(|dword| {
                let index = (28 + dword * 4) as usize;
                Some(u32::from_le_bytes(
                    slice.get(index..index + 4)?.try_into().ok()?,
                ))
            })
            .collect::<Option<Vec<u32>>>()?;

        Some(HeaderType::Type6 {
            timestamp,
            agent_version,
            reason,
            die_id,
            socket_id,
            completion_status_size,
            completion_status,
            collection_complete,
        })
    }

    pub fn from_slice(header_type_value: u16, slice: &[u8]) -> Result<Self, Error> {
        match header_type_value {
            0 => Ok(HeaderType::Type0),
            1 => Ok(HeaderType::Type1),
            2 => Self::type2_from_slice(slice).ok_or(Error::InvalidHeader),
            3 => Self::type3_from_slice(slice).ok_or(Error::InvalidHeader),
            4 => Self::type4_from_slice(slice).ok_or(Error::InvalidHeader),
            5 => Self::type5_from_slice(slice).ok_or(Error::InvalidHeader),
            6 => Self::type6_from_slice(slice).ok_or(Error::InvalidHeader),
            type_value => Err(Error::InvalidHeaderType(type_value)),
        }
    }
}

/// Header of a Crash Log record
#[derive(Debug, Default)]
pub struct Header {
    /// Version ID
    pub version: Version,
    /// Size of the record
    pub size: RecordSize,
    /// Optional fields
    pub header_type: HeaderType,
}

impl Header {
    /// Decodes a header of a raw Crash Log record.
    pub fn from_slice(slice: &[u8]) -> Result<Option<Self>, Error> {
        let Some(version) = Version::from_slice(slice) else {
            // Termination marker
            return Ok(None);
        };
        let size = RecordSize::from_slice(slice).ok_or(Error::InvalidHeader)?;
        let header_type = HeaderType::from_slice(version.header_type, slice)?;

        Ok(Some(Header {
            version,
            size,
            header_type,
        }))
    }

    /// Returns the granularity of the record size fields in bytes
    #[inline]
    fn record_size_granularity(&self) -> usize {
        match (self.version.record_type, self.version.product_id) {
            (record_types::ECORE, _) => 1,
            (record_types::PCORE, product_id) if product_id < 0x71 => 1,
            _ => 4,
        }
    }

    /// Returns the size of the record in bytes.
    #[inline]
    pub fn record_size(&self) -> usize {
        (self.size.record_size as usize + self.size.extended_record_size as usize)
            * self.record_size_granularity()
    }

    /// Returns the offset of the extended record in bytes if present.
    #[inline]
    pub fn extended_record_offset(&self) -> Option<usize> {
        if self.size.extended_record_size > 0 {
            Some(self.size.record_size as usize * self.record_size_granularity())
        } else {
            None
        }
    }

    /// Returns the size of the record in bytes.
    #[inline]
    pub fn revision(&self) -> u32 {
        self.version.revision
    }

    /// Returns the product ID of the record.
    #[inline]
    pub fn product_id(&self) -> u32 {
        self.version.product_id
    }

    /// Returns the Three-Letter Acronym associated to the product ID specified in the header.
    #[cfg(feature = "collateral_manager")]
    pub fn product<'a, T: CollateralTree>(
        &self,
        cm: &'a CollateralManager<T>,
    ) -> Result<&'a str, Error> {
        cm.target_info
            .get(&self.product_id())
            .map(|target_info| target_info.product.as_str())
            .ok_or_else(|| Error::InvalidProductID(self.product_id()))
    }

    /// Returns the product variant associated to the product ID specified in the header.
    #[cfg(feature = "collateral_manager")]
    pub fn variant<'a, T: CollateralTree>(&self, cm: &'a CollateralManager<T>) -> Option<&'a str> {
        cm.target_info
            .get(&self.product_id())
            .map(|target_info| target_info.variant.as_str())
    }

    /// Returns the ID of the socket that generated the record.
    pub fn socket_id(&self) -> u8 {
        if let HeaderType::Type6 { socket_id, .. } = self.header_type {
            socket_id
        } else {
            0
        }
    }

    /// Returns the ID of the die that generated the record.
    pub fn die_id(&self) -> Option<u8> {
        if let HeaderType::Type6 { die_id, .. } = self.header_type {
            Some(die_id)
        } else {
            None
        }
    }

    /// Returns the name of the die that generated the record.
    ///
    /// This requires a [CollateralManager] as the die names are product-specific.
    #[cfg(feature = "collateral_manager")]
    pub fn die<'a, T: CollateralTree>(&self, cm: &'a CollateralManager<T>) -> Option<&'a str> {
        let target_info = cm.target_info.get(&self.product_id())?;
        let die_id = target_info.die_id.get(&self.die_id()?)?;
        Some(die_id)
    }

    /// Returns the type of the record.
    pub fn record_type(&self) -> Result<&'static str, Error> {
        self.version.record_type_as_str()
    }

    #[cfg(feature = "collateral_manager")]
    pub(super) fn decode_definitions_paths<T: CollateralTree>(
        &self,
        cm: &CollateralManager<T>,
    ) -> Result<Vec<ItemPath>, Error> {
        let record_type = self.record_type()?;
        let revision = self.revision().to_string();

        Ok(if let Some(die) = self.die(cm) {
            let die_id = die.trim_end_matches(char::is_numeric);
            vec![ItemPath::new([
                "decode-defs",
                record_type,
                die_id,
                &revision,
            ])]
        } else {
            vec![
                ItemPath::new(["decode-defs", record_type, &revision]),
                ItemPath::new(["decode-defs", record_type, "all"]),
            ]
        })
    }

    #[cfg(feature = "collateral_manager")]
    /// Returns the [PVSS] associated to this header.
    pub fn pvss<T: CollateralTree>(&self, cm: &CollateralManager<T>) -> Result<PVSS, Error> {
        let product = match self.product(cm) {
            Err(Error::InvalidProductID(0)) => "all",
            res => res?,
        };
        let variant = self.variant(cm).unwrap_or("all");

        Ok(PVSS {
            product: product.into(),
            variant: variant.into(),
            ..PVSS::default()
        })
    }

    /// Returns the size of the header in bytes.
    pub fn header_size(&self) -> usize {
        match self.header_type {
            HeaderType::Type0 | HeaderType::Type1 => 8,
            HeaderType::Type2 { .. } => 24,
            HeaderType::Type3 { .. } => 28,
            HeaderType::Type4 { .. } => 32,
            HeaderType::Type5 { .. } => 32,
            HeaderType::Type6 {
                completion_status_size,
                ..
            } => 28 + completion_status_size as usize * 4,
        }
    }

    pub(super) fn get_root_path(&self) -> Option<String> {
        if let HeaderType::Type6 {
            socket_id, die_id, ..
        } = self.header_type
        {
            Some(format!("processors.cpu{}.die{}", socket_id, die_id))
        } else {
            None
        }
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let record_type = self.record_type().unwrap_or("RECORD");
        let version = format!(
            "product_id=0x{0:x}, record_type=0x{1:x}, revision=0x{2:x}",
            self.version.product_id, self.version.record_type, self.version.revision
        );
        let header_type = match self.header_type {
            HeaderType::Type6 {
                die_id, socket_id, ..
            } => format!("die_id={0}, socket_id={1}", die_id, socket_id),
            _ => "..".to_string(),
        };

        write!(f, "{} - ({}, {})", record_type, version, header_type)
    }
}

/// Version of the Crash Log record
#[derive(Clone, Debug, Default)]
pub struct Version {
    /// Revision of the record
    pub revision: u32,
    /// Type of the header
    pub header_type: u16,
    /// Product that generated the record
    pub product_id: u32,
    /// Type of the record
    pub record_type: u8,
    /// Indicates if the record has been consumed by IAFW
    pub consumed: bool,
    /// Integrity checker present
    pub cldic: bool,
}

impl Version {
    /// Creates a [Version] from the raw record
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        let version = u32::from_le_bytes(slice.get(0..4)?.try_into().ok()?);
        log::trace!("Decoding record version: {:#x}", version);

        if version == 0 || version == 0xdeadbeef {
            // Termination marker
            return None;
        }

        Some(Version {
            cldic: (version >> 30) & 1 == 1,
            consumed: (version >> 31) & 1 == 1,
            revision: version & 0xFF,
            header_type: ((version >> 8) & 0xF) as u16,
            product_id: (version >> 12) & 0xFFF,
            record_type: ((version >> 24) & 0x3F) as u8,
        })
    }

    pub fn as_u32(&self) -> u32 {
        ((self.consumed as u32) << 31)
            | ((self.cldic as u32) << 30)
            | ((self.record_type as u32) << 24)
            | (self.product_id << 12)
            | ((self.header_type as u32) << 8)
            | self.revision
    }

    fn record_type_as_str(&self) -> Result<&'static str, Error> {
        Ok(match self.record_type {
            record_types::PMC => "PMC",
            record_types::PMC_FW_TRACE => "PMC_FW_Trace",
            record_types::PUNIT => "Punit",
            record_types::PCORE => "PCORE",
            record_types::ECORE => "ECORE",
            record_types::UNCORE => "UNCORE",
            record_types::PMC_TRACE => "PMC_TRACE",
            record_types::TCSS => "TCSS",
            record_types::PMC_RST => "PMC_RST",
            record_types::PCODE => "PCODE",
            record_types::CRASHLOG_AGENT => "CRASHLOG_AGENT",
            record_types::BOX => "BOX",
            record_types::MCA => "MCA",
            rt => return Err(Error::InvalidRecordType(rt)),
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let record_type = self.record_type_as_str().unwrap_or("UNKNOWN");
        write!(f, "{} revision {}", record_type, self.revision)
    }
}

/// Size of the Crash Log record
#[derive(Debug, Default)]
pub struct RecordSize {
    /// Size of the main section of the record in dwords
    pub record_size: u16,
    /// Size of the extended section of the record in dwords
    pub extended_record_size: u16,
}

impl RecordSize {
    /// Creates a [RecordSize] from the raw record
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        Some(RecordSize {
            record_size: u16::from_le_bytes(slice.get(4..6)?.try_into().ok()?),
            extended_record_size: u16::from_le_bytes(slice.get(6..8)?.try_into().ok()?),
        })
    }
}

impl From<&RecordSize> for Node {
    fn from(size: &RecordSize) -> Self {
        let mut node = Node::section("record_size");
        node.add(Node::field("record_size", size.record_size as u64));
        node.add(Node::field(
            "extended_record_size",
            size.extended_record_size as u64,
        ));
        node
    }
}

impl From<&Version> for Node {
    fn from(version: &Version) -> Self {
        let mut node = Node::field("version", version.as_u32() as u64);
        node.add(Node::field("revision", version.revision as u64));
        node.add(Node::field("header_type", version.header_type as u64));
        node.add(Node::field("product_id", version.product_id as u64));
        node.add(Node::field("record_type", version.record_type as u64));
        node
    }
}

impl From<&Header> for Node {
    fn from(header: &Header) -> Self {
        let mut node = Node::section("hdr");
        node.add(Node::from(&header.version));
        node.add(Node::from(&header.size));

        match header.header_type {
            HeaderType::Type2 {
                timestamp,
                agent_version,
                reason,
            } => {
                node.add(Node::field("timestamp", timestamp));
                node.add(Node::field("agent_version", agent_version as u64));
                node.add(Node::field("reason", reason as u64));
            }
            HeaderType::Type3 {
                timestamp,
                agent_version,
                reason,
                completion_status,
                collection_complete,
            } => {
                node.add(Node::field("timestamp", timestamp));
                node.add(Node::field("agent_version", agent_version as u64));
                node.add(Node::field("reason", reason as u64));

                let mut completion_status_node = Node::section("completion_status");
                completion_status_node
                    .add(Node::field("completion_status", completion_status as u64));
                completion_status_node.add(Node::field(
                    "record_collection_completed",
                    collection_complete as u64,
                ));
                node.add(completion_status_node);
            }
            HeaderType::Type4 {
                timestamp,
                agent_version,
                reason,
                whoami,
                misc,
            } => {
                node.add(Node::field("timestamp", timestamp));
                node.add(Node::field("agent_version", agent_version as u64));
                node.add(Node::field("reason", reason as u64));
                node.add(Node::field("whoami", whoami as u64));
                node.add(Node::field("misc", misc as u64));
            }
            HeaderType::Type5 {
                timestamp,
                agent_version,
                reason,
                completion_status,
                collection_complete,
                error_status,
            } => {
                node.add(Node::field("timestamp", timestamp));
                node.add(Node::field("agent_version", agent_version as u64));
                node.add(Node::field("reason", reason as u64));
                node.add(Node::field("error_status", error_status as u64));

                let mut completion_status_node = Node::section("completion_status");
                completion_status_node
                    .add(Node::field("completion_status", completion_status as u64));
                completion_status_node.add(Node::field(
                    "record_collection_completed",
                    collection_complete as u64,
                ));
                node.add(completion_status_node);
            }
            HeaderType::Type6 {
                timestamp,
                agent_version,
                reason,
                die_id,
                socket_id,
                ref completion_status,
                completion_status_size,
                collection_complete,
            } => {
                node.add(Node::field("timestamp", timestamp));
                node.add(Node::field("agent_version", agent_version as u64));
                node.add(Node::field("reason", reason as u64));
                let mut die_skt_info = Node::section("die_skt_info");
                die_skt_info.add(Node::field("die_id", die_id as u64));
                die_skt_info.add(Node::field("socket_id", socket_id as u64));
                die_skt_info.add(Node::field(
                    "completion_status_size",
                    completion_status_size as u64,
                ));
                die_skt_info.add(Node::field(
                    "record_collection_completed",
                    collection_complete as u64,
                ));
                node.add(die_skt_info);

                for (i, completion_status) in completion_status.iter().enumerate() {
                    node.add(Node::field(
                        &format!("completion_status{}", i),
                        *completion_status as u64,
                    ));
                }
            }
            _ => (),
        }

        node
    }
}
