#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        user_input = user_input.trim().to_string();

        match user_input.as_str() {
            "exit" => std::process::exit(0),
            _ => println!("{user_input}: command not found"),
        }
    }
}
