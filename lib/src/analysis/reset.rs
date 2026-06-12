// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Analyzer;
use super::tag::Tag;
use crate::node::Node;
#[cfg(not(feature = "std"))]
use alloc::fmt;
#[cfg(feature = "std")]
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResetKind {
    Global,
    FirmwareGlobal,
    HostPartition,
}

impl fmt::Display for ResetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Global => write!(f, "GLOBAL_RESET"),
            Self::FirmwareGlobal => write!(f, "FIRMWARE_GLOBAL_RESET"),
            Self::HostPartition => write!(f, "HOST_PARTITION_RESET"),
        }
    }
}

impl Analyzer<'_> {
    fn report_reset_cause(&mut self, kind: ResetKind, node: &Node) {
        for child in node.children() {
            if let Some(value) = child.value() {
                if value == 0 {
                    continue;
                }

                self.tag(Tag::ResetCause {
                    kind,
                    cause: child.name.to_uppercase(),
                });
            }
        }
    }

    pub(super) fn analyze_pmc_rst(&mut self, node: &Node) {
        if let Some(pmc_rst) = node.get("pmc_rst").or_else(|| node.get("pmc")) {
            self.analyze_pmc_rst(pmc_rst);
            return;
        }

        if let Some(gblrst_cause) = node.get("gblrst_cause_0") {
            self.report_reset_cause(ResetKind::Global, gblrst_cause);
        }

        if let Some(fw_gblrst_cause) = node.get("fw_gblrst_cause_0") {
            self.report_reset_cause(ResetKind::FirmwareGlobal, fw_gblrst_cause);
        }

        if let Some(host_pr_cause) = node.get("host_pr_cause_0") {
            self.report_reset_cause(ResetKind::HostPartition, host_pr_cause);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crashlog_reason() {
        let mut root = Node::root();
        root.add(Node::record("pmc_rst"));
        let reg = root.create_hierarchy("pmc_rst.gblrst_cause_0");
        reg.add(Node::field("foo", 1));
        let reg = root.create_hierarchy("pmc_rst.host_pr_cause_0");
        reg.add(Node::field("bar", 1));

        let tags = Analyzer::default().with_input(&root).analyze().tags;

        assert!(
            tags.iter()
                .any(|t| t.to_string() == "RESET_CAUSE.GLOBAL_RESET.FOO")
        );
        assert!(
            tags.iter()
                .any(|t| t.to_string() == "RESET_CAUSE.HOST_PARTITION_RESET.BAR")
        );
    }
}
