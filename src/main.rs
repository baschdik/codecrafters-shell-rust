use is_executable::is_executable;
use std::fs::File;
use std::io::{self, Error, Write};
use std::{env, fs};
use std::{env::var, path::PathBuf, process::Command, str::FromStr};
use thiserror::Error;

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

struct CmdHistory {
    data: Vec<String>,
    line_written_to_file: Option<usize>,
}

trait HistHandling {
    fn show(&self);
    fn show_last(&self, num_entry_toshow: usize);
    fn read_in(&mut self, path: &str) -> Result<usize, io::Error>;
    fn write_to(&mut self, path: &str);
    fn append_to(&mut self, path: &str);
}

impl HistHandling for CmdHistory {
    fn show(&self) {
        self.show_last(self.data.len());
    }

    fn show_last(&self, no_entry_toshow: usize) {
        let from_index = self.data.len() - no_entry_toshow;
        for (current_index, entry) in self.data[from_index..].iter().enumerate() {
            println!("{:>5} {}", current_index + 1 + from_index, entry);
        }
    }

    fn write_to(&mut self, path: &str) {
        let history_str = self.data.join("\n") + "\n";
        fs::write(path, history_str).expect("Failed to write to history file!");
        self.line_written_to_file = Some(self.data.len() - 1)
    }

    fn append_to(&mut self, path: &str) {
        let write_from = match self.line_written_to_file {
            None => 0,
            Some(n) => n + 1,
        };
        let history_str = self.data[write_from..].join("\n") + "\n";
        let mut f = File::options()
            .append(true)
            .open(path)
            .expect("Failed to open history file!");
        write!(&mut f, "{}", history_str).expect("Failed to append to history file!");
        self.line_written_to_file = Some(self.data.len() - 1)
    }

    fn read_in(&mut self, path: &str) -> Result<usize, io::Error> {
        //Don't append an empty file
        let metadata = fs::metadata(path)?;
        if metadata.len() == 0 {
            return Err(Error::new(io::ErrorKind::Other, "File is empty"));
        }

        let mut history_file_content: Vec<String> = fs::read_to_string(path)?
            .trim()
            .split("\n")
            .map(|s| s.to_owned())
            .collect();
        self.data.append(&mut history_file_content);
        Ok(self.data.len())
    }
}

fn main() {
    let mut cmd_history = CmdHistory {
        data: Vec::new(),
        line_written_to_file: None,
    };

    if let Ok(histfile) = var("HISTFILE") {
        if let Ok(n) = cmd_history.read_in(&histfile) {
            cmd_history.line_written_to_file = Some(n - 1);
        }
    };

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

fn get_userinput(cmd_history: &mut CmdHistory) -> Vec<String> {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).unwrap();

    user_input = user_input.trim().to_string();
    cmd_history.data.push(user_input.to_owned());

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

fn builtin_history(user_str: Vec<String>, cmd_history: &mut CmdHistory) {
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
