// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

#![no_main]
#![no_std]
#![feature(iter_intersperse)]

mod args;
mod decode;
mod extract;
mod pager;

extern crate alloc;

use uefi::prelude::*;
use uefi::println;
use uefi::proto::console::text::Input;

use crate::args::Args;
use crate::args::Command;
use log::{LevelFilter, error};

fn run_command(args: &Args) -> Result<(), uefi::Error> {
    if args.help {
        args.show_help();
        return Ok(());
    }

    if let Some(ref command) = args.command {
        match command {
            Command::Extract { output_path } => extract::extract(output_path.as_deref()),
            Command::Info { input_paths } => {
                for input_path in input_paths {
                    if let Err(err) = decode::info(input_path) {
                        println!("Cannot decode sample: {err}");
                    }
                }
                Ok(())
            }
            Command::Decode { input_path } => decode::decode(input_path),
        }
    } else {
        args.show_help();
        Err(uefi::Error::from(Status::INVALID_PARAMETER))
    }
}

fn wait_for_input() {
    println!("Press any key to continue");
    let _ = uefi::boot::get_handle_for_protocol::<Input>()
        .ok()
        .and_then(|handle| uefi::boot::open_protocol_exclusive::<Input>(handle).ok())
        .and_then(|input| input.wait_for_key_event())
        .map(|event| uefi::boot::wait_for_event(&mut [event]));
}

fn run_command_and_wait(args: &Args) -> Result<(), uefi::Error> {
    let res = run_command(args);
    if args.wait {
        wait_for_input();
    }
    res
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    log::set_max_level(LevelFilter::Error);

    match Args::parse() {
        Ok(args) => {
            log::set_max_level(match args.verbosity {
                0 => LevelFilter::Error,
                1 => LevelFilter::Warn,
                2 => LevelFilter::Info,
                3 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            });

            match run_command_and_wait(&args) {
                Ok(_) => Status::SUCCESS,
                Err(err) => err.status(),
            }
        }
        Err(err) => {
            error!("{err}");
            error!("Use --help option for more information.");
            Status::INVALID_PARAMETER
        }
    }
}
