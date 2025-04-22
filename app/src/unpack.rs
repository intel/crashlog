// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;
use std::path::{Path, PathBuf};

fn write_file(path: &Path, slice: &[u8]) -> Result<(), Error> {
    println!("{}", path.display());
    Ok(
        std::fs::write(path, slice)
            .inspect_err(|err| log::error!("Failed to write file: {err}"))?,
    )
}

#[cfg(target_os = "windows")]
fn unpack_evtx(evtx: &Path) -> Result<(), Error> {
    let crashlogs = CrashLog::from_windows_event_logs(Some(evtx))
        .inspect_err(|err| log::error!("Failed to unpack EVTX file: {err}"))?;
    let mut path = PathBuf::from(evtx);
    for (i, crashlog) in crashlogs.iter().enumerate() {
        if let Some(filename) = path.file_stem() {
            path.set_file_name(format!(
                "{}-{i}.crashlog",
                PathBuf::from(filename).display()
            ))
        }
        write_file(&path, &crashlog.to_bytes())?;
    }
    Ok(())
}

fn unpack_crashlog(input_file: &Path) -> Result<(), Error> {
    let crashlog = CrashLog::from_slice(&std::fs::read(input_file)?)?;
    let path_prefix = PathBuf::from(input_file);

    for (i, region) in crashlog.regions.iter().enumerate() {
        let mut path = path_prefix.clone(); // Clone the original path prefix for each iteration
        if let Some(filename) = path.file_stem() {
            path.set_file_name(format!(
                "{}_region{i}.crashlog",
                PathBuf::from(filename).display()
            ))
        }
        write_file(&path, &region.to_bytes())?;
    }
    Ok(())
}

pub fn unpack(input_file: &Path) -> Result<(), Error> {
    #[cfg(target_os = "windows")]
    {
        if let Some("evtx") = input_file.extension().and_then(|p| p.to_str()) {
            return unpack_evtx(input_file);
        }
    }
    unpack_crashlog(input_file)
}
