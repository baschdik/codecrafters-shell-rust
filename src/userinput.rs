use std::io::{Stdout, Write, stdin, stdout};
use termion::cursor::DetectCursorPos;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor};

use crate::KindofCmd::Builtin;
use crate::builtin::Builtins;
use crate::misc::all_cmd_in_path;

use super::history::{CmdHistory, HistHandling};

trait HandleKeyEnvent: Write {
    fn write_char(&mut self, char: &char);
    fn del_lastchar(&mut self);
    fn write_str_to_current_line(&mut self, line: &str);
}

impl HandleKeyEnvent for RawTerminal<Stdout> {
    fn write_char(&mut self, char: &char) {
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

pub fn handle_userinput(cmd_history: &mut CmdHistory) -> Vec<String> {
    print!("$ ");
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    stdout.flush().unwrap();

    let mut user_input = String::new();
    let mut which_history_entry = 0;
    let mut history_search_down = false;
    // TODO: Update the whole history / up / down logic
    for evt in stdin.events() {
        let evt = evt.unwrap();
        match evt {
            Event::Key(Key::Char('\n')) => {
                user_input.push('\n');
                stdout.flush().unwrap();
                break;
            }
            Event::Key(Key::Up) => {
                if let Some(cmd_string) = cmd_history.get_latest(which_history_entry) {
                    stdout.write_str_to_current_line(&cmd_string);
                    user_input.clear();
                    user_input += &cmd_string[..];
                    which_history_entry += 1;
                    history_search_down = false;
                }
            }
            Event::Key(Key::Down) => {
                if !history_search_down {
                    which_history_entry = which_history_entry.checked_sub(1).unwrap_or_default();
                    history_search_down = true;
                }
                which_history_entry = which_history_entry.checked_sub(1).unwrap_or_default();

                if let Some(cmd_string) = cmd_history.get_latest(which_history_entry) {
                    stdout.write_str_to_current_line(&cmd_string);
                    user_input.clear();
                    user_input += &cmd_string[..];
                }
            }
            Event::Key(Key::Char('\t')) => {
                user_input = do_tab_completion(user_input, &mut stdout);
                stdout.write_str_to_current_line(&user_input);
            }
            Event::Key(Key::Backspace) => {
                user_input.pop();
                stdout.del_lastchar();
            }
            Event::Key(Key::Char(char)) => {
                user_input.push(char);
                stdout.write_char(&char);
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

fn do_tab_completion(mut user_input: String, stdout: &mut RawTerminal<Stdout>) -> String {
    let last_word = match user_input.split_whitespace().last() {
        Some(x) => x,
        None => return user_input,
    };
    let matches: Vec<String> = Builtins::all_cmd_names()
        .into_iter()
        .filter(|cmd| cmd.starts_with(last_word))
        .collect();
    if !matches.is_empty() {
        user_input = user_input.strip_suffix(last_word).unwrap().to_string();
        user_input.push_str(&matches[0]);
        user_input.push(' ');
        return user_input;
    }

    let matches: Vec<String>;
    if let Some(external_cmds) = all_cmd_in_path() {
        matches = external_cmds
            .into_iter()
            .filter(|cmd| cmd.starts_with(last_word))
            .collect();
    } else {
        matches = Vec::new();
    }

    //TODO: code double!
    if !matches.is_empty() {
        user_input = user_input.strip_suffix(last_word).unwrap().to_string();
        user_input.push_str(&matches[0]);
        user_input.push(' ');
        return user_input;
    } else {
        stdout.write_char(&'\x07'); //Ring the bell
    }
    user_input
}
