// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

//! Convenience re-export of common structs

#[cfg(feature = "analysis")]
pub use crate::analysis::Analyzer;
#[cfg(feature = "collateral_manager")]
pub use crate::collateral::{CollateralManager, CollateralTree};
pub use crate::crashlog::CrashLog;
pub use crate::error::Error;
pub use crate::header::Header;
pub use crate::node::{Node, NodeType};
pub use crate::record::Record;
pub use crate::region::Region;
pub use crate::source::CrashLogSource;
