use is_executable::is_executable;
use std::env;
use std::io::{Stdout, Write, stdin, stdout};
use std::{env::var, path::PathBuf, process::Command, str::FromStr};
use termion::cursor::DetectCursorPos;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor};
use thiserror::Error;

use super::history::{CmdHistory, HistHandling};

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

pub fn get_userinput(cmd_history: &mut CmdHistory) -> Vec<String> {
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
