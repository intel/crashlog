// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Berr;
use crate::CrashLog;

#[test]
fn from_slice() {
    let berr = Berr::from_bert_file(&std::fs::read("tests/samples/dummy.bert").unwrap());

    assert!(berr.is_some());
    let berr = berr.unwrap();

    assert_eq!(berr.entries.len(), 2);
}

#[test]
fn crashlog_bert() {
    let bert = std::fs::read("tests/samples/dummy.bert").unwrap();
    let crashlog = CrashLog::from_slice(&bert);

    assert!(crashlog.is_ok());
    let crashlog = crashlog.unwrap();

    assert_eq!(crashlog.regions.len(), 2);

    let bytes = crashlog.to_bert();
    let berr = Berr::from_bert_file(&bytes);
    assert!(berr.is_some());
    let berr = berr.unwrap();
    assert_eq!(berr.entries.len(), 2);
}
