// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;

#[test]
fn lnc_three_strike_timeout() {
    let data = std::fs::read("tests/samples/three_strike_timeout.crashlog").unwrap();
    let crashlog = CrashLog::from_slice(&data).unwrap();
    let mut cm = CollateralManager::embedded_tree().unwrap();
    let nodes = crashlog.decode(&mut cm);

    let status = nodes
        .get_by_path("core0.thread.arch_state.mca.bank3.status")
        .unwrap();
    assert_eq!(
        status.kind,
        NodeType::Field {
            value: 0xbe000000e1840400
        }
    );

    let lip = nodes.get_by_path("core0.thread.arch_state.lip").unwrap();
    assert_eq!(
        lip.kind,
        NodeType::Field {
            value: 0xfffff80577036530
        }
    );

    let entry = nodes.get_by_path("core0.sq.entry0").unwrap();
    assert_eq!(
        entry.kind,
        NodeType::Field {
            value: 0x47bcc051082
        }
    );
}
