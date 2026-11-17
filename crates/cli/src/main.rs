//! The only crate that prints (ADR 0001).

mod command;
mod reply;
mod translations;

use std::process::ExitCode;

use command::Command;
use reply::Reply;

fn main() -> ExitCode {
    let command = Command::parse(std::env::args().skip(1));
    Reply::for_command(&command).send()
}
