mod history;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::{
    cmd_parser::Command,
    cmd_parser::OutputDirection::{self},
    history::HistHandling,
};

mod builtin_cmd;

mod userinput;
use userinput::get_userinput;

mod cmd_parser;

mod external_cmd;

#[derive(Debug)]
struct OutputStrings {
    out: String,
    err: String,
}

impl OutputStrings {
    fn write(&self, out_direction: &OutputDirection, err_direction: &OutputDirection) {
        match out_direction {
            OutputDirection::Terminal => print!("{}", self.out),
            OutputDirection::File(path) => {
                fs::write(path, &self.out).expect("Failed to write to file")
            }
            OutputDirection::Append(path) => Self::append_file(&self.out, path),
        }
        match err_direction {
            OutputDirection::Terminal => eprint!("{}", self.err),
            OutputDirection::File(path) => {
                fs::write(path, &self.err).expect("Failed to write to file")
            }
            OutputDirection::Append(path) => Self::append_file(&self.err, path),
        }
    }
    fn append_file(str: &str, path: &PathBuf) {
        let mut f = File::options()
            .append(true)
            .open(path)
            .expect("Failed to open file!");
        write!(&mut f, "{}", str).expect("Failed to append to file");
    }
    fn from_out(out: &str) -> Self {
        let line_end = if !out.is_empty() && !out.ends_with("\n") {
            "\n"
        } else {
            ""
        };
        Self {
            out: out.to_string() + line_end,
            err: "".to_string(),
        }
    }
    fn from_err(err: &str) -> Self {
        let line_end = if !err.is_empty() && !err.ends_with("\n") {
            "\n"
        } else {
            ""
        };
        Self {
            out: "".to_string(),
            err: err.to_string() + line_end,
        }
    }
}

fn main() {
    let mut cmd_history = history::CmdHistory::init();

    loop {
        let user_input_split = get_userinput(&mut cmd_history);
        if user_input_split.len() == 0 {
            continue;
        }

        let command = cmd_parser::Command::parse(user_input_split.as_ref());
        match &command {
            Err(e) => {
                OutputStrings::from_err(&(e.to_string() + "\n"))
                    .write(&OutputDirection::Terminal, &OutputDirection::Terminal);
            }
            Ok(cmd) => match &cmd.cmd {
                cmd_parser::CommandKind::Builtin(builtin_cmd) => {
                    OutputStrings::from_out(&builtin_cmd.execute(&cmd.args, &mut cmd_history))
                        .write(&cmd.out_direct, &cmd.err_direct);
                }
                cmd_parser::CommandKind::External(_) => {
                    external_cmd::run(cmd).write(&cmd.out_direct, &cmd.err_direct)
                }
            },
        };
    }
}
