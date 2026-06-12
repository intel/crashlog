// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;
use std::path::Path;

fn triage_file<T: CollateralTree>(
    cm: &mut CollateralManager<T>,
    input: &Path,
) -> Result<(), Error> {
    let crashlog = CrashLog::from_slice(&std::fs::read(input)?)?;
    let nodes = crashlog.decode(cm);
    let tags = Analyzer::default().with_input(&nodes).analyze().tags;

    for tag in tags {
        println!("{}", tag);
    }
    Ok(())
}

pub(crate) fn triage_files<T, P>(cm: &mut CollateralManager<T>, input_files: &[P])
where
    T: CollateralTree,
    P: AsRef<Path>,
{
    for input_file in input_files {
        if input_files.len() > 1 {
            println!("\n{}:", input_file.as_ref().display());
        }
        if let Err(err) = triage_file(cm, input_file.as_ref()) {
            log::error!("Error: {err}")
        }
    }
}
