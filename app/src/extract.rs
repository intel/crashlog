// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

#![allow(unused_assignments)]

use intel_crashlog::prelude::*;
use std::path::{Path, PathBuf};

pub fn extract(output_path: Option<&Path>, sources: Vec<CrashLogSource>) {
    let mut result: Vec<CrashLog> = Vec::default();

    if sources.is_empty() {
        #[cfg(target_os = "windows")]
        match CrashLog::from_windows_event_logs(None) {
            Ok(crashlogs) => result = crashlogs,
            Err(err) => log::error!("Error while extracting from Event Log: {err}"),
        }

        #[cfg(target_os = "linux")]
        {
            result = [CrashLog::from_acpi_sysfs(), CrashLog::from_pmt_sysfs()]
                .into_iter()
                .filter_map(|crashlog| crashlog.ok())
                .collect::<Vec<CrashLog>>();
        }
    } else {
        for source in sources.iter() {
            match source.extract() {
                Ok(crashlogs) => result.extend(crashlogs),
                Err(err) => log::error!("Error while extracting from {source}: {err}"),
            }
        }
    }

    if result.is_empty() {
        log::error!("{}", Error::NoCrashLogFound);
    }

    for (i, crashlog) in result.iter().enumerate() {
        let mut path = if let Some(output_path) = output_path {
            let mut path = output_path.to_path_buf();
            if output_path.is_dir() {
                path.push(format!("{}.crashlog", crashlog.metadata))
            }
            path
        } else {
            PathBuf::from(format!("{}.crashlog", crashlog.metadata))
        };

        if result.len() > 1
            && let Some(filename) = path.file_stem()
        {
            path.set_file_name(format!(
                "{}-{i}.crashlog",
                PathBuf::from(filename).display()
            ))
        }

        println!("{}", path.display());
        if let Err(err) = std::fs::write(path, crashlog.to_bytes()) {
            log::error!("Failed to write Crash Log file: {err}");
        }
    }
}
