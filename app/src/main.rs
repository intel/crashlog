// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

mod analysis;
mod control;
mod decode;
mod extract;
mod info;
mod list;
mod table;
mod unpack;

use clap::{Parser, Subcommand, ValueEnum};
use env_logger::Env;
use intel_crashlog::prelude::*;
use log::LevelFilter;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about = "Extract and decode Intel Crash Log records.")]
struct Cli {
    /// Path to the collateral tree. If not specified, the builtin collateral tree will be used.
    #[arg(short, long, value_name = "dir")]
    collateral_tree: Option<PathBuf>,

    /// Sets the verbosity of the logging messages
    /// -v: Warning, -vv: Info, -vvv: Debug, -vvvv: Trace
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbosity: u8,

    #[command(subcommand)]
    command: Command,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, ValueEnum)]
pub(crate) enum InfoFormat {
    #[default]
    Compact,
    Markdown,
}

#[derive(Subcommand)]
enum Command {
    /// Enable Crash Log collection in the platform
    Enable {
        #[arg(short, long, value_delimiter = ',')]
        sources: Vec<CrashLogSource>,
    },
    /// Extract the Crash Log records from the platform
    Extract {
        output_path: Option<PathBuf>,
        #[arg(short, long, value_delimiter = ',')]
        sources: Vec<CrashLogSource>,
    },
    /// Decode Crash Log records into JSON
    Decode { input_file: PathBuf },
    /// Disable Crash Log collection in the platform
    Disable {
        #[arg(short, long, value_delimiter = ',')]
        sources: Vec<CrashLogSource>,
    },
    /// List the Crash Log records stored in the input file
    Info {
        #[arg(short, long, value_enum, default_value_t = InfoFormat::default())]
        format: InfoFormat,
        input_files: Vec<PathBuf>,
    },
    /// List the Crash Log sources that are available in the platform with their capabilities
    List,
    /// Trigger an on-demand Crash Log collection in the platform
    Trigger {
        #[arg(short, long, value_delimiter = ',')]
        sources: Vec<CrashLogSource>,
    },
    /// Unpack the Crash Log records stored in the input file
    Unpack { input_files: Vec<PathBuf> },
    /// Triage the Crash Log records stored in the input files
    Triage { input_files: Vec<PathBuf> },
}

impl Command {
    fn run<T: CollateralTree>(&self, mut cm: CollateralManager<T>) -> Result<(), Error> {
        match self {
            Command::Enable { sources } => control::enable(sources.clone())?,
            Command::Extract {
                output_path,
                sources,
            } => extract::extract(output_path.as_deref(), sources.clone()),
            Command::Decode { input_file } => {
                decode::decode(&mut cm, input_file, std::io::stdout().lock())?
            }
            Command::Disable { sources } => control::disable(sources.clone())?,
            Command::Info {
                input_files,
                format,
            } => info::info(&cm, input_files, *format),
            Command::List => list::list(),
            Command::Trigger { sources } => control::trigger(sources.clone())?,
            Command::Unpack { input_files } => {
                for input_file in input_files {
                    if let Err(err) = unpack::unpack(input_file) {
                        log::error!("Error: {err}")
                    }
                }
            }
            Command::Triage { input_files } => analysis::triage_files(&mut cm, input_files),
        }
        Ok(())
    }
}

fn run(cli: Cli) -> Result<(), Error> {
    if let Some(collateral_tree) = cli.collateral_tree {
        cli.command
            .run(CollateralManager::file_system_tree(&collateral_tree)?)?
    } else {
        cli.command.run(CollateralManager::embedded_tree()?)?
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let log_level = match cli.verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::Builder::from_env(Env::default().default_filter_or(log_level.to_string())).init();

    if let Err(err) = run(cli) {
        log::error!("Fatal Error: {err}");
        std::process::exit(1);
    }
}
