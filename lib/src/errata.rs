// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use crate::header::Version;

/// Collection of Intel Crash Log erratas
pub struct Errata {
    /// Type0 server legacy header
    ///
    /// Some Intel(R) products in the server segment are using legacy Crash Log record headers with
    /// Type0, which has a different layout compared with the currently defined Type0 Header.
    ///
    pub type0_legacy_server: bool,
}

impl Errata {
    pub fn from_version(version: &Version) -> Self {
        let type0_legacy_server = version.header_type == 0 && version.product_id == 0x2f;

        Errata {
            type0_legacy_server,
        }
    }
}
