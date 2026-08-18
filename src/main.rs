#[allow(unused_imports)]
use std::io::{self, Write};
use std::str::SplitWhitespace;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        user_input = user_input.trim().to_string();
        let mut user_input_iter = user_input.split_whitespace();

        match user_input_iter.next() {
            Some("exit") => std::process::exit(0),
            Some("echo") => print_slices(user_input_iter),
            _ => println!("{user_input}: command not found"),
        }
    }
}

fn print_slices(str_iter: SplitWhitespace<'_>) {
    for ele in str_iter {
        print!("{} ", ele)
    }
    println!()
}
