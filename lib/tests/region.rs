// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;
use std::io::Read;

#[test]
fn from_slice() {
    let mut file = std::fs::File::open("tests/samples/three_strike_timeout.crashlog")
        .expect("Cannot open Crash Log file");

    let mut buffer = Vec::new();
    let _ = file.read_to_end(&mut buffer);
    let region = Region::from_slice(&buffer);

    assert!(region.is_ok());
}

#[test]
fn empty() {
    let region = Region::from_slice(&[]);
    assert!(region.is_err());
}
