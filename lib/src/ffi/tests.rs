// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use crate::ffi::*;
use std::ffi::CStr;

#[test]
fn ffi() {
    let blob = std::fs::read("tests/samples/dummy.bert").unwrap();
    let ctx = crashlog_init();
    assert_ne!(ctx, std::ptr::null_mut());

    unsafe {
        let crashlog = crashlog_read_from_buffer(ctx, blob.as_ptr(), blob.len());
        assert_ne!(crashlog, std::ptr::null_mut());

        let root = crashlog_decode(ctx, crashlog);
        assert_ne!(root, std::ptr::null_mut());

        let children = crashlog_get_node_children(ctx, root);
        assert_ne!(children, std::ptr::null_mut());

        let mca = crashlog_get_next_node_child(ctx, children);
        assert_ne!(mca, std::ptr::null_mut());

        let mut buffer = [0u8; 64];
        let res = crashlog_get_node_name(ctx, mca, buffer.as_mut_ptr(), buffer.len());
        assert_eq!(res, 4);
        assert_eq!(CStr::from_ptr(buffer.as_ptr() as *const i8), c"mca");

        let res = crashlog_get_node_name(ctx, mca, buffer.as_mut_ptr(), 3);
        assert_eq!(res, 4);
        assert_eq!(CStr::from_ptr(buffer.as_ptr() as *const i8), c"mc");

        let processors = crashlog_get_next_node_child(ctx, children);
        assert_ne!(processors, std::ptr::null_mut());

        let res = crashlog_get_node_name(ctx, processors, buffer.as_mut_ptr(), buffer.len());
        assert_eq!(res, 11);
        assert_eq!(CStr::from_ptr(buffer.as_ptr() as *const i8), c"processors");
        let mut processors_value: u64 = 0;
        let value = crashlog_get_node_value(ctx, processors, &mut processors_value);
        assert!(!value);
        assert_eq!(processors_value, 0);

        let version_id = crashlog_get_node_by_path(ctx, root, c"mca.hdr.version".as_ptr());
        assert_ne!(version_id, std::ptr::null_mut());

        let mut version_id_value: u64 = 0;
        let res = crashlog_get_node_value(ctx, version_id, &mut version_id_value);
        assert!(res);
        assert_eq!(version_id_value, 0x7e07a301);

        crashlog_release_nodes(root);
        crashlog_release(crashlog);
        crashlog_deinit(ctx);
    }
}

#[test]
fn ffi_json() {
    let blob = std::fs::read("tests/samples/dummy.bert").unwrap();
    let crashlog = CrashLog::from_slice(&blob).unwrap();
    let mut cm = CollateralManager::embedded_tree().unwrap();
    let root = crashlog.decode(&mut cm);
    let reference = serde_json::to_string(&root).unwrap();

    unsafe {
        let ctx = crashlog_init();
        let crashlog = crashlog_read_from_buffer(ctx, blob.as_ptr(), blob.len());
        let node = crashlog_decode(ctx, crashlog);
        let export = crashlog_export_to_json(ctx, node);
        let mut json = String::new();

        let mut buffer = [0u8; 32];
        loop {
            let size = crashlog_read_export(ctx, export, buffer.as_mut_ptr(), buffer.len());
            if size == 0 {
                break;
            }
            json.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());
        }

        assert_eq!(reference, json);

        crashlog_release_nodes(node);
        crashlog_release_export(export);
        crashlog_deinit(ctx);
    }
}

#[test]
fn ffi_export() {
    let blob = std::fs::read("tests/samples/dummy.bert").unwrap();
    let crashlog = CrashLog::from_slice(&blob).unwrap();
    let reference = crashlog.to_bytes();

    unsafe {
        let ctx = crashlog_init();
        let crashlog = crashlog_read_from_buffer(ctx, blob.as_ptr(), blob.len());
        let export = crashlog_export_to_binary(ctx, crashlog);
        let mut data: Vec<u8> = Vec::new();

        let mut buffer = [0u8; 32];
        loop {
            let size = crashlog_read_export(ctx, export, buffer.as_mut_ptr(), buffer.len());
            if size == 0 {
                break;
            }
            data.extend(&buffer[..size]);
        }

        assert_eq!(reference, data);

        crashlog_release_export(export);
        crashlog_deinit(ctx);
    }
}
