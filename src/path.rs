use is_executable::is_executable;
use std::{
    collections::HashSet,
    env::{self, var},
    fs,
    path::PathBuf,
};

pub fn get_cmd_from_path(cmd: &str) -> Option<PathBuf> {
    let path = var("PATH").expect("No $PATH found.");
    for entry in path.split(":") {
        let full_cmd = entry.to_owned() + "/" + cmd;
        if is_executable(&full_cmd) {
            return Some(PathBuf::from(full_cmd));
        }
    }
    None
}
pub fn all_cmd_in_path() -> Option<HashSet<String>> {
    let path = env::var_os("PATH")?;

    let cmds: HashSet<_> = env::split_paths(&path)
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| is_executable(entry.path()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    (!cmds.is_empty()).then_some(cmds)
}
