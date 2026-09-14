use std::{
    env::var,
    fs::{self, File},
    io::{self, Error, Write},
};

pub struct CmdHistory {
    pub data: Vec<String>,
    pub line_written_to_file: Option<usize>,
}

pub trait HistHandling {
    fn show(&self) -> String;
    fn show_last(&self, num_entry_toshow: usize) -> String;
    fn read_in(&mut self, path: &str) -> Result<usize, io::Error>;
    fn write_to(&mut self, path: &str) -> String;
    fn append_to(&mut self, path: &str) -> String;
    fn get_latest(&self, entry_num: usize) -> Option<String>;
    fn init() -> CmdHistory;
}

impl HistHandling for CmdHistory {
    fn show(&self) -> String {
        self.show_last(self.data.len())
    }

    fn show_last(&self, num_entry_toshow: usize) -> String {
        let mut output = String::new();
        let from_index = self.data.len() - num_entry_toshow;
        for (current_index, entry) in self.data[from_index..].iter().enumerate() {
            output.push_str(format!("{:>5} {}", current_index + 1 + from_index, &entry).as_ref());
            output.push('\n');
        }
        output
    }

    fn write_to(&mut self, path: &str) -> String {
        let history_str = self.data.join("\n") + "\n";
        self.line_written_to_file = Some(self.data.len() - 1);
        match fs::write(path, history_str) {
            Ok(_) => "".to_string(),
            Err(_) => "Failed to write to history file!".to_string(),
        }
    }

    fn append_to(&mut self, path: &str) -> String {
        let write_from = match self.line_written_to_file {
            None => 0,
            Some(n) => n + 1,
        };
        let history_str = self.data[write_from..].join("\n") + "\n";
        let mut f = File::options()
            .append(true)
            .open(path)
            .expect("Failed to open history file!");
        self.line_written_to_file = Some(self.data.len() - 1);
        match write!(&mut f, "{}", history_str) {
            Ok(_) => "".to_string(),
            Err(_) => "Failed to append to history file!".to_string(),
        }
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

    fn get_latest(&self, entry_num: usize) -> Option<String> {
        if (self.data.len() as i64 - entry_num as i64 - 1) < 0 {
            return None;
        }
        let from_index = self.data.len() - entry_num - 1;
        Some(self.data[from_index].to_string())
    }

    fn init() -> CmdHistory {
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
    }
}
