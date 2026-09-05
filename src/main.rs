use is_executable::is_executable;
use std::fs::File;
use std::io::{self, Error, Stdout, Write, stdin, stdout};
use std::{env, fs};
use std::{env::var, path::PathBuf, process::Command, str::FromStr};
use termion::cursor::DetectCursorPos;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor};
use thiserror::Error;

mod history;
use history::{CmdHistory, HistHandling};

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

/*fn create_init_cmd_history() -> CmdHistory {
    let mut cmd_history = CmdHistory {
        data: Vec::new(),
        line_written_to_file: None,
    };

    if let Ok(histfile) = var("HISTFILE") {
        if let Ok(n) = cmd_history.read_in(&histfile) {
            cmd_history.line_written_to_file = Some(n - 1);
        }
    };
    cmd_history
}*/

trait HandleKeyEnvent: Write {
    fn show_char(&mut self, char: &char);
    fn del_lastchar(&mut self);
    fn write_str_to_current_line(&mut self, line: &str);
}

impl HandleKeyEnvent for RawTerminal<Stdout> {
    fn show_char(&mut self, char: &char) {
        _ = write!(self, "{}", char,);
        self.flush().unwrap()
    }
    fn del_lastchar(&mut self) {
        if let Ok((col, _)) = self.cursor_pos() {
            if col <= 3 {
                return; //Dont delete the Prompt on Screen
            }
        }
        _ = write!(self, "\x08{}", clear::AfterCursor);
        self.flush().unwrap();
    }
    fn write_str_to_current_line(&mut self, line: &str) {
        self.flush().unwrap();
        _ = write!(
            self,
            "{}{}$ {}",
            clear::CurrentLine,
            cursor::Left(999), //this in error-prone, but stdout.cursor_pos() doesn't work in codecrafter test env
            line
        );
        self.flush().unwrap();
    }
}

fn get_userinput(cmd_history: &mut CmdHistory) -> Vec<String> {
    print!("$ ");
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    stdout.flush().unwrap();

    let mut user_input = String::new();
    let mut last_cmd_counter = 0;
    for evt in stdin.events() {
        let evt = evt.unwrap();
        match evt {
            Event::Key(Key::Char('\n')) => {
                user_input.push('\n');
                stdout.flush().unwrap();
                break;
            }
            Event::Key(Key::Up) => {
                if let Some(cmd_string) = cmd_history.get_from_latest(last_cmd_counter) {
                    stdout.write_str_to_current_line(&cmd_string);
                    user_input.clear();
                    user_input += &cmd_string[..];
                    last_cmd_counter += 1;
                }
            }
            Event::Key(Key::Backspace) => {
                user_input.pop();
                stdout.del_lastchar();
            }
            Event::Key(Key::Char(char)) => {
                user_input.push(char);
                stdout.show_char(&char);
            }
            _ => continue,
        }
    }

    stdout.suspend_raw_mode().unwrap();
    println!("");

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

fn builtin_cd(user_str: Vec<String>) {
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

fn builtin_echo(user_str: Vec<String>) {
    for ele in &user_str[1..] {
        print!("{} ", ele)
    }
    println!()
}

fn builtin_exit(cmd_history: &mut CmdHistory) {
    if let Ok(histfile) = var("HISTFILE") {
        cmd_history.write_to(&histfile);
    }
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
