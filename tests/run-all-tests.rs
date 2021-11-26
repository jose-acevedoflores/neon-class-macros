use std::path::PathBuf;
use std::process::Command;

mod utils;

#[test]
fn node_tests() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let path_to_node_tests = manifest_dir.join("node_tests");
    let npm_path = utils::locate_npm();
    let mut cmd = Command::new(npm_path);

    cmd.arg("test")
        .current_dir(path_to_node_tests)
        .spawn()
        .unwrap_or_else(|_| panic!("Could not spawn command {:?}", cmd))
        .wait()
        .unwrap_or_else(|_| panic!("Could not wait command {:?}", cmd));
}

#[test]
fn try_build_tests() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_cmd = PathBuf::from(env!("CARGO"));

    let path_to_node_tests = manifest_dir.join("node_tests");

    let mut cmd = Command::new(cargo_cmd);

    cmd.args(["test"])
        .current_dir(path_to_node_tests)
        .spawn()
        .unwrap_or_else(|_| panic!("Could not spawn command {:?}", cmd))
        .wait()
        .unwrap_or_else(|_| panic!("Could not wait command {:?}", cmd));
}
