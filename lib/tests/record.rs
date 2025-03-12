// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT
#![feature(assert_matches)]

use intel_crashlog::prelude::*;
use std::assert_matches::assert_matches;
use std::fs;
use std::path::Path;

const COLLATERAL_TREE_PATH: &str = "tests/collateral";

#[test]
fn basic_decode() {
    let record = Record {
        header: Header::default(),
        data: vec![
            0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D,
            0x8E, 0x8F,
        ],
    };

    let csv = "name;offset;size;description;bitfield
foo;0;128;;0
foo.bar.baz;4;64;;0
foo.bar;4;8;;0";

    let root = record.decode_with_csv(csv.as_bytes(), 0).unwrap();
    let section = root.get_by_path("foo").unwrap();
    assert_eq!(section.kind, NodeType::Record);
    let field = root.get_by_path("foo.bar").unwrap();
    assert_eq!(field.kind, NodeType::Field { value: 0x18 });
    let field = root.get_by_path("foo.bar.baz").unwrap();
    assert_eq!(
        field.kind,
        NodeType::Field {
            value: 0x8878685848382818
        }
    );
}

#[test]
fn relative_paths() {
    let record = Record {
        header: Header::default(),
        data: vec![
            0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D,
            0x8E, 0x8F,
        ],
    };

    let csv = "name;offset;size;description;bitfield
foo;0;128;;0
.aaa;8;8;;0
.bbb;16;8;;0
..ccc;24;8;;0
foo.ddd.eee;32;8;;0
...ddd;40;8;;0
..fff;48;8;;0";

    let root = record.decode_with_csv(csv.as_bytes(), 0).unwrap();
    let section = root.get_by_path("foo").unwrap();
    assert_eq!(section.kind, NodeType::Record);
    let field = root.get_by_path("foo.aaa").unwrap();
    assert_eq!(field.kind, NodeType::Field { value: 0x81 });
    let field = root.get_by_path("foo.aaa.bbb").unwrap();
    assert_eq!(field.kind, NodeType::Field { value: 0x82 });
    let field = root.get_by_path("foo.aaa.ccc").unwrap();
    assert_eq!(field.kind, NodeType::Field { value: 0x83 });
    let field = root.get_by_path("foo.ddd.eee").unwrap();
    assert_eq!(field.kind, NodeType::Field { value: 0x84 });
    let field = root.get_by_path("foo.ddd").unwrap();
    assert_eq!(field.kind, NodeType::Field { value: 0x85 });
    let field = root.get_by_path("foo.fff").unwrap();
    assert_eq!(field.kind, NodeType::Field { value: 0x86 });
}

#[test]
fn decode() {
    let mut cm = CollateralManager::file_system_tree(Path::new(COLLATERAL_TREE_PATH)).unwrap();

    let data = fs::read("tests/samples/dummy_mca_rev1.crashlog").unwrap();
    let header = Header::from_slice(&data).unwrap().unwrap();
    let record = Record { header, data };

    let root = record.decode(&mut cm).unwrap();
    let version = root.get_by_path("mca.hdr.version.revision").unwrap();
    assert_eq!(version.kind, NodeType::Field { value: 1 });
}

#[test]
fn header_type6_decode() {
    let mut cm = CollateralManager::file_system_tree(Path::new(COLLATERAL_TREE_PATH)).unwrap();

    let data = fs::read("tests/samples/dummy_mca_rev2.crashlog").unwrap();
    let header = Header::from_slice(&data).unwrap().unwrap();
    let record = Record { header, data };

    let root = record.decode(&mut cm).unwrap();
    let version = root
        .get_by_path("processors.cpu0.die1.mca.hdr.version.revision")
        .unwrap();

    assert_eq!(version.kind, NodeType::Field { value: 2 });
}

#[test]
fn header_checksum() {
    let data = fs::read("tests/samples/three_strike_timeout.crashlog").unwrap();
    let region = Region::from_slice(&data).unwrap();

    for record in region.records.iter() {
        assert!(record.checksum().unwrap())
    }
}

#[test]
fn invalid_decode_defs() {
    let record = Record {
        header: Header::default(),
        data: vec![0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87],
    };

    let csv = "name;offset;size;description;bitfield
foo;0;64;;0
foo.bar;=2+2;8;;0";
    assert_matches!(
        record.decode_with_csv(csv.as_bytes(), 0),
        Err(Error::ParseIntError(_))
    );

    let csv = "fullname;size;offset
aaa;4;8
";
    let root = record.decode_with_csv(csv.as_bytes(), 0).unwrap();
    assert_eq!(root, Node::root());

    let csv = "name;size;offset
foo.bar;8
";
    let root = record.decode_with_csv(csv.as_bytes(), 0).unwrap();
    let field = root.get_by_path("foo.bar").unwrap();
    assert_eq!(field.kind, NodeType::Field { value: 0x80 });

    let csv = "name;size;offset;size
....foo.bar;8;0
";
    let root = record.decode_with_csv(csv.as_bytes(), 0).unwrap();
    let field = root.get_by_path("foo.bar").unwrap();
    assert_eq!(field.kind, NodeType::Field { value: 0x80 });
}
