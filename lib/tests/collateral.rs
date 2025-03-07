// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::collateral::PVSS;
use intel_crashlog::prelude::*;
use std::path::Path;

const COLLATERAL_TREE_PATH: &str = "tests/collateral";

#[test]
fn get_with_pvss() {
    let mut cm = CollateralManager::file_system_tree(Path::new(COLLATERAL_TREE_PATH)).unwrap();
    let pvss = PVSS {
        product: "XYZ".into(),
        security: "green".into(),
        ..PVSS::default()
    };

    assert!(cm.get_item_with_pvss(pvss, "target_info.json").is_ok());
}

#[test]
fn target_info() {
    let cm = CollateralManager::file_system_tree(Path::new(COLLATERAL_TREE_PATH)).unwrap();
    assert_eq!(cm.target_info.get(&0x07A).unwrap().product, "XYZ");
}

#[test]
fn get_with_pvss_embedded() {
    let mut cm = CollateralManager::embedded_tree().unwrap();
    let pvss = PVSS {
        product: "LNC".into(),
        security: "all".into(),
        ..PVSS::default()
    };

    assert!(cm.get_item_with_pvss(pvss, "target_info.json").is_ok());
}

#[test]
fn target_info_embedded() {
    let cm = CollateralManager::embedded_tree().unwrap();
    assert_eq!(cm.target_info.get(&0x052).unwrap().product, "LNC");
}

#[test]
fn out_of_tree() {
    let mut cm = CollateralManager::file_system_tree(Path::new(COLLATERAL_TREE_PATH)).unwrap();
    let pvss = PVSS {
        product: "XYZ".into(),
        security: "green".into(),
        ..PVSS::default()
    };

    assert!(
        cm.get_item_with_pvss(pvss, "../../../../../../README.md")
            .is_err()
    );
}
