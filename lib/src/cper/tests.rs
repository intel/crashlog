// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Cper;
use super::section::CperSectionBody;
use crate::CrashLog;

pub const FW_ERROR_RECORD_GUID: uguid::Guid = uguid::guid!("81212a96-09ed-4996-9471-8d729c8e69ed");

#[test]
fn from_slice() {
    let cper = Cper::from_slice(&std::fs::read("tests/samples/cper.whea").unwrap()).unwrap();

    assert_eq!(cper.record_header.section_count, 5);

    assert_eq!(cper.sections.len(), 5);

    for section in cper.sections.iter() {
        assert_eq!(section.descriptor.section_type, FW_ERROR_RECORD_GUID);
    }
}

#[test]
fn cl_from_cper() {
    let cper = Cper::from_slice(&std::fs::read("tests/samples/cper.whea").unwrap()).unwrap();
    let crashlog = CrashLog::from_cper(cper);
    assert!(crashlog.is_ok());
    let crashlog = crashlog.unwrap();

    assert_eq!(crashlog.regions.len(), 3);

    let mut records = Vec::new();

    for region in crashlog.regions.iter() {
        for record in region.records.iter() {
            records.push(record);
        }
    }

    assert_eq!(crashlog.metadata.extra_cper_sections.len(), 2);
    assert_eq!(records.len(), 3);

    let cper_bytes = crashlog.to_bytes();
    let cper = Cper::from_slice(&cper_bytes).unwrap();
    assert_eq!(cper.record_header.section_count, 5);
}

#[test]
fn cl_to_cper() {
    let data = std::fs::read("tests/samples/dummy_crashlog_agent_rev1.crashlog").unwrap();
    let crashlog = CrashLog::from_slice(&data).unwrap();
    let cper_bytes = crashlog.to_bytes();
    let cper = Cper::from_slice(&cper_bytes).unwrap();

    assert_eq!(cper.record_header.section_count, 1);

    let section = &cper.sections[0];
    let CperSectionBody::FirmwareErrorRecord(ref fer) = section.body else {
        let guid = section.descriptor.section_type;
        panic!("Section is not a FirmwareErrorRecord: {guid}");
    };
    assert_eq!(fer.payload, data);
}
