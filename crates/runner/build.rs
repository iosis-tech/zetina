use std::process::Command;

fn main() {
    // Check if cairo-run command is present
    Command::new("cairo-run")
        .arg("--version")
        .output()
        .expect("Failed to execute cairo-run command");

    // Check if cairo-compile command is present
    Command::new("cairo-compile")
        .arg("--version")
        .output()
        .expect("Failed to execute cairo-compile command");
}
