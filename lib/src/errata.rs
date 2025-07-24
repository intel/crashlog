// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

//! Collection of erratas present in the Intel(R) Crash Log technology
//!
//! The `erratas` collected in this module are reflecting the errors or corrections that had to be
//! implemented in some products regarding the structure of the Crash Log record layout.

use crate::header::{Version, record_types};

/// Collection of Intel Crash Log erratas
pub struct Errata {
    /// Type0 server legacy header
    ///
    /// Some Intel(R) products in the server segment are using legacy Crash Log record headers with
    /// Type0, which has a different layout compared with the currently defined Type0 Header.
    ///
    pub type0_legacy_server: bool,

    /// Type0 server legacy header box record
    ///
    /// Some Intel(R) products in the server segment that are using the legacy Crash Log record
    /// header with Type0 are using the PCORE record type with the same functionality as a BOX
    /// record.
    pub type0_legacy_server_box: bool,

    /// Core record using record size in bytes
    ///
    /// The Crash Log headers have their sizes in DWORDs, but for some products that are using
    /// ECORE and PCORE Crash Log records, their sizes are written in bytes.
    pub core_record_size_bytes: bool,
}

impl Errata {
    pub fn from_version(version: &Version) -> Self {
        let type0_legacy_server = version.header_type == 0 && version.product_id == 0x2f;
        let type0_legacy_server_box =
            type0_legacy_server && version.record_type == record_types::PCORE;
        let core_record_size_bytes = !type0_legacy_server
            && ((version.record_type == record_types::ECORE && version.product_id < 0x96)
                || (version.record_type == record_types::PCORE && version.product_id < 0x71));

        Errata {
            type0_legacy_server,
            type0_legacy_server_box,
            core_record_size_bytes,
        }
    }
}
