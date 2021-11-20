use std::path::PathBuf;

pub fn locate_npm() -> PathBuf {
    #[cfg(target_os = "windows")]
    const DEFAULT_NPM_LOCATION: &str = r"C:\Program Files\nodejs\npm.cmd";
    #[cfg(target_os = "linux")]
    const DEFAULT_NPM_LOCATION: &str = "npm";
    const NPM_LOCATION_ENV_KEY: &str = "NPM_LOCATION";

    if !PathBuf::from(DEFAULT_NPM_LOCATION).exists() {
        let npm_location = std::env::var(NPM_LOCATION_ENV_KEY);
        match npm_location {
            Ok(npm_path) => {
                let p = PathBuf::from(&npm_path)
                    .canonicalize()
                    .expect("NPM_LOCATION to be valid");
                if !p.exists() {
                    eprintln!(
                        "\nCould not find 'npm' at [{:?}] based on environment variable '{}' \n",
                        p.as_os_str(),
                        NPM_LOCATION_ENV_KEY
                    );
                    std::process::exit(-1);
                } else {
                    println!("running with npm located at {:?}", p);
                    p
                }
            }
            Err(_) => {
                eprintln!(
                    "\nNpm was not found in the default location [{}]. \n Try setting the '{}' environment variable\n",
                    DEFAULT_NPM_LOCATION,
                    NPM_LOCATION_ENV_KEY
                );
                std::process::exit(-1);
            }
        }
    } else {
        PathBuf::from(DEFAULT_NPM_LOCATION)
    }
}
