// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use super::table::{Row, Table};
use intel_crashlog::source::CrashLogSource;

pub fn list() {
    let sources = CrashLogSource::discover_all_sources();

    if sources.is_empty() {
        println!("No available Crash Log sources found.");
        return;
    }

    let mut table = Table::from(["Source", "Description", "Capabilities"]);

    for source in sources {
        let src = format!("{}", source);
        let description = source.description();
        let capabilities = source
            .capabilities()
            .iter()
            .map(|capability| capability.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        table.append_row(Row::from([src, description, capabilities]));
    }

    table.render();
}
