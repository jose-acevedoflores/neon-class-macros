use std::path::PathBuf;
use std::process::Command;

mod utils;

#[cfg(feature = "for-tests")]
fn check_feature() {}

#[cfg(not(feature = "for-tests"))]
fn check_feature() {
    panic!("Running tests without the 'for-tests' feature will result in errors. Use the alias `cargo t`");
}

#[test]
fn node_tests() {
    let npm_cmd = utils::locate_npm();
    run_cmd(npm_cmd);
}

#[test]
fn try_build_tests() {
    let cargo_cmd = PathBuf::from(env!("CARGO"));
    run_cmd(cargo_cmd);
}

fn run_cmd(cmd_path: PathBuf) {
    check_feature();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path_to_node_tests = manifest_dir.join("node_tests");

    let mut cmd = Command::new(cmd_path);

    cmd.args(["test"])
        .current_dir(path_to_node_tests)
        .spawn()
        .unwrap_or_else(|_| panic!("Could not spawn command {:?}", cmd))
        .wait()
        .unwrap_or_else(|_| panic!("Could not wait command {:?}", cmd));
}
