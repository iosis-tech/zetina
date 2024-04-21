use std::process::Command;

fn main() {
    // Check if cairo-run command is present
    check_command("cairo-run");

    // Check if cairo-compile command is present
    check_command("cairo-compile");
}

fn check_command(cmd: &str) {
    match Command::new(cmd).arg("--version").output() {
        Ok(_) => println!("{} command found", cmd),
        Err(e) => panic!("Failed to execute {} command: {}", cmd, e),
    }
}
