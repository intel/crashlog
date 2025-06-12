// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

#![allow(unused_assignments)]

use intel_crashlog::prelude::*;
use std::path::{Path, PathBuf};

pub fn extract(output_path: Option<&Path>) {
    let mut result: Result<Vec<CrashLog>, Error> = Err(Error::NoCrashLogFound);

    #[cfg(target_os = "windows")]
    {
        result = CrashLog::from_windows_event_logs(None);
    }
    #[cfg(target_os = "linux")]
    {
        result = CrashLog::from_linux_sysfs();
    }

    match result {
        Ok(crashlogs) => {
            for (i, crashlog) in crashlogs.iter().enumerate() {
                let mut path = if let Some(output_path) = output_path {
                    let mut path = output_path.to_path_buf();
                    if output_path.is_dir() {
                        path.push(format!("{}.crashlog", crashlog.metadata))
                    }
                    path
                } else {
                    PathBuf::from(format!("{}.crashlog", crashlog.metadata))
                };

                if crashlogs.len() > 1
                    && let Some(filename) = path.file_stem()
                {
                    path.set_file_name(format!(
                        "{}-{i}.crashlog",
                        PathBuf::from(filename).display()
                    ))
                }

                println!("{}", path.display());
                std::fs::write(path, crashlog.to_bytes()).expect("Failed to write Crash Log file")
            }
        }
        Err(err) => log::error!("Failed to extract Crash Log: {err}"),
    }
}
