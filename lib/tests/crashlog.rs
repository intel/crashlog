// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;

#[test]
fn crashlog_from_slice() {
    let crashlog = CrashLog::from_slice(
        &std::fs::read("tests/samples/three_strike_timeout.crashlog").unwrap(),
    );
    assert!(crashlog.is_ok());
}

#[test]
fn crashlog_decode() {
    let bert = std::fs::read("tests/samples/dummy.bert").unwrap();
    let crashlog = CrashLog::from_slice(&bert).unwrap();

    let mut cm = CollateralManager::embedded_tree().unwrap();
    let root = crashlog.decode(&mut cm);
    let mut children = root.children();

    assert_eq!(children.next().unwrap().name, "mca");
    assert_eq!(children.next().unwrap().name, "processors");

    let mca = root.get_by_path("mca");
    assert!(mca.is_some());

    let crashlog_agent = root.get_by_path("processors.cpu0.io0.crashlog_agent");
    assert!(crashlog_agent.is_some());
}

#[test]
fn invalid_box_record() {
    let data = [0x0, 0x0, 0x0, 0x3d, 0x1, 0x0, 0x0, 0x0, 0x0, 0xa];
    let crashlog = CrashLog::from_slice(&data);
    assert!(crashlog.is_ok());
}

#[test]
fn core_box_header_type6() {
    let data = std::fs::read("tests/samples/three_strike_timeout_box.crashlog").unwrap();
    let crashlog = CrashLog::from_slice(&data).unwrap();

    let mut cm = CollateralManager::embedded_tree().unwrap();
    let root = crashlog.decode(&mut cm);

    let entry0 = root
        .get_by_path("processors.cpu0.die8.core0.sq.entry0")
        .unwrap();

    assert_eq!(
        entry0.kind,
        NodeType::Field {
            value: 0x47bcc051082
        }
    );

    let record_type_box = root
        .get_by_path("processors.cpu0.die8.box.hdr.version.record_type")
        .unwrap();
    assert_eq!(record_type_box.kind, NodeType::Field { value: 0x3d });

    let bank3 = root
        .get_by_path("processors.cpu0.die8.core0.thread.arch_state.mca.bank3.status")
        .unwrap();

    assert_eq!(
        bank3.kind,
        NodeType::Field {
            value: 0xbe000000e1840400
        }
    );
}
