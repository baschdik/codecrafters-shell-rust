#[allow(unused_imports)]
use std::io::{self, Write};
use std::{collections::HashMap, str::SplitWhitespace};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        user_input = user_input.trim().to_string();
        let mut user_input_iter = user_input.split_whitespace();

        let builtin_commands: HashMap<&str, fn(SplitWhitespace<'_>)> = HashMap::from([
            ("echo", builtin_echo as fn(SplitWhitespace<'_>)),
            ("exit", builtin_exit),
            ("type", builtin_type),
        ]);

        let user_input_1 = user_input_iter.next().expect("No Some here");

        if user_input_1 == "type" {
            let cmd_to_test = user_input_iter.next().expect("No Some here");
            if builtin_commands.contains_key(&cmd_to_test) {
                println!("{} is a shell builtin", cmd_to_test);
            } else {
                println!("{}: not found", cmd_to_test);
            }
            continue;
        }

        let current_command = builtin_commands.get(user_input_1);
        match current_command {
            Some(func) => func(user_input_iter),
            None => println!("{user_input}: command not found"),
        }
    }
}

fn builtin_echo(str_iter: SplitWhitespace<'_>) {
    for ele in str_iter {
        print!("{} ", ele)
    }
    println!()
}

fn builtin_exit(_str_iter: SplitWhitespace<'_>) {
    std::process::exit(0)
}

fn builtin_type(_str_iter: SplitWhitespace<'_>) {} // no implementation -> only to be found as builtin
