use std::path::PathBuf;
use std::process::Command;

#[cfg(feature = "for-tests")]
fn check_feature() {}

#[cfg(not(feature = "for-tests"))]
fn check_feature() {
    panic!("Running tests without the 'for-tests' feature will result in errors. Use the alias `cargo t`");
}

#[cfg(target_os = "linux")]
#[test]
fn node_tests() {
    let cmd = Command::new("npm");
    run_cmd(cmd);
}

#[cfg(target_os = "windows")]
#[test]
fn node_tests() {
    let mut cmd = Command::new("cmd.exe");
    cmd.args(["/C", "npm"]);
    run_cmd(cmd);
}

#[test]
fn try_build_tests() {
    let cargo_cmd = PathBuf::from(env!("CARGO"));
    let cmd = Command::new(cargo_cmd);
    run_cmd(cmd);
}

fn run_cmd(mut cmd: Command) {
    check_feature();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path_to_node_tests = manifest_dir.join("node_tests");

    let res = cmd
        .args(["test"])
        .current_dir(path_to_node_tests)
        .spawn()
        .unwrap_or_else(|_| panic!("Could not spawn command {:?}", cmd))
        .wait()
        .unwrap_or_else(|_| panic!("Could not wait command {:?}", cmd))
        .success();

    assert!(res);
}
