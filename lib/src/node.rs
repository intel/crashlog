// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

//! A tree-like data structure containing the decoded Crash Log registers.

#[cfg(feature = "std")]
use std::collections::{BTreeMap, btree_map};

#[cfg(not(feature = "std"))]
use alloc::{
    collections::{BTreeMap, btree_map},
    format,
    string::String,
};

#[cfg(feature = "serialize")]
use serde::ser::{Serialize, SerializeMap, Serializer};

/// Crash Log register tree node type
#[derive(Debug, Default, PartialEq, Eq)]
pub enum NodeType {
    /// Root of the register tree
    #[default]
    Root,
    /// Component of the register tree
    Section,
    /// Root of the record section in the register tree
    Record,
    /// Crash Log field
    Field { value: u64 },
}

/// Node of the Crash Log register tree
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Node {
    /// Name of the node
    pub name: String,
    /// Description of the node
    pub description: String,
    /// Type of the node
    pub kind: NodeType,
    children: BTreeMap<String, Node>,
}

impl Node {
    /// Returns a new root node.
    ///
    /// # Examples
    ///
    /// ```
    /// use intel_crashlog::prelude::*;
    ///
    /// let root = Node::root();
    /// assert_eq!(root.kind, NodeType::Root);
    /// ```
    pub fn root() -> Node {
        Node::default()
    }

    /// Returns a new section node.
    ///
    /// # Examples
    ///
    /// ```
    /// use intel_crashlog::prelude::*;
    ///
    /// let node = Node::section("foo");
    /// assert_eq!(node.kind, NodeType::Section);
    /// assert_eq!(node.name, "foo");
    /// ```
    pub fn section(name: &str) -> Node {
        Node {
            name: name.to_lowercase(),
            kind: NodeType::Section,
            ..Node::default()
        }
    }

    /// Returns a new record node.
    ///
    /// # Examples
    ///
    /// ```
    /// use intel_crashlog::prelude::*;
    ///
    /// let node = Node::record("foo");
    /// assert_eq!(node.kind, NodeType::Record);
    /// assert_eq!(node.name, "foo");
    /// ```
    pub fn record(name: &str) -> Node {
        Node {
            name: name.to_lowercase(),
            kind: NodeType::Record,
            ..Node::default()
        }
    }

    /// Returns a new field node.
    ///
    /// # Examples
    ///
    /// ```
    /// use intel_crashlog::prelude::*;
    ///
    /// let node = Node::field("foo", 42);
    /// assert_eq!(node.kind, NodeType::Field { value: 42 });
    /// assert_eq!(node.name, "foo");
    /// ```
    pub fn field(name: &str, value: u64) -> Node {
        Node {
            name: name.to_lowercase(),
            kind: NodeType::Field { value },
            ..Node::default()
        }
    }

    /// Returns a reference to a child of the node. If the child does not exist, [`None`] is
    /// returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use intel_crashlog::prelude::*;
    ///
    /// let mut node = Node::section("foo");
    /// node.add(Node::section("bar"));
    /// assert_eq!(node.get("bar"), Some(&Node::section("bar")));
    /// assert_eq!(node.get("baz"), None);
    /// ```
    pub fn get(&self, name: &str) -> Option<&Node> {
        self.children.get(name)
    }

    /// Returns a mutable reference to a child of the node. If the child does not exist, [`None`]
    /// is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use intel_crashlog::prelude::*;
    ///
    /// let mut node = Node::section("foo");
    /// node.add(Node::section("bar"));
    /// if let Some(child) = node.get_mut("bar") {
    ///     // Change the node type
    ///     child.kind = NodeType::Field { value: 42 };
    /// }
    ///
    /// assert_eq!(node.get("bar"), Some(&Node::field("bar", 42)));
    /// ```
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Node> {
        self.children.get_mut(name)
    }

    /// Returns a reference to the node in the tree located at the specified `path`. The `path`
    /// consists in a `&str` representing the names of the parent nodes separated by `.` (For
    /// example: `foo.bar.baz`).
    ///
    /// # Examples
    ///
    /// ```
    /// use intel_crashlog::prelude::*;
    ///
    /// let mut foo = Node::section("foo");
    /// foo.add(Node::section("bar"));
    /// let mut root = Node::root();
    /// root.add(foo);
    ///
    /// assert_eq!(root.get_by_path("foo.bar"), Some(&Node::section("bar")));
    /// assert_eq!(root.get_by_path("foo.baz"), None);
    /// ```
    pub fn get_by_path(&self, path: &str) -> Option<&Node> {
        let mut ptr = self;
        for name in path.split('.') {
            ptr = ptr.get(name)?
        }
        Some(ptr)
    }

    fn merge_instance(&mut self, mut other: Node) {
        let mut instance = 0;
        let name = other.name.clone();
        while self.children.contains_key(&other.name) {
            other.name = format!("{}{}", name, instance);
            instance += 1
        }
        self.add(other)
    }

    pub fn merge(&mut self, other: Node) {
        for (_, child) in other.children {
            if let Some(self_child) = self.children.get_mut(&child.name) {
                if let NodeType::Record | NodeType::Field { .. } = self_child.kind {
                    self.merge_instance(child)
                } else {
                    self_child.merge(child)
                }
            } else {
                self.add(child)
            }
        }
    }

    pub fn add(&mut self, node: Node) {
        let _ = self.children.insert(node.name.clone(), node);
    }

    pub fn create_hierarchy(&mut self, path: &str) -> &mut Node {
        let mut ptr = self;
        for name in path.split('.') {
            if ptr.get(name).is_none() {
                ptr.add(Node::section(name));
            }
            ptr = ptr
                .get_mut(name)
                .expect("Node should be present in the hierarchy")
        }
        ptr
    }

    pub fn create_record_hierarchy(&mut self, path: &str) -> &mut Node {
        match path.split_once(".") {
            Some((record, hierarchy)) => {
                if self.get(record).is_none() {
                    self.add(Node::record(record));
                }
                let record_node = self.get_mut(record).expect("Record node should be present");
                record_node.create_hierarchy(hierarchy)
            }
            None => self.create_hierarchy(path),
        }
    }

    /// Returns an iterator over the node's children. The children nodes are sorted alphabetically.
    ///
    /// # Examples
    ///
    /// ```
    /// use intel_crashlog::prelude::*;
    ///
    /// let mut root = Node::root();
    /// root.add(Node::section("foo"));
    /// root.add(Node::section("bar"));
    ///
    /// let mut children = root.children();
    /// assert_eq!(children.next(), Some(&Node::section("bar")));
    /// assert_eq!(children.next(), Some(&Node::section("foo")));
    /// assert_eq!(children.next(), None);
    /// ```
    pub fn children(&self) -> NodeChildren {
        NodeChildren {
            iter: self.children.values(),
        }
    }
}

#[cfg(feature = "serialize")]
impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.kind {
            NodeType::Field { value } => {
                if self.children.is_empty() {
                    serializer.serialize_str(&format!("0x{:x}", value))
                } else {
                    let mut map = serializer.serialize_map(Some(self.children.len() + 1))?;
                    map.serialize_entry("_value", &format!("0x{:x}", value))?;
                    for (k, v) in self.children.iter() {
                        map.serialize_entry(k, v)?;
                    }
                    map.end()
                }
            }
            NodeType::Root => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("crashlog_data", &self.children)?;
                map.end()
            }
            _ => {
                let mut map = serializer.serialize_map(Some(self.children.len()))?;
                for (k, v) in self.children.iter() {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

/// An iterator over the children of a node.
///
/// This struct is created by the [`children`] method on a [`Node`].
///
/// [`children`]: Node::children
pub struct NodeChildren<'a> {
    iter: btree_map::Values<'a, String, Node>,
}

impl<'a> Iterator for NodeChildren<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
