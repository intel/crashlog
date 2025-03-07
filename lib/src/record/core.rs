// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Record;
use crate::Error;
#[cfg(feature = "collateral_manager")]
use crate::collateral::{CollateralManager, CollateralTree};
use crate::node::{Node, NodeType};
#[cfg(not(feature = "std"))]
use alloc::format;

impl Record {
    #[cfg(feature = "collateral_manager")]
    pub(super) fn decode_as_core_record<T: CollateralTree>(
        &self,
        cm: &mut CollateralManager<T>,
    ) -> Result<Node, Error> {
        let mut section = Node::section("core");

        for decode_def in ["layout_thread.csv", "layout_core.csv"] {
            if let Ok(thread) = self.decode_with_decode_def(cm, decode_def, 0) {
                let core_id = thread
                    .children()
                    .next()
                    .and_then(|child| child.get_by_path("hdr.whoami.core_id"))
                    .map(|core_id| &core_id.kind);
                if let Some(NodeType::Field { value }) = core_id {
                    section.name = format!("core{value}");
                }
                section.merge(thread);
            }
        }

        if let Some(offset) = self.header.extended_record_offset() {
            for decode_def in ["layout_sq.csv", "layout_module.csv"] {
                if let Ok(node) = self.decode_with_decode_def(cm, decode_def, offset) {
                    section.merge(node);
                    break;
                }
            }
        }

        let mut root = Node::root();
        root.add(section);
        Ok(root)
    }
}
