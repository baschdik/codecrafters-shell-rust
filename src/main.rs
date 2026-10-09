use std::process::{Command, Stdio};

fn main() {
    print!("$ ");
    let child1 = Command::new("tail")
        .args(["-f", "Cargo.toml"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("First child cmd gone wrong");

    let child1_out = child1.stdout.expect("child1 stdout doesn't open");

    let child2 = Command::new("head")
        .args(["-n", "5"])
        .stdin(Stdio::from(child1_out))
        .stdout(Stdio::piped())
        .spawn()
        .expect("child2 cmd gone wrong");

    let output = child2.wait_with_output().expect("child2 stdout went wrong");

    //println!("Child2 out: {:?}", output);
}
