// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(target_family = "windows")]
mod win;

use crate::CrashLog;
use crate::error::Error;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(all(target_family = "windows", feature = "std"))]
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct EventLog {
    #[cfg(all(target_family = "windows", feature = "std"))]
    path: Option<PathBuf>,
}

impl EventLog {
    #[cfg(all(target_family = "windows", feature = "std"))]
    fn from_path(path: &Path) -> Self {
        Self {
            path: Some(path.to_owned()),
        }
    }

    pub fn is_available(&self) -> bool {
        cfg!(target_family = "windows")
    }

    pub fn extract(&self) -> Result<Vec<CrashLog>, Error> {
        #[cfg(all(target_family = "windows", feature = "std"))]
        {
            match win::extract_crashlogs(self.path.as_deref()) {
                Ok(crashlogs) => return Ok(crashlogs),
                Err(err) => log::error!("Cannot extract Crash Log from EventLog: {err}"),
            }
        }

        Err(Error::NoCrashLogFound)
    }
}

impl CrashLog {
    #[cfg(target_family = "windows")]
    pub fn from_event_logs(path: Option<&Path>) -> Result<Vec<Self>, Error> {
        let event_log = if let Some(path) = path {
            EventLog::from_path(path)
        } else {
            EventLog::default()
        };
        event_log.extract()
    }
}
