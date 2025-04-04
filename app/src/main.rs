// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

mod decode;
mod extract;
mod unpack;

use clap::{Parser, Subcommand};
use env_logger::Env;
use intel_crashlog::prelude::*;
use log::LevelFilter;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Extract and decode Intel Crash Log records.")]
struct Cli {
    /// Path to the collateral tree. If not specified, the builtin collateral tree will be used.
    #[arg(short, long, value_name = "dir")]
    collateral_tree: Option<PathBuf>,

    /// Sets the verbosity of the logging messages
    /// -v: Warning, -vv: Info, -vvv: Debug, -vvvv: Trace
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbosity: u8,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Extract the Crash Log records from the platform
    Extract { output_path: Option<PathBuf> },
    /// Decode Crash Log records into JSON
    Decode { input_file: PathBuf },
    /// List the Crash Log records stored in the input file
    Info { input_files: Vec<PathBuf> },
    /// Unpack the Crash Log records stored in the input file
    Unpack { input_files: Vec<PathBuf> },
}

impl Command {
    fn run<T: CollateralTree>(&self, mut cm: CollateralManager<T>) -> Result<(), Error> {
        match self {
            Command::Extract { output_path } => extract::extract(output_path.as_deref()),
            Command::Decode { input_file } => {
                decode::decode(&mut cm, input_file, std::io::stdout().lock())?
            }
            Command::Info { input_files } => {
                for input_file in input_files {
                    if input_files.len() > 1 {
                        println!("\n{}:\n", input_file.display());
                    }
                    if let Err(err) = decode::info(&cm, input_file) {
                        log::error!("Error: {err}")
                    }
                }
            }
            Command::Unpack { input_files } => {
                for input_file in input_files {
                    if let Err(err) = unpack::unpack(input_file) {
                        log::error!("Error: {err}")
                    }
                }
            }
        }
        Ok(())
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        if let Some(collateral_tree) = cli.collateral_tree {
            command.run(CollateralManager::file_system_tree(&collateral_tree)?)?
        } else {
            command.run(CollateralManager::embedded_tree()?)?
        }
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

    if let Err(err) = run() {
        log::error!("Fatal Error: {err}");
    }
}
