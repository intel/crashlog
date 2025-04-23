// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

//! C interface to the Intel Crash Log extraction and decoding functions.

#![allow(unused_variables)]

#[cfg(test)]
mod tests;

#[cfg(feature = "embedded_collateral_tree")]
use crate::collateral::{CollateralManager, EmbeddedTree};
use crate::crashlog::CrashLog;
use crate::node::{Node, NodeChildren, NodeType};
#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    collections::VecDeque,
    ffi::CString,
    string::{String, ToString},
};
use core::slice;
#[cfg(any(all(target_os = "uefi", feature = "extraction"), doc))]
use core::{ffi::c_void, ptr::NonNull};
#[cfg(not(feature = "std"))]
use core::{
    ffi::{CStr, c_char, c_uchar},
    ptr,
};
#[cfg(feature = "std")]
use std::{
    collections::VecDeque,
    ffi::{CStr, CString, c_char, c_uchar},
    ptr,
};
#[cfg(all(target_os = "uefi", feature = "extraction"))]
use uefi_raw::table::system::SystemTable;

/// Crash Log Global Context.
///
/// Contains all the resources required by the Crash Log library.
/// It can be initialized using the [`crashlog_init`] function and freed using the
/// [`crashlog_deinit`] function.
pub struct CrashLogContext {
    #[cfg(feature = "embedded_collateral_tree")]
    collateral_manager: CollateralManager<EmbeddedTree>,
}

/// Opaque type that represents an iterator over Crash Logs.
///
/// The actual content of the structure can be accessed from this structure using the
/// [`crashlog_next`] function.
pub struct CrashLogs {
    crashlogs: VecDeque<CrashLog>,
}

/// Opaque type that stores a representation of a Crash Log.
///
/// This structure is created by the [`crashlog_export_to_json`] and [`crashlog_export_to_binary`]
/// functions and must be released using the [`crashlog_release_export`] function.
///
/// The actual content of the structure can be accessed from this structure using the
/// [`crashlog_read_export`] function.
pub struct CrashLogExport {
    data: VecDeque<u8>,
}

/// Opaque type that represents an iterator over the children of a Crash Log register tree node.
///
/// This structure is created by the [`crashlog_get_node_children`] function.
///
/// The actual nodes can be accessed from this structure using the [`crashlog_get_next_node_child`]
/// function.
pub struct CrashLogNodeChildren<'a> {
    children: NodeChildren<'a>,
}

/// Initializes the Crash Log Decoder Context
///
/// This function allocates the memory needed to store the [`CrashLogContext`],
/// initializes it, and returns a pointer to it.
///
/// The value returned by this function must be passed to any subsequent calls made to the
/// other functions exposed by this library.
///
/// The memory allocated by this function can be released using the [`crashlog_deinit`] function.
///
/// # Errors
///
/// Returns a NULL pointer if the context cannot be initialized.
#[unsafe(no_mangle)]
pub extern "C" fn crashlog_init() -> *mut CrashLogContext {
    #[cfg(not(feature = "embedded_collateral_tree"))]
    {
        alloc(CrashLogContext {})
    }

    #[cfg(feature = "embedded_collateral_tree")]
    {
        if let Ok(collateral_manager) = CollateralManager::embedded_tree() {
            alloc(CrashLogContext { collateral_manager })
        } else {
            ptr::null_mut()
        }
    }
}

/// Creates a [`CrashLog`] object from a binary blob.
///
/// The binary blob pointed by the `data` argument can be a raw Crash Log region, a BERT
/// dump, or a CPER dump.
///
/// The memory allocated by this function can be freed using the [`crashlog_release`] function.
///
/// # Safety
///
/// This must be called with a pointer to a [`CrashLogContext`] that was earlier obtained by
/// calling the [`crashlog_init`] function.
///
/// The `data` pointer must point to a valid memory region that contains at most `size` bytes of
/// Crash Log data.
///
/// # Errors
///
/// Returns a `NULL` pointer if the binary blob does not encode any valid Crash Log records.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn crashlog_read_from_buffer(
    context: *mut CrashLogContext,
    data: *const u8,
    size: usize,
) -> *mut CrashLog {
    CrashLog::from_slice(unsafe { slice::from_raw_parts(data, size) })
        .map(alloc)
        .unwrap_or(ptr::null_mut())
}

/// Reads the Crash Log from the UEFI System Table.
///
/// # Errors
///
/// Returns a `NULL` pointer if the Crash Log records cannot be found.
#[cfg(all(target_os = "uefi", feature = "extraction"))]
#[unsafe(no_mangle)]
pub extern "C" fn crashlog_read_from_system_table(
    context: *mut CrashLogContext,
    system_table: *mut c_void,
) -> *mut CrashLog {
    CrashLog::from_system_table(NonNull::new(system_table as *mut SystemTable))
        .map(alloc)
        .unwrap_or(ptr::null_mut())
}

/// Reads the Crash Log records from the Windows Event Logs.
///
/// # Errors
///
/// Returns a `NULL` pointer if the Crash Log records cannot be found.
#[cfg(any(all(target_os = "windows", feature = "extraction"), doc))]
#[unsafe(no_mangle)]
pub extern "C" fn crashlog_read_from_windows_event_logs(
    context: *mut CrashLogContext,
) -> *mut CrashLogs {
    CrashLog::from_windows_event_logs(None)
        .map(|crashlogs| {
            alloc(CrashLogs {
                crashlogs: VecDeque::from(crashlogs),
            })
        })
        .unwrap_or(ptr::null_mut())
}

/// Reads the Crash Log reported through ACPI from the linux sysfs.
///
/// # Errors
///
/// Returns a `NULL` pointer if the Crash Log records cannot be found.
#[cfg(any(all(target_os = "linux", feature = "extraction"), doc))]
#[unsafe(no_mangle)]
pub extern "C" fn crashlog_read_from_linux_sysfs(context: *mut CrashLogContext) -> *mut CrashLog {
    CrashLog::from_linux_sysfs()
        .map(alloc)
        .unwrap_or(ptr::null_mut())
}

/// Returns the next Crash Log in the iterator.
///
/// The memory allocated for the iterator will be automatically freed by this function once all the
/// children have been returned.
///
/// # Safety
///
/// This must be called with a pointer to a [`CrashLogContext`] that was earlier obtained by
/// calling the [`crashlog_init`] function.
///
/// If the previous call to this function returns a `NULL` pointer, the iterator must not be used
/// again as it is freed automatically by this function.
///
/// # Errors
///
/// Returns a `NULL` pointer if one of the arguments is `NULL` or if no more Crash Logs is
/// available in the iterator.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn crashlog_next(
    context: *mut CrashLogContext,
    crashlogs: *mut CrashLogs,
) -> *mut CrashLog {
    if crashlogs.is_null() {
        return ptr::null_mut();
    }

    unsafe { &mut *crashlogs }
        .crashlogs
        .pop_front()
        .map(alloc)
        .unwrap_or_else(|| {
            free(crashlogs);
            ptr::null_mut()
        })
}

/// Decodes a [`CrashLog`] into a register tree.
///
/// Returns a [`Node`] object that represents the root node of the register tree.
///
/// The memory allocated by this function can be released using the [`crashlog_release_nodes`]
/// function.
///
/// # Safety
///
/// This must be called with a pointer to a [`CrashLogContext`] that was earlier obtained by
/// calling the [`crashlog_init`] function.
///
/// The `crashlog` pointer must be obtained using one of the `crashlog_read_from_*` functions.
///
/// # Errors
///
/// Returns a `NULL` pointer if one of the arguments is `NULL`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn crashlog_decode(
    context: *mut CrashLogContext,
    crashlog: *const CrashLog,
) -> *mut Node {
    if crashlog.is_null() || context.is_null() {
        return ptr::null_mut();
    }

    let context = unsafe { &mut *context };
    let crashlog = unsafe { &*crashlog };
    #[cfg(feature = "embedded_collateral_tree")]
    {
        alloc(crashlog.decode(&mut context.collateral_manager))
    }
    #[cfg(not(feature = "embedded_collateral_tree"))]
    {
        alloc(crashlog.decode_without_cm())
    }
}

/// Exports the Crash Log as binary blob.
///
/// The memory allocated by this function can be freed using the [`crashlog_release_export`]
/// function.
///
/// # Safety
///
/// This must be called with a pointer to a [`CrashLogContext`] that was earlier obtained by
/// calling the [`crashlog_init`] function.
///
/// The `crashlog` pointer must be valid and obtained using one of the `crashlog_read_*()`
/// functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn crashlog_export_to_binary(
    context: *mut CrashLogContext,
    crashlog: *const CrashLog,
) -> *mut CrashLogExport {
    if crashlog.is_null() {
        return ptr::null_mut();
    }

    let crashlog = unsafe { &*crashlog };
    alloc(CrashLogExport {
        data: VecDeque::from(crashlog.to_bytes()),
    })
}

/// Exports the Crash Log register tree ([`Node`]) as JSON file.
///
/// The memory allocated by this function can be freed using the [`crashlog_release_export`]
/// function.
///
/// # Safety
///
/// This must be called with a pointer to a [`CrashLogContext`] that was earlier obtained by
/// calling the [`crashlog_init`] function.
///
/// The `node` pointer must be obtained using the [`crashlog_decode`] function.
///
/// # Errors
///
/// Returns a `NULL` pointer if one of the arguments is `NULL` or if an error happens during the
/// generation of the JSON.
#[unsafe(no_mangle)]
#[cfg(feature = "serialize")]
pub unsafe extern "C" fn crashlog_export_to_json(
    context: *mut CrashLogContext,
    node: *const Node,
) -> *mut CrashLogExport {
    if node.is_null() {
        return ptr::null_mut();
    }

    let node = unsafe { &*node };

    serde_json::to_string(node)
        .ok()
        .map(|json| {
            alloc(CrashLogExport {
                data: VecDeque::from(json.into_bytes()),
            })
        })
        .unwrap_or(ptr::null_mut())
}

/// Reads the next chunk of the Crash Log export.
///
/// Writes the next `buffer_size` bytes of the Crash Log export in the buffer pointed by the
/// `buffer` argument.
///
/// Returns the amount of bytes that has been written into the buffer. Zero is returned if an error
/// occurred or if no more data is available.
///
/// # Safety
///
/// This must be called with a pointer to a [`CrashLogContext`] that was earlier obtained by
/// calling the [`crashlog_init`] function.
///
/// The `export` pointer must be obtained using the [`crashlog_export_to_json`] function or the
/// [`crashlog_export_to_binary`] function.
///
/// The `buffer` pointer must point to a valid writable memory region of `buffer_size` bytes.
/// The data written to this buffer is not nul-terminated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn crashlog_read_export(
    context: *mut CrashLogContext,
    export: *mut CrashLogExport,
    buffer: *mut u8,
    buffer_size: usize,
) -> usize {
    if export.is_null() {
        return 0;
    }

    let export = unsafe { &mut *export };
    let buffer = unsafe { slice::from_raw_parts_mut(buffer, buffer_size) };

    for (i, dst) in buffer.iter_mut().enumerate().take(buffer_size) {
        if let Some(byte) = export.data.pop_front() {
            *dst = byte;
        } else {
            return i;
        }
    }

    buffer_size
}

/// Writes the name of the Crash Log [`Node`] in the buffer pointed by the `buffer` argument.
///
/// A maximum of `buffer_size` bytes are written into the buffer, including the trailing nul
/// terminator.
///
/// Returns the amount of bytes required to store the full name of the register,
/// including the nul terminator. Returns zero if an error occurred while accessing the name of the
/// node.
///
/// # Safety
///
/// This must be called with a pointer to a [`CrashLogContext`] that was earlier obtained by
/// calling the [`crashlog_init`] function.
///
/// The `node` pointer must be obtained using the [`crashlog_decode`],
/// [`crashlog_get_next_node_child`], or the [`crashlog_get_node_by_path`] functions.
///
/// The `buffer` pointer must point to a writable memory region of `buffer_size` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn crashlog_get_node_name(
    context: *mut CrashLogContext,
    node: *const Node,
    buffer: *mut c_uchar,
    buffer_size: usize,
) -> usize {
    if node.is_null() {
        return 0;
    }

    let node = unsafe { &*node };

    if let Ok(name) = CString::new(&*node.name) {
        let src = name.into_bytes();
        let end = src.len().min(buffer_size - 1);
        let buffer = unsafe { slice::from_raw_parts_mut(buffer, buffer_size) };
        buffer[..end].copy_from_slice(&src[..end]);
        buffer[end] = 0;
        return src.len() + 1;
    }

    0
}

/// Copies the value stored in the Crash Log register tree node into the qword pointed by the
/// `value` argument.
///
/// Returns true if the value has been written. If an error happens or if the node does not store
/// any value, false is returned.
///
/// # Safety
///
/// This must be called with a pointer to a [`CrashLogContext`] that was earlier obtained by
/// calling the [`crashlog_init`] function.
///
/// The `node` pointer must be obtained using the [`crashlog_decode`],
/// [`crashlog_get_next_node_child`], or the [`crashlog_get_node_by_path`] functions.
///
/// The `value` pointer must point to a writable memory region of 8 bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn crashlog_get_node_value(
    context: *mut CrashLogContext,
    node: *const Node,
    value: *mut u64,
) -> bool {
    if node.is_null() || value.is_null() {
        return false;
    }

    let node = unsafe { &*node };
    let dst = unsafe { &mut *value };
    if let NodeType::Field { value } = node.kind {
        *dst = value;
        true
    } else {
        false
    }
}

/// Returns a iterator over the children of a Crash Log register tree node.
///
/// The actual children can be obtained from the iterator using the
/// [`crashlog_get_next_node_child`] function.
///
/// # Safety
///
/// This must be called with a pointer to a [`CrashLogContext`] that was earlier obtained by
/// calling the [`crashlog_init`] function.
///
/// The `node` pointer must be obtained using the [`crashlog_decode`],
/// [`crashlog_get_next_node_child`], or the [`crashlog_get_node_by_path`] functions.
///
/// # Errors
///
/// Returns a `NULL` pointer if one of the arguments is `NULL`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn crashlog_get_node_children<'a>(
    context: *mut CrashLogContext,
    node: *const Node,
) -> *mut CrashLogNodeChildren<'a> {
    if node.is_null() {
        return ptr::null_mut();
    }

    let node = unsafe { &*node };
    alloc(CrashLogNodeChildren {
        children: node.children(),
    })
}

/// Returns the next child of a Crash Log register tree node.
///
/// The memory allocated for the iterator will be automatically freed by this function once all the
/// children have been returned.
///
/// # Safety
///
/// This must be called with a pointer to a [`CrashLogContext`] that was earlier obtained by
/// calling the [`crashlog_init`] function.
///
/// The `children` pointer must be obtained using the [`crashlog_get_node_children`] function.
/// If the previous call to this function returns a `NULL` pointer, the iterator must not be used
/// again as it is freed automatically by this function.
///
/// # Errors
///
/// Returns a `NULL` pointer if one of the arguments is `NULL` or if no more node is available in
/// the iterator.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn crashlog_get_next_node_child(
    context: *mut CrashLogContext,
    children: *mut CrashLogNodeChildren,
) -> *const Node {
    if children.is_null() {
        return ptr::null();
    }

    unsafe { &mut *children }
        .children
        .next()
        .map(ptr::from_ref)
        .unwrap_or_else(|| {
            free(children);
            ptr::null()
        })
}

/// Returns the Crash Log register tree node located at the provided `path`.
///
/// # Safety
///
/// This must be called with a pointer to a [`CrashLogContext`] that was earlier obtained by
/// calling the [`crashlog_init`] function.
///
/// The `node` pointer must be obtained using the [`crashlog_decode`],
/// [`crashlog_get_next_node_child`], or the [`crashlog_get_node_by_path`] functions.
///
/// The `path` must be a valid nul-terminated string.
///
/// # Errors
///
/// Returns a `NULL` pointer if one of the arguments is `NULL` or if the node does not exist.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn crashlog_get_node_by_path(
    context: *mut CrashLogContext,
    node: *const Node,
    path: *const c_char,
) -> *const Node {
    if node.is_null() {
        return ptr::null();
    }

    let node = unsafe { &*node };
    let path = String::from_utf8_lossy(unsafe { CStr::from_ptr(path) }.to_bytes()).to_string();
    node.get_by_path(&path)
        .map(ptr::from_ref)
        .unwrap_or(ptr::null())
}

fn alloc<T>(data: T) -> *mut T {
    Box::into_raw(Box::new(data))
}

fn free<T>(ptr: *mut T) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        let _ = Box::from_raw(ptr);
    }
}

/// Releases the memory allocated for the Crash Log.
#[unsafe(no_mangle)]
pub extern "C" fn crashlog_release(crashlog: *mut CrashLog) {
    free(crashlog)
}

/// Releases the memory allocated for the Crash Log register tree.
///
/// The root node of the tree is expected to be passed to this function.
#[unsafe(no_mangle)]
pub extern "C" fn crashlog_release_nodes(node: *mut Node) {
    free(node)
}

/// Releases the memory allocated for the Crash Log export.
#[unsafe(no_mangle)]
pub extern "C" fn crashlog_release_export(node: *mut CrashLogExport) {
    free(node)
}

/// Releases the memory allocated for the Crash Log global context.
#[unsafe(no_mangle)]
pub extern "C" fn crashlog_deinit(ctx: *mut CrashLogContext) {
    free(ctx)
}
