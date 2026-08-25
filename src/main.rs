#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

enum Builtins {
    Echo,
    Exit,
    Type,
}

impl FromStr for Builtins {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "echo" => Ok(Builtins::Echo),
            "exit" => Ok(Builtins::Exit),
            "type" => Ok(Builtins::Type),
            _ => Err(()),
        }
    }
}

fn main() {
    loop {
        let user_input_split = get_userinput();
        let command = user_input_split[0].parse::<Builtins>();

        if command.is_err() {
            println!("{}: command not found", &user_input_split[0]);
            continue;
        }

        match command.unwrap() {
            Builtins::Echo => builtin_echo(user_input_split),
            Builtins::Exit => builtin_exit(),
            Builtins::Type => builtin_type(user_input_split),
        }
    }
}

fn get_userinput() -> Vec<String> {
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).unwrap();
    user_input = user_input.trim().to_string();
    user_input.split_whitespace().map(String::from).collect()
}

fn builtin_echo(str_iter: Vec<String>) {
    for ele in &str_iter[1..] {
        print!("{} ", ele)
    }
    println!()
}

fn builtin_exit() {
    std::process::exit(0)
}

fn builtin_type(str_split: Vec<String>) {
    if let Ok(_) = str_split[1].parse::<Builtins>() {
        println!("{} is a shell builtin", &str_split[1]);
        return;
    }
    match get_funcpath_from_path(&str_split[1]) {
        Some(path) => println!("{} is {:?}", str_split[1], path),
        None => println!("{}: not found", str_split[1]),
    }
    /*
    match typed_cmd {
        Ok(_) => println!("{} is a shell builtin", str_split[1]),
        Err(_) => println!("{}: not found", str_split[1]),
    } */
}

fn get_funcpath_from_path(cmd: &str) -> Option<PathBuf> {
    //TODO!
    println!("{}", cmd);
    Some(PathBuf::from("/test/123.exe"))
}
