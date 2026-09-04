// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use crate::CrashLog;
use crate::metadata::{Metadata, Time};
use std::alloc::{Layout, alloc, dealloc};
use std::collections::{BTreeMap, HashMap};
use std::ffi::c_void;
use std::ops::{Deref, Drop};
use std::path::Path;
use std::slice;
use windows::Win32::Foundation::*;
use windows::Win32::System::EventLog::EVT_VARIANT;
use windows::Win32::System::EventLog::*;
use windows::Win32::System::Time::FileTimeToSystemTime;
use windows::core::*;

struct EvtHandle(EVT_HANDLE);

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

struct EvtRenderedValues {
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
            Some(**context),
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
                Some(**context),
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

#[derive(Default)]
struct EvtRecord {
    id: u32,
    metadata: Metadata,
    chunks: BTreeMap<u32, Vec<u8>>,
}

impl EvtRecord {
    fn new(id: u32) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    fn add_chunk(&mut self, seq_num: u32, chunk: &[u8]) {
        if self.chunks.insert(seq_num, chunk.to_vec()).is_some() {
            log::warn!(
                "Chunk {seq_num} of the record {} is defined several times in the Event Log.",
                self.id
            );
        }
    }

    fn set_time(&mut self, filetimeval: u64) {
        let mut time = SYSTEMTIME::default();
        let filetime = FILETIME {
            dwHighDateTime: (filetimeval >> 32) as u32,
            dwLowDateTime: (filetimeval & 0xFFFFFFFF) as u32,
        };
        let res = unsafe {
            FileTimeToSystemTime(&filetime as *const FILETIME, &mut time as *mut SYSTEMTIME)
        };

        if let Err(err) = res {
            log::warn!(
                "Failed to convert filetime to system time for event record {}: {err}",
                self.id
            );
            return;
        }

        self.metadata.time = Some(Time {
            year: time.wYear,
            month: time.wMonth as u8,
            day: time.wDay as u8,
            hour: time.wHour as u8,
            minute: time.wMinute as u8,
            second: time.wSecond as u8,
            millisecond: time.wMilliseconds,
        });
    }

    fn set_computer(&mut self, computer: PCWSTR) {
        self.metadata.computer = unsafe { computer.to_string().ok() };
    }

    fn into_crashlog(self) -> Option<CrashLog> {
        let mut binary = Vec::new();

        for (i, (seq_num, chunk)) in self.chunks.into_iter().enumerate() {
            if i as u32 != seq_num {
                log::warn!(
                    "Event record {} is incomplete. Chunk {i} is missing.",
                    self.id
                );
                return None;
            }

            binary.extend(chunk);
        }

        let mut crashlog = CrashLog::from_slice(&binary)
            .inspect_err(|err| {
                log::warn!("Error while decoding Crash Log read from Event Logs: {err}")
            })
            .ok()?;

        crashlog.metadata = self.metadata.clone();
        Some(crashlog)
    }
}

fn query_crashlogs(path: PCWSTR, query: PCWSTR, query_flags: u32) -> Result<Vec<CrashLog>> {
    let query_handle = evt_query(path, query, query_flags)?;

    let context = unsafe {
        EvtCreateRenderContext(
            Some(&[
                w!("Event/EventData/Data[@Name=\"RawData\"]"),
                w!("Event/EventData/Data[@Name=\"RecordId\"]"),
                w!("Event/EventData/Data[@Name=\"SeqNum\"]"),
                w!("Event/System/TimeCreated/@SystemTime"),
                w!("Event/System/Computer"),
            ]),
            EvtRenderContextValues.0,
        )
        .map(EvtHandle)
    }?;

    let mut records = HashMap::new();

    loop {
        let events = evt_next(&query_handle, 1)?;
        let Some(event) = events.first() else {
            break;
        };
        let values = evt_render_values(&context, event)?;

        if let Some(values) = values {
            let values = values.values();

            // Extract all the rendered values
            let binary = unsafe {
                slice::from_raw_parts::<u8>(values[0].Anonymous.BinaryVal, values[0].Count as usize)
            };
            let record_id = unsafe { values[1].Anonymous.UInt32Val };
            let seq_num = unsafe { values[2].Anonymous.UInt32Val };
            let filetimeval = unsafe { values[3].Anonymous.FileTimeVal };
            let computer = unsafe { values[4].Anonymous.StringVal };

            records.entry(record_id).or_insert_with(|| {
                let mut record = EvtRecord::new(record_id);
                record.set_time(filetimeval);
                record.set_computer(computer);
                record
            });

            if let Some(record) = records.get_mut(&record_id) {
                record.add_chunk(seq_num, binary);
            }
        }
    }

    Ok(records
        .into_values()
        .filter_map(|record| record.into_crashlog())
        .collect())
}

pub(super) fn extract_crashlogs(path: Option<&Path>) -> Result<Vec<CrashLog>> {
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
