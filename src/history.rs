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
    fn show(&self);
    fn show_last(&self, num_entry_toshow: usize);
    fn read_in(&mut self, path: &str) -> Result<usize, io::Error>;
    fn write_to(&mut self, path: &str);
    fn append_to(&mut self, path: &str);
    fn get_from_latest(&self, entry_num: usize) -> Option<String>;
    fn init() -> CmdHistory;
}

impl HistHandling for CmdHistory {
    fn show(&self) {
        self.show_last(self.data.len());
    }

    fn show_last(&self, no_entry_toshow: usize) {
        let from_index = self.data.len() - no_entry_toshow;
        for (current_index, entry) in self.data[from_index..].iter().enumerate() {
            println!("{:>5} {}", current_index + 1 + from_index, entry);
        }
    }

    fn write_to(&mut self, path: &str) {
        let history_str = self.data.join("\n") + "\n";
        fs::write(path, history_str).expect("Failed to write to history file!");
        self.line_written_to_file = Some(self.data.len() - 1)
    }

    fn append_to(&mut self, path: &str) {
        let write_from = match self.line_written_to_file {
            None => 0,
            Some(n) => n + 1,
        };
        let history_str = self.data[write_from..].join("\n") + "\n";
        let mut f = File::options()
            .append(true)
            .open(path)
            .expect("Failed to open history file!");
        write!(&mut f, "{}", history_str).expect("Failed to append to history file!");
        self.line_written_to_file = Some(self.data.len() - 1)
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

    fn get_from_latest(&self, entry_num: usize) -> Option<String> {
        if (self.data.len() as i64 - entry_num as i64 - 1) < 0 {
            return None;
        }
        let from_index = self.data.len() - entry_num - 1;
        Some(self.data[from_index].to_string())
        //TODO: Implement None if i is to high
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
