use assert_cmd::cargo::cargo_bin; // macro
use std::process::Command;

#[test]
fn cli_starts_and_prints_help() {
    let mut cmd = Command::new(cargo_bin!("rstress"));
    cmd.arg("--help");
    let status = cmd.status().expect("failed to run rstress");
    assert!(status.success());
}