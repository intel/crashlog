// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;
use std::fs;

#[test]
fn three_strike_timeout() {
    let data = fs::read("tests/samples/three_strike_timeout_with_xq.crashlog").unwrap();

    let crashlog = CrashLog::from_slice(&data).unwrap();
    let mut cm = CollateralManager::embedded_tree().unwrap();
    let root = crashlog.decode(&mut cm);

    let tags = Analyzer::default()
        .with_input(&root)
        .analyze()
        .tags
        .into_iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        tags[0],
        "CORE_TIMEOUT.SINGLE_STUCK_TRANSACTION.13014002340H"
    );
    assert_eq!(tags[1], "MCA.BANK3.INTERNAL_TIMER_ERROR.MSCOD_E184H");
}
