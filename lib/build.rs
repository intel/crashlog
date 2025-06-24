// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(feature = "ffi")]
extern crate cbindgen;

use std::env;

#[cfg(feature = "embedded_collateral_tree")]
use std::{
    collections::BTreeSet,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

const COLLATERAL_PATH_VAR: &str = "CRASHLOG_COLLATERAL_TREE";
const PRODUCTS_VAR: &str = "CRASHLOG_PRODUCTS";

#[cfg(feature = "ffi")]
fn generate_headers() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    cargo_emit::rerun_if_changed!("src/ffi.rs");

    for (language, header) in [
        (cbindgen::Language::C, "target/include/intel_crashlog.h"),
        (cbindgen::Language::Cxx, "target/include/intel_crashlog.hpp"),
    ] {
        cbindgen::Builder::new()
            .with_crate(crate_dir.clone())
            .with_config(cbindgen::Config {
                language,
                pragma_once: language == cbindgen::Language::Cxx,
                include_guard: if language == cbindgen::Language::C {
                    Some("CRASHLOG_H".into())
                } else {
                    None
                },
                package_version: true,
                ..cbindgen::Config::default()
            })
            .rename_item("Node", "CrashLogNode")
            .generate()
            .map_or_else(
                |error| match error {
                    cbindgen::Error::ParseSyntaxError { .. } => {}
                    e => panic!("{e:?}"),
                },
                |bindings| {
                    bindings.write_to_file(header);
                },
            );
    }
}

#[cfg(feature = "embedded_collateral_tree")]
fn embed_collateral_tree() {
    cargo_emit::rerun_if_env_changed!(COLLATERAL_PATH_VAR);
    cargo_emit::rerun_if_env_changed!(PRODUCTS_VAR);

    let collateral_tree =
        env::var(COLLATERAL_PATH_VAR).unwrap_or_else(|_| "collateral".to_string());

    cargo_emit::rerun_if_changed!(collateral_tree);

    let requested_products: BTreeSet<String> = env::var(PRODUCTS_VAR)
        .unwrap_or_else(|_| String::new())
        .split(",")
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_owned())
        .collect();

    let mut included_products = BTreeSet::new();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("embedded_collateral_tree.rs");
    let mut file = File::create(dest_path).unwrap();

    let tree_path = std::path::absolute(Path::new(&collateral_tree)).unwrap();
    file.write_all("{\n".as_ref()).unwrap();
    for (product, variant, stepping, security, fullpath) in visit_collateral_tree(&tree_path) {
        // Check if product must be included in the collateral tree
        if !requested_products.is_empty() && !requested_products.contains(&product) {
            continue;
        }

        if product != "all" {
            included_products.insert(product.clone());
        }

        let path = fullpath
            .strip_prefix(
                tree_path
                    .join(&product)
                    .join(&variant)
                    .join(&stepping)
                    .join(&security)
                    .join("crashlog"),
            )
            .unwrap()
            .iter()
            .filter_map(|component| component.to_str())
            .collect::<Vec<&str>>()
            .join("/");

        file.write_all(
            format!(
                "    tree.insert_item(
        {product:?},
        {variant:?},
        {stepping:?},
        {security:?},
        {path:?},
        include_bytes!({fullpath:?})
    );\n"
            )
            .as_ref(),
        )
        .unwrap();
    }
    file.write_all("}\n".as_ref()).unwrap();

    for requested_product in requested_products {
        if !included_products.contains(&requested_product) {
            cargo_emit::warning!("Requested unknown Crash Log product: {}", requested_product);
        }
    }

    if !included_products.is_empty() {
        cargo_emit::warning!(
            "Embedding Crash Log support for the following product(s): {}",
            included_products
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

#[cfg(feature = "embedded_collateral_tree")]
fn visit_collateral_tree(root: &Path) -> Vec<(String, String, String, String, PathBuf)> {
    let mut items = Vec::new();

    for product in std::fs::read_dir(root).unwrap() {
        let product = product.unwrap();
        for variant in std::fs::read_dir(product.path()).unwrap() {
            let variant = variant.unwrap();
            for stepping in std::fs::read_dir(variant.path()).unwrap() {
                let stepping = stepping.unwrap();
                for security in std::fs::read_dir(stepping.path()).unwrap() {
                    let security = security.unwrap();
                    let mut path = security.path();
                    path.push("crashlog");

                    for path in visit_dirs(&path) {
                        items.push((
                            product.file_name().into_string().unwrap(),
                            variant.file_name().into_string().unwrap(),
                            stepping.file_name().into_string().unwrap(),
                            security.file_name().into_string().unwrap(),
                            path,
                        ));
                    }
                }
            }
        }
    }

    items
}

#[cfg(feature = "embedded_collateral_tree")]
fn visit_dirs(path: &Path) -> Vec<PathBuf> {
    let mut paths = vec![];

    if !path.is_dir() {
        paths.push(path.to_path_buf());
        return paths;
    }

    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        paths.append(&mut visit_dirs(&path));
    }

    paths
}

fn main() {
    #[cfg(feature = "ffi")]
    generate_headers();

    #[cfg(feature = "embedded_collateral_tree")]
    embed_collateral_tree();
}
