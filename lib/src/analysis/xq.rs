// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use crate::node::Node;
#[cfg(not(feature = "std"))]
use alloc::{fmt, vec::Vec};
#[cfg(feature = "std")]
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub enum TransactionQueueState {
    MultipleStuckTransaction,
    SingleStuckTransaction(u64),
    NoStuckTransaction,
    NotCaptured,
}

impl fmt::Display for TransactionQueueState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotCaptured => write!(f, "NO_XQ_INFO"),
            Self::NoStuckTransaction => write!(f, "NO_STUCK_TRANSACTIONS"),
            Self::SingleStuckTransaction(addr) => write!(f, "SINGLE_STUCK_TRANSACTION.{addr:X}H"),
            Self::MultipleStuckTransaction => write!(f, "MULTIPLE_STUCK_TRANSACTIONS"),
        }
    }
}

#[derive(Clone)]
pub struct Transaction {
    pub valid: bool,
    pub address: u64,
}

impl Transaction {
    pub fn from_node(node: &Node) -> Option<Self> {
        let basename = node.name.as_str().trim_end_matches(char::is_numeric);
        if basename != "entry" && basename != "xq_" {
            return None;
        }

        Some(Self {
            valid: node
                .get("valid")
                .and_then(|valid| valid.value())
                .map(|valid| valid != 0)
                .unwrap_or(false),
            address: node.get("addr").and_then(|addr| addr.value()).or_else(|| {
                node.get("addr_51_6")
                    .and_then(|addr| addr.value())
                    .map(|value| value << 6)
            })?,
        })
    }
}

#[derive(Default)]
pub struct TransactionQueue {
    entries: Vec<Transaction>,
}

impl TransactionQueue {
    pub fn from_node(node: &Node) -> Option<Self> {
        let mut xq = TransactionQueue::default();

        for child in node.children() {
            if let Some(entry) = Transaction::from_node(child) {
                xq.entries.push(entry);
            }
        }

        Some(xq)
    }

    pub fn triage(&self) -> TransactionQueueState {
        let mut stuck_transaction = None;

        for entry in self.entries.iter() {
            if !entry.valid {
                continue;
            }

            if stuck_transaction.is_some() {
                return TransactionQueueState::MultipleStuckTransaction;
            } else {
                stuck_transaction = Some(entry.clone());
            }
        }

        if let Some(stuck_transaction) = stuck_transaction {
            TransactionQueueState::SingleStuckTransaction(stuck_transaction.address)
        } else {
            TransactionQueueState::NoStuckTransaction
        }
    }
}
