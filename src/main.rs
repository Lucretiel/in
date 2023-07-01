/*
Copyright 2023 Nathan West

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use std::{
    borrow::Cow,
    convert::Infallible,
    env,
    os::unix::process::CommandExt,
    path,
    process::{Command, ExitCode},
};

use clap::Parser;
use joinery::prelude::*;
use shell_escape;

#[derive(Parser, Debug)]
struct Args {
    /// If given, the PWD variable is not modified
    #[clap(long, short)]
    no_pwd: bool,

    /// The path to change to before running the command
    path: path::PathBuf,

    /// The command to run
    command: String,

    /// Arguments to be forwarded to the command
    args: Vec<String>,
}

fn main() -> Result<Infallible, ExitCode> {
    let args: Args = Args::parse();

    // We could use Command::current_dir, but we want to be able to use
    // `env::current_dir` to update `PWD`
    env::set_current_dir(&args.path).map_err(|err| {
        let directory = args.path.display();
        eprintln!("Failed to change working directory to {directory}:\n  {err}",);

        ExitCode::FAILURE
    })?;

    // We could use Command::env, but it's too annoying to do conditionally
    // (because the command builder is based on &mut self rather than self)
    if !args.no_pwd {
        let absolute_dir = env::current_dir().map_err(|err| {
            let directory = args.path.display();
            eprintln!("Failed to change working directory to {directory}:\n  {err}",);

            ExitCode::FAILURE
        })?;

        env::set_var("PWD", &absolute_dir);
    }

    let err = Command::new(args.command.as_str()).args(&args.args).exec();

    let formatted_command = [&args.command]
        .into_iter()
        .chain(&args.args)
        .map(|arg| shell_escape::escape(Cow::Borrowed(arg.as_str())))
        .join_with(" ");

    eprintln!("Failed to run command: {formatted_command}\n  {err}");

    Err(ExitCode::FAILURE)
}
