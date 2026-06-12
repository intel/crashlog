// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Analyzer;
use super::tag::Tag;
use crate::node::Node;
#[cfg(not(feature = "std"))]
use alloc::fmt;
#[cfg(feature = "std")]
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub enum CrashLogReason {
    Raw(u32),
}

impl fmt::Display for CrashLogReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw(value) => write!(f, "{value:X}H"),
        }
    }
}

impl Analyzer<'_> {
    pub(super) fn analyze_reason(&mut self, node: &Node) {
        if let Some(reason) = node.get_by_path("hdr.reason")
            && let Some(value) = reason.value()
        {
            self.tag(Tag::CrashLogReason {
                record: node.name.to_uppercase(),
                reason: CrashLogReason::Raw(value as u32),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crashlog_reason() {
        let mut root = Node::root();
        root.add(Node::record("punit"));
        let hdr = root.create_hierarchy("punit.hdr");
        hdr.add(Node::field("reason", 0x42));

        let tags = Analyzer::default().with_input(&root).analyze().tags;

        assert!(
            tags.iter()
                .any(|t| t.to_string() == "CRASHLOG_REASON.PUNIT.42H")
        );
    }
}
