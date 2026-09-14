use std::{path::PathBuf, str::FromStr};
use thiserror::Error;

use crate::{
    builtin_cmd::{self, Builtins},
    external_cmd::{self},
    history::CmdHistory,
};

#[derive(Debug)]
pub enum CommandKind {
    Builtin(builtin_cmd::Builtins),
    External(PathBuf),
}

#[derive(Debug)]
pub enum OutputDirection {
    Terminal,
    File(PathBuf),
    Append(PathBuf),
}

#[derive(Debug)]
pub struct Command {
    pub cmd: CommandKind,
    pub args: Vec<String>,
    pub out_direct: OutputDirection,
    pub err_direct: OutputDirection,
}

#[derive(Error, Debug)]
pub enum CommandErrors {
    #[error("input vector empty")]
    NoInput,
    #[error("{0}: command not found")]
    CmdNotFound(String),
    #[error("Missing path after redirection operator ({0})")]
    MissingRedirectionPath(String),
}

impl Command {
    pub fn parse(input: &[String]) -> Result<Self, CommandErrors> {
        // parsing...

        let cmd_arg = input.first().ok_or(CommandErrors::NoInput)?;

        let cmd = Builtins::from_str(cmd_arg)
            .map(CommandKind::Builtin)
            .or_else(|()| {
                external_cmd::from_path(cmd_arg)
                    .map(CommandKind::External)
                    .ok_or(())
            })
            .map_err(|_| CommandErrors::CmdNotFound(cmd_arg.clone()))?;

        let mut args: Vec<String> = Vec::new();
        let mut stdout = OutputDirection::Terminal;
        let mut stderr = OutputDirection::Terminal;

        let mut iter = input.iter().skip(1);
        while let Some(ele) = iter.next() {
            if matches!(ele.as_str(), ">" | "1>") {
                match iter.next() {
                    Some(path) => stdout = OutputDirection::File(PathBuf::from(path)),
                    None => {
                        return Err(CommandErrors::MissingRedirectionPath("Stdout".to_string()));
                    }
                }
            } else if matches!(ele.as_str(), ">>" | "1>>") {
                match iter.next() {
                    Some(path) => stdout = OutputDirection::Append(PathBuf::from(path)),
                    None => {
                        return Err(CommandErrors::MissingRedirectionPath("Stdout".to_string()));
                    }
                }
            } else if ele == "2>" {
                match iter.next() {
                    Some(path) => stderr = OutputDirection::File(PathBuf::from(path)),
                    None => {
                        return Err(CommandErrors::MissingRedirectionPath("Stderr".to_string()));
                    }
                }
            } else if ele == "2>>" {
                match iter.next() {
                    Some(path) => stderr = OutputDirection::Append(PathBuf::from(path)),
                    None => {
                        return Err(CommandErrors::MissingRedirectionPath("Stderr".to_string()));
                    }
                }
            } else {
                args.push(ele.to_string());
            }
        }

        Ok(Command {
            cmd,
            args,
            out_direct: stdout,
            err_direct: stderr,
        })
    }
}
