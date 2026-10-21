//! BRUSH - Baschdik's Rust-based Shell
//!
//! A simple Unix shell implementation

use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
};

mod builtin_cmd;
mod cmd_parser;
mod external_cmd;
mod history;
mod tokenizer;
mod userinput;

use crate::{
    cmd_parser::{
        CommandKind,
        OutputDirection::{self},
        UserCommand,
    },
    history::HistHandling,
};
use userinput::get_userinput;

/// Holds out and err strings
#[derive(Debug)]
struct OutputStrings {
    out: String,
    err: String,
    out_direction: OutputDirection,
    err_direction: OutputDirection,
}

impl OutputStrings {
    /// Writes Output and Error independently in the given direction
    fn write(&self) {
        match &self.out_direction {
            OutputDirection::Terminal => print!("{}", self.out),
            OutputDirection::File(path) => {
                fs::write(path, &self.out).expect("Failed to write to file")
            }
            OutputDirection::Append(path) => Self::append_file(&self.out, &path),
        }
        match &self.err_direction {
            OutputDirection::Terminal => eprint!("{}", self.err),
            OutputDirection::File(path) => {
                fs::write(path, &self.err).expect("Failed to write to file")
            }
            OutputDirection::Append(path) => Self::append_file(&self.err, &path),
        }
    }
    fn append_file(str: &str, path: &PathBuf) {
        let mut f = File::options()
            .append(true)
            .create(true)
            .open(path)
            .expect("Failed to open file!");
        write!(&mut f, "{}", str).expect("Failed to append to file");
    }
    /// Creates OutputStrings struct if only Output is given (no error)
    fn from_out(out: &str, out_direction: &OutputDirection) -> Self {
        let line_end = if !out.is_empty() && !out.ends_with("\n") {
            "\n"
        } else {
            ""
        };
        Self {
            out: out.to_string() + line_end,
            err: "".to_string(),
            out_direction: out_direction.to_owned(),
            err_direction: OutputDirection::Terminal,
        }
    }
    /// Creates OutputStrings struct if only error is given (no output)
    fn from_err(err: &str, err_direction: &OutputDirection) -> Self {
        let line_end = if !err.is_empty() && !err.ends_with("\n") {
            "\n"
        } else {
            ""
        };
        Self {
            out: "".to_string(),
            err: err.to_string() + line_end,
            out_direction: OutputDirection::Terminal,
            err_direction: err_direction.to_owned(),
        }
    }
}

fn main() {
    let mut cmd_history = history::CmdHistory::init();
    loop {
        let user_input = get_userinput(&mut cmd_history);
        if user_input.is_empty() {
            continue;
            //break; //DEBUG
        }

        let user_input_tokens = tokenizer::tokenize(&user_input);

        let user_commands = cmd_parser::parse_cmd_set(&user_input_tokens);
        //println!("CmdSet: {:?}", user_commands); //DEBUG

        match user_commands {
            Err(e) => {
                OutputStrings::from_err(&(e.to_string() + "\n"), &OutputDirection::Terminal)
                    .write();
            }
            Ok(cmds) => {
                for pipeline in cmds {
                    run_pipeline(pipeline, &mut cmd_history).write();
                }
            }
        }
    }

    // To go one here and actually implement the pipeline!
    fn run_pipeline(
        pipeline: Vec<cmd_parser::UserCommand>,
        cmd_history: &mut history::CmdHistory,
    ) -> OutputStrings {
        //println!("Thats the pipeline: {:?}", pipeline); //DEBUG
        if let Some(cmd) = pipeline.first()
            && let cmd_parser::CommandKind::Builtin(cmd_builtin) = &cmd.cmd
        {
            return OutputStrings::from_out(
                &cmd_builtin.execute(&cmd.args, cmd_history),
                &cmd.out_direct,
            );
        };
        //external_cmd::run(pipeline.get(0).expect("Ohh no, pipeline is empty :(")) // This implements the old behavior w/o pipelines

        trait ExecutePipe {
            fn create_process(&self) -> std::process::Command;
        }

        impl ExecutePipe for UserCommand {
            fn create_process(&self) -> Command {
                let CommandKind::External(cmd_path) = &self.cmd else {
                    panic!("Should be external command")
                };

                let mut process =
                    Command::new(cmd_path.file_name().expect("No filename in command path"));
                process.args(&self.args);
                process
            }
        }

        let last_cmd_no = pipeline.len() - 1;
        let last_cmd = pipeline.last().expect("Pipeline is empty!");
        let out_redirected = !(last_cmd.out_direct == OutputDirection::Terminal);
        let err_redirected = !(last_cmd.err_direct == OutputDirection::Terminal);

        let mut prev_stdout = None;
        let mut children = Vec::new();

        for (i, cmd) in pipeline.iter().enumerate() {
            let is_last = i == last_cmd_no;
            let mut process = cmd.create_process();

            if let Some(prev) = prev_stdout.take() {
                process.stdin(Stdio::from(prev));
            }

            if is_last && !out_redirected {
                // head now sees a terminal and is line-buffered, like in bash
                process.stdout(Stdio::inherit());
            } else {
                process.stdout(Stdio::piped());
            }

            if is_last {
                process.stderr(if err_redirected {
                    Stdio::piped()
                } else {
                    Stdio::inherit()
                });
            }

            let mut child = process.spawn().expect("Spawning failed");
            if !is_last {
                prev_stdout = child.stdout.take();
            }
            children.push(child);
        }

        let mut last_child = children.pop().expect("No Child in Children");

        // stderr in its own thread, otherwise reading stdout and stderr one after
        // the other can deadlock when the child fills the stderr pipe buffer.
        let err_thread = last_child.stderr.take().map(|mut e| {
            thread::spawn(move || {
                let mut buf = Vec::new();
                let _ = e.read_to_end(&mut buf);
                buf
            })
        });

        // Only Some(..) if stdout was redirected (piped). With Stdio::inherit()
        // the child writes directly to the terminal and this is None.
        let mut captured: Vec<u8> = Vec::new();
        if let Some(mut child_out) = last_child.stdout.take() {
            let _ = child_out.read_to_end(&mut captured);
        }

        let _ = last_child.wait();

        // Clean up the earlier stages (e.g. `tail -f`, which is still running after `head` exited)
        for mut child in children {
            let _ = child.kill();
            let _ = child.wait();
        }

        let err_bytes = err_thread
            .map(|h| h.join().unwrap_or_default())
            .unwrap_or_default();

        OutputStrings {
            out: String::from_utf8_lossy(&captured).into_owned(), // empty if inherited
            err: String::from_utf8_lossy(&err_bytes).into_owned(),
            out_direction: last_cmd.out_direct.to_owned(),
            err_direction: last_cmd.err_direct.to_owned(),
        }
    }
}
