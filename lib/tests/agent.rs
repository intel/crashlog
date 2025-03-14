// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;

#[test]
fn invalid_recipe() {
    let data = std::fs::read("tests/samples/invalid_recipe.crashlog").unwrap();
    let crashlog = CrashLog::from_slice(&data).unwrap();
    let mut cm = CollateralManager::embedded_tree().unwrap();
    let nodes = crashlog.decode(&mut cm);

    let status = nodes.get_by_path("crashlog_agent.status").unwrap();
    assert_eq!(status.kind, NodeType::Field { value: 1 });
}
