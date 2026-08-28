// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;
use intel_crashlog::source::Capability;

pub fn rearm(sources: Vec<CrashLogSource>) -> Result<(), Error> {
    run_control_command(sources, Capability::Rearm, CrashLogSource::rearm)
}

pub fn trigger(sources: Vec<CrashLogSource>) -> Result<(), Error> {
    run_control_command(sources, Capability::Trigger, CrashLogSource::trigger)
}

pub fn clear(sources: Vec<CrashLogSource>) -> Result<(), Error> {
    run_control_command(sources, Capability::Clear, CrashLogSource::clear)
}

pub fn enable(sources: Vec<CrashLogSource>) -> Result<(), Error> {
    run_control_command(sources, Capability::EnableDisable, CrashLogSource::enable)
}

pub fn disable(sources: Vec<CrashLogSource>) -> Result<(), Error> {
    run_control_command(sources, Capability::EnableDisable, CrashLogSource::disable)
}

fn run_control_command<F>(
    sources: Vec<CrashLogSource>,
    capability: Capability,
    control: F,
) -> Result<(), Error>
where
    F: Fn(&CrashLogSource) -> Result<(), Error>,
{
    let mut cmd_success = false;
    let mut first_error = None;

    let control_sources = if sources.is_empty() {
        CrashLogSource::discover_distinct()
            .into_iter()
            .filter(|src| src.capabilities().contains(&capability))
            .collect()
    } else {
        sources
    };

    if control_sources.is_empty() {
        return Err(Error::NoCrashLogSourceFound);
    }

    for source in control_sources {
        match control(&source) {
            Ok(()) => cmd_success = true,
            Err(err) => {
                log::warn!("Error while running {capability} command on {source}: {err}");
                if first_error.is_none() {
                    first_error = Some(err);
                }
            }
        }
    }

    if !cmd_success {
        return Err(first_error.unwrap_or(Error::InternalError));
    }

    Ok(())
}
