use std::{path::PathBuf, str::FromStr};
use thiserror::Error;

use crate::{
    builtin_cmd::{self, Builtins},
    external_cmd::{self},
};

#[derive(Debug, PartialEq, Eq)]
pub enum CommandKind {
    Builtin(builtin_cmd::Builtins),
    External(PathBuf),
}

impl CommandKind {
    pub fn has_sideeffects(&self) -> bool {
        match self {
            CommandKind::Builtin(builtin) => builtin.has_sideeffects(),
            CommandKind::External(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputDirection {
    Terminal,
    File(PathBuf),
    Append(PathBuf),
}

#[derive(Debug)]
pub struct UserCommand {
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

impl UserCommand {
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

        Ok(UserCommand {
            cmd,
            args,
            out_direct: stdout,
            err_direct: stderr,
        })
    }
    pub fn is_builtin(&self) -> bool {
        matches!(self.cmd, CommandKind::Builtin(_))
    }
}

type CmdPipeline = Vec<UserCommand>;
type CmdPipelines = Vec<CmdPipeline>;

pub fn parse_cmd_set(tokens: &[String]) -> Result<CmdPipelines, CommandErrors> {
    // Accept a trailing semicolon.
    let tokens = if tokens.last().is_some_and(|t| t == ";") {
        &tokens[..tokens.len() - 1]
    } else {
        tokens
    };

    if tokens.is_empty() {
        return Err(CommandErrors::NoInput);
    }

    tokens
        .split(|token| token == ";")
        .map(|pipeline| {
            pipeline
                .split(|token| token == "|")
                .map(|cmd| {
                    if cmd.is_empty() {
                        return Err(CommandErrors::NoInput);
                    }

                    UserCommand::parse(cmd)
                })
                .collect()
        })
        .collect()
}
