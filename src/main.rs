mod package_types;

use serde::Deserialize;

use std::fs;

use crate::package_types::PackagesConfig;

#[derive(Debug, Deserialize)]
enum RunMode {
    #[serde(rename = "idempotent")]
    Idempotent,
    #[serde(rename = "imperative")]
    Imperative,
}

#[derive(Debug, Deserialize)]
struct Config {
    mode: RunMode,
    packages: Option<PackagesConfig>,
}

fn main() -> std::io::Result<()> {
    let f = fs::read_to_string("./bow.yaml").unwrap();

    match serde_yaml_bw::from_str(&f) {
        Ok(Config { mode, packages }) => {
            if let Some(packages) = packages {
                packages.install(mode)?
            }
        }
        Err(e) => eprintln!("{e}"),
    }

    Ok(())
}
