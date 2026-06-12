// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Analyzer;
use super::tag::Tag;
use crate::node::Node;
#[cfg(not(feature = "std"))]
use alloc::{fmt, string::ToString};
#[cfg(feature = "std")]
use std::fmt;

pub enum ModelSpecificCode {
    Raw(u16),
}

impl fmt::Display for ModelSpecificCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw(mscod) => write!(f, "MSCOD_{mscod:04X}H"),
        }
    }
}

impl ModelSpecificCode {
    pub fn from_u16(code: u16) -> Self {
        Self::Raw(code)
    }
}

pub enum MachineCheckErrorCode {
    Raw(u16),
    InternalTimerError,
}

impl fmt::Display for MachineCheckErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw(mcacod) => write!(f, "MCACOD_{mcacod:04X}H"),
            Self::InternalTimerError => write!(f, "INTERNAL_TIMER_ERROR"),
        }
    }
}

impl MachineCheckErrorCode {
    pub fn from_u16(code: u16) -> Self {
        match code {
            0x0400 => Self::InternalTimerError,
            _ => Self::Raw(code),
        }
    }
}

pub struct Status {
    pub valid: bool,
    pub mscod: ModelSpecificCode,
    pub mcacod: MachineCheckErrorCode,
}

impl Status {
    pub fn from_u64(status: u64) -> Option<Self> {
        if status & 0xFFFFFFFF == 0xDEADBEEF {
            return None;
        }

        Some(Self {
            valid: (status >> 63) != 0,
            mscod: ModelSpecificCode::from_u16(((status >> 16) & 0xFFFF) as u16),
            mcacod: MachineCheckErrorCode::from_u16((status & 0xFFFF) as u16),
        })
    }
}

pub struct Bank {
    pub id: usize,
    pub status: Status,
}

impl Bank {
    pub fn from_node(bank: &Node) -> Option<Self> {
        let status = bank.get("status")?.value()?;

        Some(Self {
            id: bank
                .name
                .strip_prefix("bank")
                .and_then(|id| id.parse().ok())?,
            status: Status::from_u64(status)?,
        })
    }
}

impl Analyzer<'_> {
    pub(super) fn triage_mca_bank(&mut self, bank: &Bank) {
        if !bank.status.valid {
            return;
        }

        self.tag_with_priority(
            5,
            Tag::MachineCheck {
                bank: bank.id,
                mscod: bank.status.mscod.to_string(),
                mcacod: bank.status.mcacod.to_string(),
            },
        )
    }

    pub(super) fn analyze_mca(&mut self, mca: &Node) {
        for child in mca.children() {
            if let Some(bank) = Bank::from_node(child) {
                self.triage_mca_bank(&bank);
            } else {
                self.analyze_mca(child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mca_record() {
        let mut root = Node::root();
        root.add(Node::record("mca"));
        let bank = root.create_hierarchy("mca.foo.bar.bank42");
        bank.add(Node::field("status", 0x8000000012345678));

        let tags = Analyzer::default().with_input(&root).analyze().tags;

        assert!(
            tags.iter()
                .any(|t| t.to_string() == "MCA.BANK42.MCACOD_5678H.MSCOD_1234H")
        );
    }
}
