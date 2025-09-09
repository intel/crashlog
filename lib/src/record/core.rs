// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Record;
use crate::Error;
#[cfg(feature = "collateral_manager")]
use crate::collateral::{CollateralManager, CollateralTree};
use crate::node::Node;
#[cfg(not(feature = "std"))]
use alloc::format;

impl Record {
    #[cfg(feature = "collateral_manager")]
    pub(super) fn decode_as_core_record<T: CollateralTree>(
        &self,
        cm: &mut CollateralManager<T>,
    ) -> Result<Node, Error> {
        let mut section = Node::section(self.header.record_type()?);

        for subsection_name in ["thread", "core"] {
            let decode_def = format!("layout_{subsection_name}.csv");
            let Ok(mut root) = self.decode_with_decode_def(cm, &decode_def, 0) else {
                continue;
            };

            if let Some(offset) = self.header.extended_record_offset() {
                for decode_def in ["layout_sq.csv", "layout_module.csv"] {
                    let Ok(extension) = self.decode_with_decode_def(cm, decode_def, offset) else {
                        continue;
                    };

                    root.merge(extension);
                    break;
                }
            }

            let Some(subsection) = root.get(subsection_name) else {
                continue;
            };
            let hierarchy = ["module", "core", "thread"]
                .into_iter()
                .filter_map(|level| {
                    subsection
                        .get_value_by_path(&format!("hdr.whoami.{level}_id"))
                        .map(|id| format!("{level}{id}"))
                });

            section.create_hierarchy_from_iter(hierarchy).merge(root);

            let mut root = Node::root();
            root.add(section);
            return Ok(root);
        }

        Err(Error::MissingDecodeDefinitions(self.header.version.clone()))
    }
}
