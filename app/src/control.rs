// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;

pub fn trigger(sources: Vec<CrashLogSource>) -> Result<(), Error> {
    control_command(sources, CrashLogSource::trigger)
}

pub fn enable(sources: Vec<CrashLogSource>) -> Result<(), Error> {
    control_command(sources, CrashLogSource::enable)
}

pub fn disable(sources: Vec<CrashLogSource>) -> Result<(), Error> {
    control_command(sources, CrashLogSource::disable)
}

fn control_command<F>(sources: Vec<CrashLogSource>, control: F) -> Result<(), Error>
where
    F: Fn(&CrashLogSource) -> Result<(), Error>,
{
    for source in sources {
        control(&source)?;
    }

    Ok(())
}
