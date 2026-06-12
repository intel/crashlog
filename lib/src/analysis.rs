// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

//! Analysis and interpretation of decoded Crash Log records.
//!
//! This module provides functionality to analyze crash log data structures,
//! identify root causes of system failures, and extract diagnostic information.
//! The analyzer examines various record types including reset sources, MCA
//! (Machine Check Architecture) events, core exceptions, and PMC (Power Management
//! Controller) data to generate comprehensive analysis reports.
//!
//! Current capabilities include triage analysis for prioritizing crash log findings,
//! with additional analysis features planned for future releases.
//!
//! # Analysis Process
//!
//! 1. Queue one or more decoded crash log nodes using [`Analyzer::with_input`]
//! 2. Call [`Analyzer::analyze`] to perform analysis across all queued nodes
//! 3. Receive an [`AnalysisReport`] containing diagnostic findings

mod core;
mod mca;
mod reason;
mod report;
mod reset;
mod tag;
mod xq;

use crate::node::{Node, NodeType};
#[cfg(not(feature = "std"))]
use alloc::collections::{BinaryHeap, VecDeque};
pub use report::AnalysisReport;
#[cfg(feature = "std")]
use std::collections::{BinaryHeap, VecDeque};
pub use tag::Tag;
use tag::TriageTag;

/// Analyzes decoded Crash Log records to extract diagnostic information.
///
/// The analyzer examines crash log data structures to identify root causes,
/// extract diagnostic information, and generate structured reports. Results
/// include prioritized findings and interpretation of crash log events.
#[derive(Default)]
pub struct Analyzer<'a> {
    inputs: VecDeque<&'a Node>,
    tags: BinaryHeap<TriageTag>,
}

impl<'a> Analyzer<'a> {
    fn analyze_record(&mut self, node: &Node) {
        match node.name.as_str().trim_end_matches(char::is_numeric) {
            "pmc_rst" => self.analyze_pmc_rst(node),
            "punit" | "pmc" => self.analyze_reason(node),
            "pcore" | "ecore" => self.analyze_core(node),
            "mca" => self.analyze_mca(node),
            _ => (),
        }
    }

    fn analyze_records(&mut self, node: &Node) {
        if let NodeType::Record = node.kind {
            self.analyze_record(node)
        } else {
            for child in node.children() {
                self.analyze_records(child);
            }
        }
    }

    /// Adds a crash log node to the analysis queue.
    ///
    /// Multiple nodes can be chained for batch analysis. The analyzer will
    /// process all queued nodes when [`analyze`](Self::analyze) is called.
    pub fn with_input(mut self, node: &'a Node) -> Self {
        self.inputs.push_back(node);
        self
    }

    /// Consumes the analyzer and performs analysis on all queued nodes.
    ///
    /// Traverses the crash log data structures, identifies issues, and generates
    /// a report containing diagnostic findings and interpretations.
    ///
    /// # Returns
    ///
    /// An [`AnalysisReport`] containing analysis results.
    pub fn analyze(mut self) -> AnalysisReport {
        while let Some(input) = self.inputs.pop_front() {
            self.analyze_records(input);
        }

        self.build_report()
    }
}
