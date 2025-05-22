// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;
use std::path::Path;

pub fn decode<T: CollateralTree, O: std::io::Write>(
    cm: &mut CollateralManager<T>,
    input: &Path,
    output: O,
) -> Result<(), Error> {
    let crashlog = CrashLog::from_slice(&std::fs::read(input)?)?;
    let nodes = crashlog.decode(cm);
    Ok(serde_json::to_writer_pretty(output, &nodes)?)
}
