// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

use super::Analyzer;
use super::tag::Tag;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Structured report containing analysis results and diagnostic findings.
///
/// The report aggregates all triage tags identified during crash log analysis.
/// Tags are ordered by priority, with higher-priority findings appearing first.
pub struct AnalysisReport {
    /// Triage tags ordered by priority (highest priority first).
    pub tags: Vec<Tag>,
}

impl Analyzer<'_> {
    pub(super) fn build_report(mut self) -> AnalysisReport {
        let mut tags = Vec::new();

        while let Some(tag) = self.tags.pop() {
            tags.push(tag.tag);
        }

        AnalysisReport { tags }
    }
}
