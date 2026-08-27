use is_executable::is_executable;
use std::io::{self, Write};
use std::{env, fs};
use std::{env::var, path::PathBuf, process::Command, str::FromStr};

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

enum Builtins {
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
            "echo" => Ok(Builtins::Echo),
            "exit" => Ok(Builtins::Exit),
            "type" => Ok(Builtins::Type),
            "pwd" => Ok(Builtins::Pwd),
            "cd" => Ok(Builtins::Cd),
            "history" => Ok(Builtins::History),
            _ => Err(()),
        }
    }
}

fn main() {
    let mut cmd_history: Vec<String> = Vec::new();

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
                Builtins::Exit => builtin_exit(),
                Builtins::Type => builtin_type(user_input_split),
                Builtins::Pwd => builtin_pwd(),
                Builtins::Cd => builtin_cd(user_input_split),
                Builtins::History => builtin_history(user_input_split, &mut cmd_history),
            },
            Ok(KindofCmd::External(_)) => run_external_cmd(user_input_split),
        }
    }
}

fn get_userinput(cmd_history: &mut Vec<String>) -> Vec<String> {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).unwrap();

    user_input = user_input.trim().to_string();
    cmd_history.push(user_input.to_owned());

    user_input.split_whitespace().map(String::from).collect()
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

// Builtin commands implementation starts here

fn builtin_cd(str_split: Vec<String>) {
    if str_split.len() == 1 {
        return;
    }
    let path_str = match &str_split[1][..] {
        "~" => var("HOME").expect("No $HOME found."),
        _ => str_split[1].to_string(),
    };

    if let Err(_) = env::set_current_dir(&path_str) {
        println!("cd: {}: No such file or directory", path_str)
    }
}

fn builtin_echo(user_str: Vec<String>) {
    for ele in &user_str[1..] {
        print!("{} ", ele)
    }
    println!()
}

fn builtin_exit() {
    std::process::exit(0)
}

fn builtin_history(user_str: Vec<String>, cmd_history: &mut Vec<String>) {
    let history_length = cmd_history.len();
    let mut number_cmds_toshow = history_length;
    let mut read_history_file = false;

    if user_str.len() >= 2 {
        if user_str[1] == "-r" {
            read_history_file = true;
        } else {
            number_cmds_toshow = match user_str[1].parse::<usize>() {
                Ok(val) if val == 0 || val >= history_length => history_length,
                Ok(val) => val,
                Err(_) => {
                    println!("Usage: history n  - n: Integer");
                    return;
                }
            };
        }
    }

    if read_history_file {
        if user_str.len() < 3 {
            println!("Usage: history -r <Path_to_History>")
        }
        /*let history_file_content =
            fs::read_to_string(&user_str[2]).expect("Reading history file failed!");
        let mut content_split: Vec<String> = history_file_content
            .trim()
            .split("\n")
            .map(|s| s.to_owned())
            .collect(); */
        let mut history_file_content: Vec<String> = fs::read_to_string(&user_str[2])
            .expect("Reading history file failed!")
            .trim()
            .split("\n")
            .map(|s| s.to_owned())
            .collect();
        cmd_history.append(&mut history_file_content);
        return;
    }

    let from_index = history_length - number_cmds_toshow;
    for (current_index, entry) in cmd_history[from_index..].iter().enumerate() {
        println!("{:>5} {}", current_index + 1 + from_index, entry)
    }
}

fn builtin_pwd() {
    println!("{}", env::current_dir().expect("pwd failed").display())
}

fn builtin_type(str_split: Vec<String>) {
    if let Ok(_) = str_split[1].parse::<Builtins>() {
        println!("{} is a shell builtin", &str_split[1]);
        return;
    }
    match get_cmd_from_path(&str_split[1]) {
        Some(path) => println!("{} is {}", str_split[1], path.display()),
        None => println!("{}: not found", str_split[1]),
    }
}
