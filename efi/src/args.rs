// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

use alloc::fmt;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use uefi::CString16;
use uefi::fs::PathBuf;
use uefi::prelude::*;
use uefi::println;
use uefi::proto::shell_params::ShellParameters;

pub enum ArgsError {
    InvalidArgument(String),
    MissingArgument(&'static str),
}

impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArgsError::MissingArgument(arg) => write!(f, "Missing argument: {arg}"),
            ArgsError::InvalidArgument(arg) => write!(f, "Invalid argument: {arg}"),
        }
    }
}

pub enum Command {
    Extract { output_path: Option<PathBuf> },
    Info { input_paths: Vec<PathBuf> },
    Decode { input_path: PathBuf },
}

#[derive(Default)]
pub struct Args {
    pub app_name: CString16,
    pub help: bool,
    pub wait: bool,
    pub command: Option<Command>,
}

impl Args {
    pub fn parse() -> Result<Self, ArgsError> {
        let image = uefi::boot::image_handle();
        let shell_param = match uefi::boot::open_protocol_exclusive::<ShellParameters>(image) {
            Ok(shell_param) => shell_param,
            Err(err) => {
                log::debug!("Failed to open ShellParameters protocol: {err}");
                log::warn!("The command line arguments could not be read.");
                log::warn!("Using the following options instead: -w extract");
                return Ok(Args {
                    command: Some(Command::Extract { output_path: None }),
                    wait: true,
                    ..Args::default()
                });
            }
        };

        let mut tokens = shell_param.args();

        let mut args = Args {
            app_name: tokens
                .next()
                .map(CString16::from)
                .unwrap_or_else(|| CString16::from(cstr16!("iclg.efi"))),
            ..Default::default()
        };

        while let Some(token) = tokens.next() {
            let token = token.to_string();
            match token.as_str() {
                "-h" | "--help" => args.help = true,
                "-w" | "--wait" => args.wait = true,
                "extract" => {
                    args.command = Some(Command::Extract {
                        output_path: tokens.next().map(PathBuf::from),
                    });

                    if let Some(token) = tokens.next() {
                        return Err(ArgsError::InvalidArgument(token.to_string()));
                    }
                }
                "info" => {
                    args.command = Some(Command::Info {
                        input_paths: tokens.map(PathBuf::from).collect(),
                    });
                    break;
                }
                "decode" => {
                    args.command = Some(Command::Decode {
                        input_path: tokens
                            .next()
                            .map(PathBuf::from)
                            .ok_or(ArgsError::MissingArgument("FILENAME"))?,
                    });

                    if let Some(token) = tokens.next() {
                        return Err(ArgsError::InvalidArgument(token.to_string()));
                    }
                }
                _ => return Err(ArgsError::InvalidArgument(token)),
            }
        }

        Ok(args)
    }

    pub fn show_help(&self) {
        println!(
            "Lightweight Crash Log Framework - UEFI Application, version: {}",
            env!("CARGO_PKG_VERSION")
        );

        match self.command {
            None => {
                println!("Usage: {} [OPTIONS] [COMMAND] [ARGUMENTS]\n", self.app_name);
                println!("Commands:");
                println!("    extract");
                println!("    info");
                println!("    decode");
            }
            Some(Command::Extract { .. }) => {
                println!("Usage: {} [OPTIONS] extract [OUTPUT_PATH]\n", self.app_name);
                println!("Examples:");
                println!("    > {} extract", self.app_name);
                println!("    > {} extract sample.crashlog", self.app_name);
            }
            Some(Command::Info { .. }) => {
                println!("Usage: {} [OPTIONS] info [INPUT_PATH] ...\n", self.app_name);
                println!("Examples:");
                println!("    > {} info sample.crashlog", self.app_name);
                println!(
                    "    > {} info sample0.crashlog sample1.crashlog",
                    self.app_name
                );
            }
            Some(Command::Decode { .. }) => {
                println!("Usage: {} [OPTIONS] decode [INPUT_PATH]\n", self.app_name);
                println!("Examples:");
                println!("    > {} decode sample.crashlog", self.app_name);
            }
        }

        println!("Options:");
        println!("     -h, --help   print this help");
        println!("     -w, --wait   wait for input before exiting");
    }
}
