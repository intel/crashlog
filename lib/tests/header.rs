// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;
use std::fs;
use std::path::Path;

const COLLATERAL_TREE_PATH: &str = "tests/collateral";

#[test]
fn decode_header_size() {
    // 0x3e07a108
    let data = vec![0x08, 0xa1, 0x07, 0x3e, 0x2, 0x0, 0x0, 0x0];
    let header = Header::from_slice(&data).unwrap().unwrap();

    assert_eq!(header.size.record_size, 2);

    let record_size_bytes = header.record_size();
    assert_eq!(record_size_bytes, 8);

    let revision = header.revision();
    assert_eq!(revision, 8);

    let product_id = header.product_id();
    assert_eq!(product_id, 0x7A);

    let record_type = header.record_type().unwrap();
    assert_eq!(record_type, "MCA")
}

#[test]
fn decode_header_product() {
    let cm = CollateralManager::file_system_tree(Path::new(COLLATERAL_TREE_PATH)).unwrap();
    let data = vec![0x08, 0xa1, 0x07, 0x3e, 0x2, 0x0, 0x0, 0x0];
    let header = Header::from_slice(&data).unwrap().unwrap();

    let product = header.product(&cm).unwrap();
    assert_eq!(product, "XYZ");

    let variant = header.variant(&cm);
    assert_eq!(variant, Some("all"));

    assert_eq!(
        header.to_string(),
        "MCA - (product_id=0x7a, record_type=0x3e, revision=0x8, ..)"
    )
}

#[test]
fn decode_header_to_node() {
    let data = vec![0x08, 0xa1, 0x07, 0x3e, 0x2, 0x0, 0x0, 0x0];
    let header = Header::from_slice(&data).unwrap().unwrap();

    let node = Node::from(&header);
    assert_eq!(
        node.get_by_path("version.product_id").unwrap().kind,
        NodeType::Field { value: 0x7a }
    );
}

#[test]
fn decode_die_id_header() {
    let cm = CollateralManager::file_system_tree(Path::new(COLLATERAL_TREE_PATH)).unwrap();

    let data = fs::read("tests/samples/dummy_mca_rev2.crashlog").unwrap();
    let header = Header::from_slice(&data).unwrap().unwrap();

    let die_id = header.die_id().unwrap();
    assert_eq!(die_id, 1);

    let die = header.die(&cm).unwrap();
    assert_eq!(die, "io1");
}
