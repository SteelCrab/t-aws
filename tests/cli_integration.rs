use std::process::Command;

fn emd_bin() -> &'static str {
    env!("CARGO_BIN_EXE_emd")
}

#[test]
fn cli_help_runs_without_aws_account() {
    let output = Command::new(emd_bin())
        .arg("--help")
        .output()
        .expect("run --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage"));
}

#[test]
fn cli_version_runs_without_aws_account() {
    let output = Command::new(emd_bin())
        .arg("--version")
        .output()
        .expect("run --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.2.0"));
}

#[test]
fn cli_update_help_is_accessible() {
    let output = Command::new(emd_bin())
        .args(["update", "--help"])
        .output()
        .expect("run update --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("update"));
}
