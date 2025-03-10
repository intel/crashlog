// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use intel_crashlog::prelude::*;
use serde_json::json;

#[test]
fn hierarchy() {
    let mut section = Node::section("foo");
    let field = Node::field("bar", 42);
    section.add(field);

    assert!(section.get("bar").is_some());
    assert!(section.get("baz").is_none());
}

#[test]
fn path() {
    let mut root = Node::root();
    let mut section = Node::section("foo");
    let mut field = Node::field("bar", 42);
    let bitfield = Node::field("baz", 1);
    field.add(bitfield);
    section.add(field);
    root.add(section);

    assert!(root.get_by_path("foo.bar.baz").is_some());
    assert!(root.get_by_path("foo.does_no_exist").is_none());
}

#[test]
fn create_hierarchy() {
    let mut root = Node::root();
    let path0 = "foo.bar.baz";
    let path1 = "foo.baz.bar";
    root.create_hierarchy(path0);
    root.create_hierarchy(path1);
    assert!(root.get_by_path(path0).is_some());
    assert!(root.get_by_path(path1).is_some());
    assert!(root.get_by_path("foo.does_no_exist").is_none());
}

#[test]
fn export_json() {
    let mut root = Node::root();
    let field = root.create_hierarchy("foo.bar");
    field.kind = NodeType::Field { value: 42 };
    let field = root.create_hierarchy("foo.bar.baz");
    field.kind = NodeType::Field { value: 1 };
    let json = serde_json::to_value(&root).unwrap();
    assert_eq!(
        json,
        json!({
            "crashlog_data": {
                "foo": {
                    "bar":{
                        "_value": "0x2a",
                        "baz": "0x1"
                    }
                }
            }
        })
    );
}

#[test]
fn merge() {
    let mut root0 = Node::root();
    root0.add(Node::field("foo", 1));
    let mut root1 = Node::root();
    root1.add(Node::field("foo", 2));
    let mut root2 = Node::root();
    root2.add(Node::field("foo", 3));

    root0.merge(root1);
    root0.merge(root2);

    assert_eq!(root0.get("foo").unwrap().kind, NodeType::Field { value: 1 });
    assert_eq!(
        root0.get("foo0").unwrap().kind,
        NodeType::Field { value: 2 }
    );
    assert_eq!(
        root0.get("foo1").unwrap().kind,
        NodeType::Field { value: 3 }
    );
}

#[test]
fn merge_record() {
    let mut root0 = Node::root();
    root0.add(Node::record("foo"));
    root0.create_hierarchy("foo.bar.reg0");
    root0.create_hierarchy("some.bar.reg1");

    let mut root1 = Node::root();
    root1.add(Node::record("foo"));
    root1.create_hierarchy("foo.bar.reg1");

    root0.merge(root1);

    assert_eq!(root0.get("foo").unwrap().kind, NodeType::Record);
    assert!(root0.get("foo0").is_some());
    assert_eq!(root0.get("foo0").unwrap().kind, NodeType::Record);
    assert_eq!(root0.get("some").unwrap().kind, NodeType::Section);
}

#[test]
fn merge_section() {
    let mut root0 = Node::root();
    root0.create_hierarchy("foo.bar.baz");

    let mut root1 = Node::root();
    root1.create_hierarchy("foo.bar.baz");

    assert_eq!(root0.get("foo").unwrap().kind, NodeType::Section);
    assert!(root0.get("foo0").is_none());
}
