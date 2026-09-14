use std::collections::HashSet;
use std::env;
use std::{env::var, str::FromStr};
use strum::{EnumIter, IntoEnumIterator};
use thiserror::Error;

use crate::external_cmd;
use crate::history;
use crate::history::HistHandling;

#[derive(Debug, EnumIter)]
pub enum Builtins {
    Echo,
    Exit,
    Type,
    Pwd,
    Cd,
    History,
}

impl FromStr for Builtins {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cd" => Ok(Builtins::Cd),
            "echo" => Ok(Builtins::Echo),
            "exit" => Ok(Builtins::Exit),
            "history" => Ok(Builtins::History),
            "pwd" => Ok(Builtins::Pwd),
            "type" => Ok(Builtins::Type),
            _ => Err(()),
        }
    }
}

impl Builtins {
    fn get_cmd_name(&self) -> String {
        match &self {
            Builtins::Cd => "cd".to_string(),
            Builtins::Echo => "echo".to_string(),
            Builtins::Exit => "exit".to_string(),
            Builtins::History => "history".to_string(),
            Builtins::Pwd => "pwd".to_string(),
            Builtins::Type => "type".to_string(),
        }
    }

    pub fn all_cmd_names() -> HashSet<String> {
        let mut names = HashSet::new();
        for ele in Builtins::iter() {
            names.insert(ele.get_cmd_name());
        }
        names
    }

    pub fn execute(&self, args: &[String], cmd_history: &mut history::CmdHistory) -> String {
        match self {
            Builtins::Cd => builtin_cd(args),
            Builtins::Echo => builtin_echo(args),
            Builtins::Exit => builtin_exit(cmd_history),
            Builtins::History => builtin_history(args, cmd_history),
            Builtins::Pwd => builtin_pwd(),
            Builtins::Type => builtin_type(args),
        }
    }
}

pub fn builtin_cd(args: &[String]) -> String {
    let path_str = match args.first() {
        Some(s) if s == "~" => var("HOME").expect("No $HOME found."),
        Some(_) => args[0].to_string(),
        None => return "".to_string(),
    };

    if let Err(_) = env::set_current_dir(&path_str) {
        return format!("cd: {}: No such file or directory", path_str);
    } else {
        return "".to_string();
    }
}

pub fn builtin_echo(args: &[String]) -> String {
    String::from(args.join(" "))
}

pub fn builtin_exit(cmd_history: &mut history::CmdHistory) -> String {
    if let Ok(histfile) = var("HISTFILE") {
        cmd_history.write_to(&histfile);
    }
    std::process::exit(0)
}

pub fn builtin_history(args: &[String], cmd_history: &mut history::CmdHistory) -> String {
    return match HistoryArgs::new(args) {
        Err(e) => {
            format!("{}", e)
        }
        Ok(HistoryArgs::Show) => cmd_history.show(),
        Ok(HistoryArgs::ShowLast(n)) => cmd_history.show_last(n),
        Ok(HistoryArgs::ReadHistory(path)) => match cmd_history.read_in(&path) {
            Ok(_) => "".to_string(),
            Err(e) => e.to_string(),
        },
        Ok(HistoryArgs::WriteHistory(path)) => cmd_history.write_to(&path),
        Ok(HistoryArgs::AppendHistory(path)) => cmd_history.append_to(&path),
    };
    enum HistoryArgs {
        Show,
        ShowLast(usize),
        ReadHistory(String),
        WriteHistory(String),
        AppendHistory(String),
    }

    impl HistoryArgs {
        fn new(user_str: &[String]) -> Result<HistoryArgs, HistoryArgErrors> {
            if user_str.len() == 0 {
                return Ok(HistoryArgs::Show);
            }
            match &user_str[0][..] {
                "-r" => {
                    if user_str.len() >= 3 {
                        return Ok(HistoryArgs::ReadHistory(user_str[1].to_string()));
                    } else {
                        return Err(HistoryArgErrors::MissingPathArgument(
                            "Usage: history -r <Path_to_History>".to_string(),
                        ));
                    }
                }
                "-w" => {
                    if user_str.len() >= 3 {
                        return Ok(HistoryArgs::WriteHistory(user_str[1].to_string()));
                    } else {
                        return Err(HistoryArgErrors::MissingPathArgument(
                            "Usage: history -w <Path_to_History>".to_string(),
                        ));
                    }
                }
                "-a" => {
                    if user_str.len() >= 3 {
                        return Ok(HistoryArgs::AppendHistory(user_str[1].to_string()));
                    } else {
                        return Err(HistoryArgErrors::MissingPathArgument(
                            "Usage: history -a <Path_to_History>".to_string(),
                        ));
                    }
                }
                n => match n.parse::<usize>() {
                    Ok(val) => return Ok(HistoryArgs::ShowLast(val)),
                    Err(_) => return Err(HistoryArgErrors::NoIntArg),
                },
            };
        }
    }

    #[derive(Error, Debug)]
    enum HistoryArgErrors {
        #[error("Missing Path Argument: {0}")]
        MissingPathArgument(String),

        #[error("Number of Entrys to Show must be Integer")]
        NoIntArg,
    }
}

pub fn builtin_pwd() -> String {
    format!("{}", env::current_dir().expect("pwd failed").display())
}

pub fn builtin_type(args: &[String]) -> String {
    if args.len() == 0 {
        return "".to_string();
    }

    if let Ok(_) = args[0].parse::<Builtins>() {
        return format!("{} is a shell builtin", &args[0]);
    }
    match external_cmd::from_path(&args[0]) {
        Some(path) => format!("{} is {}", args[0], path.display()),
        None => format!("{}: not found", args[0]),
    }
}
