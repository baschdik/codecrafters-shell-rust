use std::collections::HashSet;
use std::env;
use std::{env::var, str::FromStr};
use strum::{EnumIter, IntoEnumIterator};
use thiserror::Error;

use crate::misc;
use crate::{builtin, history::*};

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

    pub fn execute_cmd(self, user_str: Vec<String>, cmd_history: &mut CmdHistory) {
        match self {
            Builtins::Echo => builtin_echo(user_str),
            Builtins::Exit => builtin_exit(cmd_history),
            Builtins::Type => builtin_type(user_str),
            Builtins::Pwd => builtin_pwd(),
            Builtins::Cd => builtin_cd(user_str),
            Builtins::History => builtin_history(user_str, cmd_history),
        }
    }
}

pub fn builtin_cd(user_str: Vec<String>) {
    if user_str.len() == 1 {
        return;
    }
    let path_str = match &user_str[1][..] {
        "~" => var("HOME").expect("No $HOME found."),
        _ => user_str[1].to_string(),
    };

    if let Err(_) = env::set_current_dir(&path_str) {
        println!("cd: {}: No such file or directory", path_str)
    }
}

pub fn builtin_echo(user_str: Vec<String>) {
    for ele in &user_str[1..] {
        print!("{} ", ele)
    }
    println!()
}

pub fn builtin_exit(cmd_history: &mut CmdHistory) {
    if let Ok(histfile) = var("HISTFILE") {
        cmd_history.write_to(&histfile);
    }
    std::process::exit(0)
}

pub fn builtin_history(user_str: Vec<String>, cmd_history: &mut CmdHistory) {
    match HistoryArgs::new(user_str) {
        Err(e) => {
            println!("{}", e);
            return;
        }
        Ok(HistoryArgs::Show) => cmd_history.show(),
        Ok(HistoryArgs::ShowLast(n)) => cmd_history.show_last(n),
        Ok(HistoryArgs::ReadHistory(path)) => {
            cmd_history
                .read_in(&path)
                .expect("Couldn't read Histfile at given path");
        }
        Ok(HistoryArgs::WriteHistory(path)) => cmd_history.write_to(&path),
        Ok(HistoryArgs::AppendHistory(path)) => cmd_history.append_to(&path),
    }
    enum HistoryArgs {
        Show,
        ShowLast(usize),
        ReadHistory(String),
        WriteHistory(String),
        AppendHistory(String),
    }

    impl HistoryArgs {
        fn new(user_str: Vec<String>) -> Result<HistoryArgs, HistoryArgErrors> {
            if user_str.len() == 1 {
                return Ok(HistoryArgs::Show);
            }
            match &user_str[1][..] {
                "-r" => {
                    if user_str.len() >= 3 {
                        return Ok(HistoryArgs::ReadHistory(user_str[2].to_string()));
                    } else {
                        return Err(HistoryArgErrors::MissingPathArgument(
                            "Usage: history -r <Path_to_History>".to_string(),
                        ));
                    }
                }
                "-w" => {
                    if user_str.len() >= 3 {
                        return Ok(HistoryArgs::WriteHistory(user_str[2].to_string()));
                    } else {
                        return Err(HistoryArgErrors::MissingPathArgument(
                            "Usage: history -w <Path_to_History>".to_string(),
                        ));
                    }
                }
                "-a" => {
                    if user_str.len() >= 3 {
                        return Ok(HistoryArgs::AppendHistory(user_str[2].to_string()));
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

pub fn builtin_pwd() {
    println!("{}", env::current_dir().expect("pwd failed").display())
}

pub fn builtin_type(str_split: Vec<String>) {
    if let Ok(_) = str_split[1].parse::<Builtins>() {
        println!("{} is a shell builtin", &str_split[1]);
        return;
    }
    match misc::get_cmd_from_path(&str_split[1]) {
        Some(path) => println!("{} is {}", str_split[1], path.display()),
        None => println!("{}: not found", str_split[1]),
    }
}
