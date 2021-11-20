use std::path::PathBuf;
use std::process::Command;

mod utils;

#[test]
fn try_build() {
    let t = trybuild::TestCases::new();
    t.pass("node_tests/src/derived_class.rs");

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let path_to_node_tests = manifest_dir.join("node_tests");
    let npm_path = utils::locate_npm();
    let mut cmd = Command::new(npm_path);

    cmd.arg("test")
        .current_dir(path_to_node_tests)
        .spawn()
        .unwrap_or_else(|_| panic!("Could not run command {:?}", cmd))
        .wait()
        .unwrap_or_else(|_| panic!("Could not run command {:?}", cmd));
}
