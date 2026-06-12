// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Analyzer;
use super::reason::CrashLogReason;
use super::reset::ResetKind;
use super::xq::TransactionQueueState;
#[cfg(not(feature = "std"))]
use alloc::{fmt, string::String};
#[cfg(not(feature = "std"))]
use core::cmp::Ordering;
#[cfg(feature = "std")]
use std::cmp::Ordering;
#[cfg(feature = "std")]
use std::fmt;

/// Triage tag identifying a specific crash log issue or finding.
///
/// Tags represent decoded and classified information from crash log records.
/// Each variant corresponds to a different category of system failure or
/// diagnostic event.
///
/// # Display Format
///
/// Tags implement [`Display`](fmt::Display) with a hierarchical dot-notation format
/// for easy parsing and filtering:
///
/// - `RESET_CAUSE.{kind}.{cause}` - System reset events
/// - `CRASHLOG_REASON.{record}.{reason}` - Crash Log trigger reasons
/// - `CORE_TIMEOUT.{transaction_queue}` - Core timeout
/// - `MCA.BANK{n}.{mcacod}.{mscod}` - Machine Check Architecture events
#[derive(Clone, PartialEq, Eq)]
pub enum Tag {
    /// Cause of the latest reset.
    ResetCause { kind: ResetKind, cause: String },
    /// Defines why the Crash Log collection was triggered.
    CrashLogReason {
        record: String,
        reason: CrashLogReason,
    },
    /// Core timeout has occurred.
    CoreTimeout {
        transaction_queue: TransactionQueueState,
    },
    /// Machine Check Architecture (MCA) error has occurred.
    MachineCheck {
        bank: usize,
        mscod: String,
        mcacod: String,
    },
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct TriageTag {
    priority: i32,
    pub tag: Tag,
}

impl Ord for TriageTag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for TriageTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResetCause { kind, cause } => write!(f, "RESET_CAUSE.{kind}.{cause}"),
            Self::CrashLogReason { record, reason } => {
                write!(f, "CRASHLOG_REASON.{record}.{reason}")
            }
            Self::CoreTimeout { transaction_queue } => {
                write!(f, "CORE_TIMEOUT.{transaction_queue}")
            }
            Self::MachineCheck {
                bank,
                mscod,
                mcacod,
            } => {
                write!(f, "MCA.BANK{bank}.{mcacod}.{mscod}")
            }
        }
    }
}

impl Analyzer<'_> {
    pub(super) fn tag(&mut self, tag: Tag) {
        self.tag_with_priority(0, tag);
    }

    pub(super) fn tag_with_priority(&mut self, priority: i32, tag: Tag) {
        self.tags.push(TriageTag { priority, tag });
    }
}
