mod conditions;
mod parsing;
mod windows;

use anyhow::{bail, Context, Error, Result};
use std::{env, thread, time};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process;
use std::process::Command;
use xcb::Connection;

pub static RAISEN_SOCKET: &'static str = "/tmp/raisen.sock";
pub static PARAM_DAEMON: &'static str = "--daemon";

fn exec_program(prog: &str, args: &[String]) -> Error {
    let error = Command::new(prog).args(args).exec();
    Error::new(error).context("Executing program failed")
}

fn handle_stream(mut unix_stream: UnixStream) -> anyhow::Result<()> {
    let mut message = String::new();
    unix_stream
        .read_to_string(&mut message)
        .context("Failed at reading the unix stream")?;

    println!("We received this message: {}\n", message);
    run_raise_cycle(message)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: raisen --daemon");
        println!("Usage: raisen 'class~\"Google-chrome\"' google-chrome");
        // process::exit(1);
        return Ok(());
    } else if args.len() == 2 &&  PARAM_DAEMON.eq(&args[1]) {
            println!("Running as daemon...");

            if let Err(err) = run_daemon() {
                eprintln!("{}: {}", env!("CARGO_BIN_NAME"), err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!("caused by:\n{}", cause));
                let ten_millis = time::Duration::from_secs(10);
                thread::sleep(ten_millis);
                process::exit(1);
            }

        return Ok(());
    } else {
        println!("Running as client...");

        let mut unix_stream =
            UnixStream::connect(RAISEN_SOCKET).context("Could not connect to raisen socket.")?;

        write_request(&mut unix_stream, args)?;
        return Ok(());
    }

    Ok(())
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn run_raise_cycle(argsStr: String) -> Result<()> {
    let args: Vec<_> = env::args().collect();
    print_type_of(&args);
    // let app = &args[0];

    println!("argsStr: {}", argsStr);
    let mut exec_args: Vec<_> = argsStr.split(",").map(str::to_string).collect();
    print_type_of(&exec_args);
    // let exec_args: Vec<_> = gs().collect();
    println!("exec_args: {:?}", exec_args);
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
        None => return Err(exec_program(prog, prog_args)),
    }
    conn.flush().map_err(Into::into)
}

fn run_daemon() -> anyhow::Result<()> {
    loop {
        if std::fs::metadata(RAISEN_SOCKET).is_ok() {
            println!("A socket is already present. Deleting...");
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
    let argsStr = args
        .iter()
        .map(|args| args.to_string())
        .collect::<Vec<_>>()
        .join(",");

    println!("argsStr: {}", argsStr);

    unix_stream
        // .write(args.into_iter().collect()::<String>().as_bytes())
        .write(argsStr.as_bytes())
        .context("Failed at writing onto the unix stream")?;

    Ok(())
}
