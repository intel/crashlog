// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use alloc::format;
use intel_crashlog::CrashLog;
use log::{error, warn};
use uefi::CString16;
use uefi::fs::{FileSystem, Path, PathBuf};
use uefi::prelude::*;

fn generate_filename(crashlog: &CrashLog) -> Result<PathBuf, uefi::Error> {
    CString16::try_from(format!("{}.crashlog", crashlog.metadata).as_str())
        .map_err(|err| {
            warn!("Cannot convert generated Crash Log name: {err}");
            uefi::Error::from(Status::INVALID_PARAMETER)
        })
        .map(PathBuf::from)
}

fn write_file(path: &Path, data: &[u8]) -> Result<(), uefi::Error> {
    let image = uefi::boot::image_handle();
    let fs_protocol = uefi::boot::get_image_file_system(image)
        .inspect_err(|err| warn!("Failed to open FileSystem protocol: {err}"))?;
    let mut fs = FileSystem::new(fs_protocol);
    fs.write(path, data).map_err(|err| {
        error!("Failed to write file: {err}");
        match err {
            uefi::fs::Error::Io(err) => err.uefi_error,
            _ => uefi::Error::from(Status::INVALID_PARAMETER),
        }
    })
}

pub fn extract(output_path: Option<&Path>) -> Result<(), uefi::Error> {
    let crashlog = CrashLog::from_system_table(None).map_err(|err| {
        error!("Cannot read Crash Log data from System Table: {err}");
        uefi::Error::from(Status::NOT_FOUND)
    })?;

    let filename = match output_path {
        Some(path) => path,
        None => &generate_filename(&crashlog)?,
    };

    write_file(filename, &crashlog.to_bytes())
}
