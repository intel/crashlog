// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Analyzer;
use super::mca::{Bank, MachineCheckErrorCode};
use super::tag::Tag;
use super::xq::{TransactionQueue, TransactionQueueState};
use crate::node::Node;

impl Analyzer<'_> {
    fn analyze_three_strike_timeout(&mut self, core: &Node) {
        let Some(bank) = core.get_by_path("thread.arch_state.mca.bank3") else {
            return;
        };

        let Some(bank) = Bank::from_node(bank) else {
            return;
        };

        if !bank.status.valid {
            return;
        }

        let MachineCheckErrorCode::InternalTimerError = bank.status.mcacod else {
            return;
        };

        let mut transaction_queue = TransactionQueueState::NotCaptured;

        if let Some(xq) = core.get("sq").or_else(|| core.get("xq"))
            && let Some(xq) = TransactionQueue::from_node(xq)
        {
            transaction_queue = xq.triage();
        }

        self.tag_with_priority(10, Tag::CoreTimeout { transaction_queue });
    }

    fn analyze_core_mca(&mut self, core: &Node) {
        let Some(mca) = core.get_by_path("thread.arch_state.mca") else {
            return;
        };

        for child in mca.children() {
            let Some(bank) = Bank::from_node(child) else {
                continue;
            };
            self.triage_mca_bank(&bank)
        }
    }

    pub(super) fn analyze_core(&mut self, node: &Node) {
        if node.get("thread").is_some() {
            self.analyze_three_strike_timeout(node);
            self.analyze_core_mca(node);
        } else {
            for child in node.children() {
                self.analyze_core(child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_core_record() -> Node {
        let mut root = Node::root();
        root.add(Node::record("pcore0"));

        let bank = root.create_hierarchy("pcore0.foo.bar.thread.arch_state.mca.bank3");
        bank.add(Node::field("status", 0xbe000000e1840400));

        root
    }

    #[test]
    fn no_xq_captured() {
        let root = build_core_record();

        let tags = Analyzer::default()
            .with_input(&root)
            .analyze()
            .tags
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        assert!(tags.contains(&"CORE_TIMEOUT.NO_XQ_INFO".to_string()));
    }

    #[test]
    fn no_stuck_transations() {
        let mut root = build_core_record();

        let entry = root.create_hierarchy("pcore0.foo.bar.xq.entry0");
        entry.add(Node::field("valid", 0));
        entry.add(Node::field("addr", 0x42));

        let tags = Analyzer::default()
            .with_input(&root)
            .analyze()
            .tags
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        assert!(tags.contains(&"CORE_TIMEOUT.NO_STUCK_TRANSACTIONS".to_string()));
    }

    #[test]
    fn single_stuck_transation() {
        let mut root = build_core_record();

        let entry = root.create_hierarchy("pcore0.foo.bar.xq.entry0");
        entry.add(Node::field("valid", 1));
        entry.add(Node::field("addr", 0x42));

        let tags = Analyzer::default()
            .with_input(&root)
            .analyze()
            .tags
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        assert!(tags.contains(&"CORE_TIMEOUT.SINGLE_STUCK_TRANSACTION.42H".to_string()));
    }

    #[test]
    fn multiple_stuck_transation() {
        let mut root = build_core_record();

        for i in 0..2 {
            let entry = root.create_hierarchy(&format!("pcore0.foo.bar.xq.entry{i}"));
            entry.add(Node::field("valid", 1));
            entry.add(Node::field("addr", 0x42));
        }

        let tags = Analyzer::default()
            .with_input(&root)
            .analyze()
            .tags
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        assert!(tags.contains(&"CORE_TIMEOUT.MULTIPLE_STUCK_TRANSACTIONS".to_string()));
    }
}
