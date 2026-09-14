use crate::OutputStrings;

use super::cmd_parser;
use is_executable::is_executable;
use std::{
    collections::HashSet,
    env::{self, var},
    fs,
    path::PathBuf,
    process,
};

pub fn from_path(cmd: &str) -> Option<PathBuf> {
    let path = var("PATH").expect("No $PATH found.");
    for entry in path.split(":") {
        let full_cmd = entry.to_owned() + "/" + cmd;
        if is_executable(&full_cmd) {
            return Some(PathBuf::from(full_cmd));
        }
    }
    None
}
pub fn all_in_path() -> Option<HashSet<String>> {
    let path = env::var_os("PATH")?;

    let cmds: HashSet<_> = env::split_paths(&path)
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| is_executable(entry.path()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    (!cmds.is_empty()).then_some(cmds)
}

pub fn run(cmd: &cmd_parser::Command) -> OutputStrings {
    let cmd_parser::CommandKind::External(path) = &cmd.cmd else {
        panic!("expected external cmd")
    };
    let output = process::Command::new(path.file_name().expect("No filename in command path"))
        .args(&cmd.args[..])
        .output()
        .expect("failed to run process");
    let output_msg = String::from_utf8(output.stdout);
    let error_msg = String::from_utf8(output.stderr);
    OutputStrings {
        out: format!("{}", output_msg.unwrap()),
        err: format!("{}", error_msg.unwrap()),
    }
}
