use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Check if cairo-run command is present
    check_command("cairo-run");

    // Check if cairo-compile command is present
    check_command("cairo-compile");

    let workspace_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"))
            .join("../../");
    let out_dir =
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR env not present")).join("../../../../");
    let cairo_path = PathBuf::from(env::var("CAIRO_PATH").expect("CAIRO_PATH env not present"));
    let bootloader_path =
        PathBuf::from(env::var("BOOTLOADER_PATH").expect("BOOTLOADER_PATH env not present"));
    let bootloader_out_name = PathBuf::from(
        env::var("BOOTLOADER_OUT_NAME").expect("BOOTLOADER_OUT_NAME env not present"),
    );

    // Compile Bootloader
    Command::new("cairo-compile")
        .arg("--cairo_path")
        .arg(workspace_root.join(&cairo_path))
        .arg(workspace_root.join(&cairo_path).join(bootloader_path))
        .arg("--output")
        .arg(out_dir.join(bootloader_out_name))
        .arg("--proof_mode")
        .arg("--no_debug_info")
        .output()
        .expect("bootloader compile failed");
}

fn check_command(cmd: &str) {
    match Command::new(cmd).arg("--version").output() {
        Ok(_) => println!("{} command found", cmd),
        Err(e) => panic!("Failed to execute {} command: {}", cmd, e),
    }
}
