/*
Copyright 2020 Nathan West

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use joinery::prelude::*;
use libc;
use shell_escape;
use std::{env, ffi, io, path, process, ptr};
use structopt::{clap::AppSettings, StructOpt};

fn parse_cstring(input: &str) -> Result<ffi::CString, ffi::NulError> {
    ffi::CString::new(input.as_bytes())
}

fn run_io(task: impl FnOnce() -> libc::c_int) -> io::Result<()> {
    if task() < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
#[structopt(setting = AppSettings::TrailingVarArg)]
struct Args {
    /// If given, the PWD variable is not modified
    #[structopt(long, short)]
    no_pwd: bool,

    /// The path to change to before running the command
    path: path::PathBuf,

    /// The command to run
    #[structopt(required=true, parse(try_from_str = parse_cstring))]
    command: Vec<ffi::CString>,
}

fn main() {
    let args: Args = Args::from_args();

    env::set_current_dir(&args.path).unwrap_or_else(|err| {
        eprintln!(
            "Failed to change working directory to {}:\n  {}",
            args.path.display(),
            err
        );
        process::exit(1);
    });

    if !args.no_pwd {
        let absolute_dir = env::current_dir().unwrap_or_else(|err| {
            eprintln!(
                "Failed to change working directory to {}:\n  {}",
                &args.path.display(),
                err
            );
            process::exit(1);
        });

        env::set_var("PWD", &absolute_dir);
    }

    let program = args.command[0].as_ptr();

    let command: Vec<*const libc::c_char> = args
        .command
        .iter()
        .map(|arg| arg.as_ptr())
        .chain(Some(ptr::null()))
        .collect();

    run_io(|| unsafe { libc::execvp(program, command.as_ptr()) }).unwrap_or_else(|err| {
        let command = args
            .command
            .iter()
            .map(|arg| arg.to_string_lossy())
            .map(|arg| shell_escape::escape(arg))
            .join_with(" ");

        eprintln!("Failed to run command: {}\n  {}", command, err);
        process::exit(1);
    });
}
