//! The only crate that prints (ADR 0001).

mod command;
mod json;
mod new_note;
mod reply;
mod translations;
mod validate;
mod vault_access;

use std::process::ExitCode;

use command::Command;
use reply::Reply;

fn main() -> ExitCode {
    let command = Command::parse(std::env::args().skip(1));
    Reply::for_command(&command).send()
}
