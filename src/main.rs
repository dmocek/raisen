/*
 * This file is part of raisen.
 *
 * Copyright (C) 2023 Darryl Mocek
 * Portions Copyright (C) soft.github.io/run-or-raise/
 *
 * raisen is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * raisen is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with raisen.  If not, see <https://www.gnu.org/licenses/>.
 */

mod conditions;
mod parsing;
mod windows;

use anyhow::{bail, Context, Result};
// use log::{debug, error, SetLoggerError, Level, LevelFilter, log_enabled, info};
use std::{env, thread, time};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process;
use std::process::Command;
use env_logger::Logger;
use syslog::{Facility, Formatter3164};
use xcb::Connection;

pub static RAISEN_SOCKET: &'static str = "/tmp/raisen.sock";
pub static PARAM_DAEMON: &'static str = "--daemon";

// Spawns a child program and keeps running.
fn exec_program(prog: &str, args: &[String]) {
    let child_res = Command::new(prog).args(args).spawn();

    match child_res {
        Ok(child) => {log::info!("Spawning with id: {:?}", child.id())},
        Err(error) => match error.kind() {
            other_error => {
                log::error!("Problem exec'ing program: {:?}", other_error);
            }
        },
    };
}

fn handle_stream(mut unix_stream: UnixStream) -> anyhow::Result<()> {
    let mut message = String::new();
    unix_stream
        .read_to_string(&mut message)
        .context("Failed at reading the unix stream")?;

    log::info!("We received this message: {}\n", message);
    run_raise_cycle(message)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "raisen".into(),
        pid: 0,
    };

    let _logger = Logger::from_default_env();

    // Get the default env_logger
    let _res = match syslog::unix(formatter) {
        Ok(res) => {
            log::info!("Using syslog as the logger.", );
            let _logger= res;
        },
        Err(_res) => {
            log::error!("Cannot connect to syslog, using default logger.", );
        },
    };

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: raisen --daemon");
        println!("Usage: raisen 'class~\"Google-chrome\"' google-chrome");
        return Ok(());
    } else if args.len() == 2 &&  PARAM_DAEMON.eq(&args[1]) {
            log::info!("Running as daemon...");

            if let Err(err) = run_daemon() {
                log::error!("{}: {}", env!("CARGO_BIN_NAME"), err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| log::error!("caused by:\n{}", cause));
                let ten_millis = time::Duration::from_secs(10);
                thread::sleep(ten_millis);
                process::exit(1);
            }

        return Ok(());
    } else {
        log::info!("Running as client...");

        let mut unix_stream =
            UnixStream::connect(RAISEN_SOCKET).context("Could not connect to raisen socket.")?;

        write_request(&mut unix_stream, args)?;
        return Ok(());
    }
}

fn print_type_of<T>(_: &T) {
    log::info!("{}", std::any::type_name::<T>())
}

fn run_raise_cycle(args_str: String) -> Result<()> {
    let args: Vec<_> = env::args().collect();
    print_type_of(&args);

    log::info!("argsStr: {}", args_str);
    let exec_args: Vec<_> = args_str.split(",").map(str::to_string).collect();
    print_type_of(&exec_args);
    log::info!("exec_args: {:?}", exec_args);
    let app = &exec_args[0];

    let (condition, prog, prog_args) = if exec_args.len() >= 3 {
        (&exec_args[1], &exec_args[2], &exec_args[3..])
    } else {
        bail!("{} CONDITION PROGRAM [ARGS...]", app);
    };

    let cond = condition.parse()?;
    let (conn, screen_num) = Connection::connect(None)?;
    let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();

    match windows::find_matching_window(&conn, &screen, &cond)? {
        Some(win) => windows::set_active_window(&conn, &screen, win)?,
        None => exec_program(prog, prog_args),
    }

    conn.flush().map_err(Into::into)
}

fn run_daemon() -> anyhow::Result<()> {
    loop {
        if std::fs::metadata(RAISEN_SOCKET).is_ok() {
            log::warn!("A socket is already present. Deleting...");
            std::fs::remove_file(RAISEN_SOCKET).with_context(|| {
                format!("could not delete previous socket at {:?}", RAISEN_SOCKET)
            })?;
        }

        let unix_listener =
            UnixListener::bind(RAISEN_SOCKET).context("Could not create the unix socket")?;

        // put the server logic in a loop to accept several connections
        loop {
            let (unix_stream, _socket_address) = unix_listener
                .accept()
                .context("Failed at accepting a connection on the unix listener")?;
            handle_stream(unix_stream)?;
        }
    }
}

fn write_request(unix_stream: &mut UnixStream, args: Vec<String>) -> anyhow::Result<()> {
    let args_str = args
        .iter()
        .map(|args| args.to_string())
        .collect::<Vec<_>>()
        .join(",");

    log::info!("argsStr: {}", args_str);

    unix_stream
        .write(args_str.as_bytes())
        .context("Failed at writing onto the unix stream")?;

    Ok(())
}
