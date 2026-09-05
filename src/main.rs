use is_executable::is_executable;
use std::{env::var, path::PathBuf, process::Command, str::FromStr};

mod history;
use history::{CmdHistory, HistHandling};

mod builtin;
use builtin::*;

mod userinput;
use userinput::get_userinput;

#[allow(dead_code)]
enum KindofCmd {
    Builtin(Builtins),
    External(PathBuf),
}

impl FromStr for KindofCmd {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(builtin) = s.parse::<Builtins>() {
            return Ok(KindofCmd::Builtin(builtin));
        }
        if let Some(path_buf) = get_cmd_from_path(s) {
            return Ok(KindofCmd::External(path_buf));
        }
        Err(())
    }
}

fn main() {
    let mut cmd_history = CmdHistory::init();

    loop {
        let user_input_split = get_userinput(&mut cmd_history);

        if user_input_split.len() == 0 {
            continue;
        }
        let command = user_input_split[0].parse::<KindofCmd>();
        match command {
            Err(_) => println!("{}: command not found", &user_input_split[0]),
            Ok(KindofCmd::Builtin(cmd)) => match cmd {
                Builtins::Echo => builtin_echo(user_input_split),
                Builtins::Exit => builtin_exit(&mut cmd_history),
                Builtins::Type => builtin_type(user_input_split),
                Builtins::Pwd => builtin_pwd(),
                Builtins::Cd => builtin_cd(user_input_split),
                Builtins::History => builtin_history(user_input_split, &mut cmd_history),
            },
            Ok(KindofCmd::External(_)) => run_external_cmd(user_input_split),
        }
    }
}

fn get_cmd_from_path(cmd: &str) -> Option<PathBuf> {
    let path = var("PATH").expect("No $PATH found.");
    for entry in path.split(":") {
        let full_cmd = entry.to_owned() + "/" + cmd;
        if is_executable(&full_cmd) {
            return Some(PathBuf::from(full_cmd));
        }
    }
    None
}

fn run_external_cmd(args: Vec<String>) {
    let output = Command::new(&args[0])
        .args(&args[1..])
        .output()
        .expect("failed to run process");
    let output_msg = String::from_utf8(output.stdout);
    print!("{}", output_msg.unwrap())
}
