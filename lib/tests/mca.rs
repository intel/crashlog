// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;

#[test]
fn lnl() {
    let data = std::fs::read("tests/samples/three_strike_timeout.crashlog").unwrap();
    let crashlog = CrashLog::from_slice(&data).unwrap();
    let mut cm = CollateralManager::embedded_tree().unwrap();
    let nodes = crashlog.decode(&mut cm);

    let status = nodes.get_by_path("mca.core1.thread0.bank0.ctl").unwrap();
    assert_eq!(status.kind, NodeType::Field { value: 0x1fff });
    let status = nodes.get_by_path("mca.core7.bank0.ctl").unwrap();
    assert_eq!(status.kind, NodeType::Field { value: 0x1de });
}
