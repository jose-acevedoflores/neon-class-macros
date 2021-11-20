use std::path::PathBuf;
use std::process::Command;

mod utils;

#[test]
fn try_build() {
    let t = trybuild::TestCases::new();
    t.pass("derived_class/src/derived_class.rs");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let n = manifest_dir.join("derived_class");
    let npm_path = utils::locate_npm();
    let mut cmd = Command::new(npm_path);

    cmd.arg("test")
        .current_dir(n)
        .spawn()
        .unwrap_or_else(|_| panic!("Could not run command {:?}", cmd))
        .wait()
        .unwrap_or_else(|_| panic!("Could not run command {:?}", cmd));
}
