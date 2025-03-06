// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use crate::metadata;
use crate::CrashLog;
use std::alloc::{alloc, dealloc, Layout};
use std::ffi::c_void;
use std::ops::{Deref, Drop};
use std::path::Path;
use std::slice;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::EventLog::EVT_VARIANT;
use windows::Win32::System::EventLog::*;
use windows::Win32::System::Time::FileTimeToSystemTime;

pub struct EvtHandle(EVT_HANDLE);

impl Drop for EvtHandle {
    fn drop(&mut self) {
        let _ = unsafe { EvtClose(**self) };
    }
}

impl Deref for EvtHandle {
    type Target = EVT_HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn evt_query(path: PCWSTR, query: PCWSTR, flags: u32) -> Result<EvtHandle> {
    unsafe { EvtQuery(None, path, query, flags).map(EvtHandle) }
}

fn evt_next(result_set: &EvtHandle, count: usize) -> Result<Vec<EvtHandle>> {
    let mut events = vec![0; count];
    let mut event_count: u32 = 0;

    let res = unsafe {
        EvtNext(
            **result_set,
            events.as_mut_slice(),
            u32::MAX,
            0,
            &mut event_count as *mut u32,
        )
    };

    if let Err(error) = res.as_ref() {
        if WIN32_ERROR::from_error(error) == Some(ERROR_NO_MORE_ITEMS) {
            return Ok(vec![]);
        }
        return Err(error.clone());
    }

    events.resize(event_count as usize, 0);
    Ok(events
        .iter()
        .map(|event| EvtHandle(EVT_HANDLE(*event)))
        .collect())
}

pub struct EvtRenderedValues {
    buffer: *mut u8,
    layout: Layout,
    property_count: u32,
}

impl Drop for EvtRenderedValues {
    fn drop(&mut self) {
        unsafe { dealloc(self.buffer, self.layout) }
    }
}

impl EvtRenderedValues {
    fn new(buffer_size: usize) -> EvtRenderedValues {
        let layout = Layout::array::<u8>(buffer_size)
            .expect("Failed to create layout for EvtRenderedValues");
        let buffer = unsafe { alloc(layout) };

        EvtRenderedValues {
            buffer,
            layout,
            property_count: 0,
        }
    }

    fn values(&self) -> &[EVT_VARIANT] {
        unsafe {
            slice::from_raw_parts::<EVT_VARIANT>(
                self.buffer as *const EVT_VARIANT,
                self.property_count as usize,
            )
        }
    }
}

fn evt_render_values(context: &EvtHandle, event: &EvtHandle) -> Result<Option<EvtRenderedValues>> {
    let mut buffer_used = 0;
    let mut property_count = 0;

    let res = unsafe {
        EvtRender(
            **context,
            **event,
            EvtRenderEventValues.0,
            0,
            None,
            &mut buffer_used,
            &mut property_count,
        )
    };

    if let Err(error) = res.as_ref() {
        if WIN32_ERROR::from_error(error) != Some(ERROR_INSUFFICIENT_BUFFER) {
            return Err(error.clone());
        }

        let mut values = EvtRenderedValues::new(buffer_used as usize);

        unsafe {
            EvtRender(
                **context,
                **event,
                EvtRenderEventValues.0,
                values.layout.size() as u32,
                Some(values.buffer as *mut c_void),
                &mut buffer_used,
                &mut values.property_count,
            )?
        };

        return Ok(Some(values));
    }

    Ok(None)
}

fn metadata_from_evt_values(
    filetime: EVT_VARIANT,
    computer: EVT_VARIANT,
) -> Result<metadata::Metadata> {
    let mut time = SYSTEMTIME::default();
    unsafe {
        let filetime = FILETIME {
            dwHighDateTime: (filetime.Anonymous.FileTimeVal >> 32) as u32,
            dwLowDateTime: (filetime.Anonymous.FileTimeVal & 0xFFFFFFFF) as u32,
        };
        FileTimeToSystemTime(&filetime as *const FILETIME, &mut time as *mut SYSTEMTIME)?
    }

    Ok(metadata::Metadata {
        time: Some(metadata::Time {
            year: time.wYear,
            month: time.wMonth as u8,
            day: time.wDay as u8,
            hour: time.wHour as u8,
            minute: time.wMinute as u8,
        }),
        computer: unsafe { computer.Anonymous.StringVal.to_string().ok() },
    })
}

fn query_crashlogs(path: PCWSTR, query: PCWSTR, query_flags: u32) -> Result<Vec<CrashLog>> {
    let query_handle = evt_query(path, query, query_flags)?;

    let context = unsafe {
        EvtCreateRenderContext(
            Some(&[
                w!("Event/EventData/Data[@Name=\"RawData\"]"),
                w!("Event/System/TimeCreated/@SystemTime"),
                w!("Event/System/Computer"),
            ]),
            EvtRenderContextValues.0,
        )
        .map(EvtHandle)
    }?;

    let mut crashlogs = Vec::new();

    loop {
        let events = evt_next(&query_handle, 1)?;
        if events.is_empty() {
            break;
        }

        let values = evt_render_values(&context, &events[0])?;
        if let Some(values) = values {
            let values = values.values();

            let binary = unsafe {
                slice::from_raw_parts::<u8>(values[0].Anonymous.BinaryVal, values[0].Count as usize)
            };

            match CrashLog::from_slice(binary) {
                Ok(mut crashlog) => {
                    crashlog.metadata = metadata_from_evt_values(values[1], values[2])?;
                    crashlogs.push(crashlog)
                }
                Err(err) => {
                    log::warn!("Error while decoding Crash Log read from Event Logs: {err}")
                }
            }
        }
    }

    Ok(crashlogs)
}

pub(crate) fn get_crashlogs_from_event_logs(path: Option<&Path>) -> Result<Vec<CrashLog>> {
    let evtx_path_hstring = path.map(HSTRING::from);
    let evtx_path = evtx_path_hstring
        .as_ref()
        .map(|hstring| PCWSTR(hstring.as_ptr()));
    let query_flags = if path.is_some() {
        EvtQueryFilePath.0
    } else {
        EvtQueryChannelPath.0
    };

    let mut crashlogs = query_crashlogs(
        evtx_path.unwrap_or(w!("Microsoft-Windows-Kernel-WHEA/Errors")),
        w!("*[System[Provider[@Name=\"Microsoft-Windows-Kernel-WHEA\"]]]"),
        query_flags,
    )?;
    log::info!(
        "Extracted {} Crash Logs from Application Event Logs",
        crashlogs.len()
    );

    let mut system_crashlogs = query_crashlogs(
        evtx_path.unwrap_or(w!("System")),
        w!("*[System[Provider[@Name=\"Microsoft-Windows-WHEA-Logger\"]]]"),
        query_flags,
    )?;
    log::info!(
        "Extracted {} Crash Logs from Windows Event Logs",
        system_crashlogs.len()
    );

    crashlogs.append(&mut system_crashlogs);
    Ok(crashlogs)
}
